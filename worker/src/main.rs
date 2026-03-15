//! AssetLab transcoding worker.
//!
//! Polls S3_RAW_BUCKET/queue/ every 30 seconds for pending jobs.
//! For each job:
//!   1. Claims it by deleting the queue marker.
//!   2. Generates a presigned S3 URL — no download, no /tmp RAM usage.
//!   3. Passes the URL directly to FFmpeg (HTTP Range-based streaming).
//!   4. Uploads each rendition to S3_VIDEO_BUCKET.
//!   5. Deletes raw file from S3_RAW_BUCKET.
//!   6. Writes final status.json to S3_VIDEO_BUCKET.

use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use serde_json::json;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{interval, Duration};

// ── Config ────────────────────────────────────────────────────────────────────

struct Config {
    s3: aws_sdk_s3::Client,
    raw_bucket: String,
    video_bucket: String,
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
            ),
        )
        .init();

    let region = aws_types::region::Region::new(
        std::env::var("AWS_DEFAULT_REGION").unwrap_or_else(|_| "eu-west-3".to_string()),
    );
    let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region)
        .load()
        .await;

    let cfg = Config {
        s3: aws_sdk_s3::Client::new(&aws_config),
        raw_bucket: std::env::var("S3_RAW_BUCKET").expect("S3_RAW_BUCKET must be set"),
        video_bucket: std::env::var("S3_VIDEO_BUCKET").expect("S3_VIDEO_BUCKET must be set"),
    };

    tracing::info!(
        raw_bucket = %cfg.raw_bucket,
        video_bucket = %cfg.video_bucket,
        "worker started, polling every 30s"
    );

    let mut ticker = interval(Duration::from_secs(30));
    loop {
        ticker.tick().await;
        if let Err(e) = poll_and_process(&cfg).await {
            tracing::error!("poll error: {e}");
        }
    }
}

// ── Poll loop ─────────────────────────────────────────────────────────────────

async fn poll_and_process(cfg: &Config) -> anyhow::Result<()> {
    let list = cfg
        .s3
        .list_objects_v2()
        .bucket(&cfg.raw_bucket)
        .prefix("queue/")
        .send()
        .await?;

    let keys: Vec<String> = list
        .contents()
        .iter()
        .filter_map(|obj| obj.key().map(str::to_owned))
        .collect();

    if keys.is_empty() {
        tracing::debug!("no pending jobs");
        return Ok(());
    }

    for queue_key in keys {
        let id = match queue_key.strip_prefix("queue/") {
            Some(id) if !id.is_empty() => id.to_owned(),
            _ => continue,
        };

        tracing::info!(id, "picked up job");

        // Claim: delete queue marker first so no other worker instance grabs it.
        cfg.s3
            .delete_object()
            .bucket(&cfg.raw_bucket)
            .key(&queue_key)
            .send()
            .await?;

        // Process — write error status on failure so the frontend can surface it.
        if let Err(e) = process_video(cfg, &id).await {
            tracing::error!(id, "transcoding failed: {e}");
            let _ = write_status(cfg, &id, json!({ "status": "error", "error": e.to_string() })).await;
        }
    }

    Ok(())
}

// ── Transcoding pipeline ──────────────────────────────────────────────────────

