//! AssetLab transcoding worker.
//!
//! Polls S3_RAW_BUCKET/queue/ every 30 seconds for pending jobs.
//! For each job:
//!   1. Claims it by deleting the queue marker.
//!   2. Downloads the raw file from S3_RAW_BUCKET.
//!   3. Transcodes to 360p / 720p / 1080p via FFmpeg.
//!   4. Uploads renditions to S3_VIDEO_BUCKET.
//!   5. Deletes raw file from S3_RAW_BUCKET.
//!   6. Writes final status.json to S3_VIDEO_BUCKET.

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

    // ── 1. Download raw file ──────────────────────────────────────────────────
    let tmp_dir = std::env::temp_dir();
    let raw_path = tmp_dir.join(format!("{}.orig", id));

    tracing::info!(id, path = %raw_path.display(), "downloading raw file");
    download_from_s3(cfg, &format!("uploads/{}.orig", id), &raw_path, &cfg.raw_bucket).await?;

    // ── 2. Probe source height ────────────────────────────────────────────────
    let source_height = probe_height(&raw_path).await.unwrap_or_else(|e| {
        tracing::warn!(id, "ffprobe failed ({e}), assuming 720p");
        720
    });
    tracing::info!(id, source_height, "source probed");

    // ── 3. Transcode each rendition ───────────────────────────────────────────
    let target_heights: &[u32] = &[360, 720, 1080];
    let mut produced: Vec<String> = Vec::new();

    for &height in target_heights {
        if height > source_height {
            tracing::info!(id, height, "skipping (exceeds source)");
            continue;
        }

        let out_path = tmp_dir.join(format!("{}_{}.mp4", id, height));
        tracing::info!(id, height, "transcoding");

        let status = Command::new("ffmpeg")
            .args([
                "-fflags", "nobuffer",  // don't buffer input in RAM
                "-i",
                raw_path.to_str().unwrap(),
                "-c:v", "libx264",
                "-crf", "23",
                "-preset", "ultrafast",
                "-threads", "1",
                "-vf", &format!("scale=-2:{}", height),
                "-c:a", "aac",
                "-b:a", "128k",
                "-bufsize", "512k",     // cap output buffer size
                "-maxrate", "2M",       // cap peak bitrate to bound encoder RAM
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

        // Clean up local temp file immediately.
        let _ = tokio::fs::remove_file(&out_path).await;

        produced.push(format!("{}p", height));
    }

    // ── 4. Clean up raw file ──────────────────────────────────────────────────
    let _ = tokio::fs::remove_file(&raw_path).await;

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

/// Stream an S3 object to a local file.
async fn download_from_s3(
    cfg: &Config,
    key: &str,
    dest: &PathBuf,
    bucket: &str,
) -> anyhow::Result<()> {
    let mut output = cfg
        .s3
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;

    let mut file = tokio::fs::File::create(dest).await?;
    while let Some(chunk) = output.body.try_next().await? {
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    Ok(())
}

/// Upload a local file to S3_VIDEO_BUCKET using ByteStream::from_path.
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

/// Run ffprobe to get the video stream height. Returns the height in pixels.
async fn probe_height(path: &PathBuf) -> anyhow::Result<u32> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=height",
            "-of", "csv=p=0",
            path.to_str().unwrap(),
        ])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let height = stdout.trim().parse::<u32>()?;
    Ok(height)
}