async fn process_video(cfg: &Config, id: &str) -> anyhow::Result<()> {
    // Mark as processing so the API can report it immediately.
    write_status(cfg, id, json!({ "status": "processing" })).await?;

    // ── 1. Presign the raw S3 object — no download needed ────────────────────
    // FFmpeg reads via HTTP Range requests, so the raw file never touches RAM.
    let presign_cfg = PresigningConfig::expires_in(std::time::Duration::from_secs(7200))?;
    let presigned = cfg
        .s3
        .get_object()
        .bucket(&cfg.raw_bucket)
        .key(format!("uploads/{}.orig", id))
        .presigned(presign_cfg)
        .await?;
    let input_url = presigned.uri().to_string();

    tracing::info!(id, "presigned input URL generated (no download)");

    // ── 2. Probe source height via the presigned URL ──────────────────────────
    let source_height = probe_height(&input_url).await.unwrap_or_else(|e| {
        tracing::warn!(id, "ffprobe failed ({e}), assuming 720p");
        720
    });
    tracing::info!(id, source_height, "source probed");

    // ── 3. Transcode each rendition ───────────────────────────────────────────
    let tmp_dir = std::env::temp_dir();
    let target_heights: &[u32] = &[360, 720, 1080];
    let mut produced: Vec<String> = Vec::new();

    for &height in target_heights {
        if height > source_height {
            tracing::info!(id, height, "skipping (exceeds source)");
            continue;
        }

        let out_path = tmp_dir.join(format!("{}_{}.mp4", id, height));
        tracing::info!(id, height, "transcoding");

        // The scale filter does two things:
        //   1. scale=w=min(iw\,1920):h=-2  — cap input at 1920px wide first,
        //      so FFmpeg never holds full 4K frames in RAM even for small outputs.
        //   2. scale=-2:HEIGHT             — then scale to the target height.
        // Both happen inside a single filter chain so only one decode pass occurs.
        let vf = format!(
            "scale=w='min(iw,1920)':h=-2,scale=-2:{}",
            height
        );

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                &input_url,           // stream directly from S3, no local copy
                "-c:v", "libx264",
                "-crf", "22",
                "-preset", "ultrafast",
                "-threads", "1",
                "-vf", &vf,
                "-c:a", "aac",
                "-b:a", "128k",
                "-movflags", "+faststart",
                "-y",
                out_path.to_str().unwrap(),
            ])
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!("ffmpeg failed for {}p (exit {})", height, status);
        }

        // Upload rendition to video bucket.
        let s3_key = format!("videos/{}/{}p.mp4", id, height);
        tracing::info!(id, height, s3_key, "uploading rendition");
        upload_to_s3(cfg, &out_path, &s3_key).await?;

        // Clean up local temp file immediately to free disk.
        let _ = tokio::fs::remove_file(&out_path).await;

        produced.push(format!("{}p", height));
    }

    // ── 4. Delete raw file from S3 ────────────────────────────────────────────
    cfg.s3
        .delete_object()
        .bucket(&cfg.raw_bucket)
        .key(format!("uploads/{}.orig", id))
        .send()
        .await?;

    tracing::info!(id, ?produced, "raw file deleted from S3");

    // ── 5. Write done status ──────────────────────────────────────────────────
    write_status(cfg, id, json!({ "status": "done", "qualities": produced })).await?;
    tracing::info!(id, "job complete");

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Upload a local file to S3_VIDEO_BUCKET.
async fn upload_to_s3(cfg: &Config, src: &PathBuf, key: &str) -> anyhow::Result<()> {
    let body = ByteStream::from_path(src).await?;
    cfg.s3
        .put_object()
        .bucket(&cfg.video_bucket)
        .key(key)
        .content_type("video/mp4")
        .body(body)
        .send()
        .await?;
    Ok(())
}

/// Write a JSON value to videos/{id}/status.json in S3_VIDEO_BUCKET.
async fn write_status(cfg: &Config, id: &str, body: serde_json::Value) -> anyhow::Result<()> {
    let key = format!("videos/{}/status.json", id);
    let bytes = serde_json::to_vec(&body)?;
    cfg.s3
        .put_object()
        .bucket(&cfg.video_bucket)
        .key(key)
        .content_type("application/json")
        .body(ByteStream::from(bytes))
        .send()
        .await?;
    Ok(())
}

/// Run ffprobe against a URL to get the video stream height in pixels.
async fn probe_height(url: &str) -> anyhow::Result<u32> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=height",
            "-of", "csv=p=0",
            url,
        ])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let height = stdout.trim().parse::<u32>()?;
    Ok(height)
}
