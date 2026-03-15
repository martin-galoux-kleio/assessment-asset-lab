//! AssetLab transcoding worker — AWS MediaConvert edition.
//!
//! Polls S3_RAW_BUCKET/queue/ every 30 seconds for pending jobs.
//! For each job, submits an AWS MediaConvert job and polls until complete.
//! No FFmpeg, no local file I/O, no memory issues.

use aws_sdk_mediaconvert::types::{
    AudioCodec, AudioCodecSettings, AudioDefaultSelection, AudioDescription,
    AudioSelector, ContainerSettings, ContainerType, FileGroupSettings, H264RateControlMode,
    H264Settings, H264QvbrSettings, Input, JobSettings, Output, OutputGroup,
    OutputGroupSettings, OutputGroupType, VideoCodec, VideoCodecSettings, VideoDescription,
    AacSettings, AacCodingMode,
};
use aws_sdk_s3::primitives::ByteStream;
use serde_json::json;
use tokio::time::{interval, sleep, Duration};

// ── Config ────────────────────────────────────────────────────────────────────

struct Config {
    s3: aws_sdk_s3::Client,
    mediaconvert: aws_sdk_mediaconvert::Client,
    raw_bucket: String,
    video_bucket: String,
    mediaconvert_role_arn: String,
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .init();

    let region = aws_types::region::Region::new(
        std::env::var("AWS_DEFAULT_REGION").unwrap_or_else(|_| "eu-west-3".to_string()),
    );
    let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region)
        .load()
        .await;

    // Discover the account-specific MediaConvert endpoint automatically.
    // This avoids needing a MEDIACONVERT_ENDPOINT env var.
    let discovery_client = aws_sdk_mediaconvert::Client::new(&aws_config);
    let mediaconvert_endpoint = discovery_client
        .describe_endpoints()
        .send()
        .await
        .expect("failed to describe MediaConvert endpoints")
        .endpoints()
        .first()
        .and_then(|e| e.url())
        .expect("no MediaConvert endpoint returned")
        .to_string();

    tracing::info!(mediaconvert_endpoint, "discovered MediaConvert endpoint");

    let mc_config = aws_sdk_mediaconvert::config::Builder::from(&aws_config)
        .endpoint_url(&mediaconvert_endpoint)
        .build();

    let cfg = Config {
        s3: aws_sdk_s3::Client::new(&aws_config),
        mediaconvert: aws_sdk_mediaconvert::Client::from_conf(mc_config),
        raw_bucket: std::env::var("S3_RAW_BUCKET").expect("S3_RAW_BUCKET must be set"),
        video_bucket: std::env::var("S3_VIDEO_BUCKET").expect("S3_VIDEO_BUCKET must be set"),
        mediaconvert_role_arn: std::env::var("MEDIACONVERT_ROLE_ARN")
            .expect("MEDIACONVERT_ROLE_ARN must be set"),
    };

    tracing::info!(
        raw_bucket = %cfg.raw_bucket,
        video_bucket = %cfg.video_bucket,
        "worker started (MediaConvert mode), polling every 30s"
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

        // Claim: delete queue marker so no other instance grabs it.
        cfg.s3
            .delete_object()
            .bucket(&cfg.raw_bucket)
            .key(&queue_key)
            .send()
            .await?;

        if let Err(e) = process_video(cfg, &id).await {
            tracing::error!(id, "job failed: {e}");
            let _ = write_status(cfg, &id, json!({ "status": "error", "error": e.to_string() })).await;
        }
    }

    Ok(())
}

// ── MediaConvert job ──────────────────────────────────────────────────────────

async fn process_video(cfg: &Config, id: &str) -> anyhow::Result<()> {
    write_status(cfg, id, json!({ "status": "processing" })).await?;

    let input_s3 = format!("s3://{}/uploads/{}.orig", cfg.raw_bucket, id);
    let output_prefix = format!("s3://{}/videos/{}/", cfg.video_bucket, id);

    // ── Build MediaConvert job ────────────────────────────────────────────────

    let input = Input::builder()
        .file_input(&input_s3)
        .audio_selectors(
            "Audio Selector 1",
            AudioSelector::builder()
                .default_selection(AudioDefaultSelection::Default)
                .build(),
        )
        .video_selector(aws_sdk_mediaconvert::types::VideoSelector::builder().build())
        .build();

    // Helper: build one H.264 output at a given height and max bitrate (bps).
    fn make_output(name_modifier: &str, height: i32, max_bitrate: i32) -> Output {
        Output::builder()
            .name_modifier(name_modifier)
            .video_description(
                VideoDescription::builder()
                    .height(height)
                    // FIT_NO_UPSCALE: if source is smaller than target, keep source size.
                    .scaling_behavior(aws_sdk_mediaconvert::types::ScalingBehavior::FitNoUpscale)
                    .codec_settings(
                        VideoCodecSettings::builder()
                            .codec(VideoCodec::H264)
                            .h264_settings(
                                H264Settings::builder()
                                    .rate_control_mode(H264RateControlMode::Qvbr)
                                    .qvbr_settings(
                                        H264QvbrSettings::builder()
                                            .qvbr_quality_level(7) // ~CRF 23 equivalent
                                            .build(),
                                    )
                                    .max_bitrate(max_bitrate)
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .audio_descriptions(
                AudioDescription::builder()
                    .audio_source_name("Audio Selector 1")
                    .codec_settings(
                        AudioCodecSettings::builder()
                            .codec(AudioCodec::Aac)
                            .aac_settings(
                                AacSettings::builder()
                                    .sample_rate(48000)
                                    .bitrate(128000_f64)
                                    .coding_mode(AacCodingMode::CodingMode20)
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .container_settings(
                ContainerSettings::builder()
                    .container(ContainerType::Mp4)
                    .build(),
            )
            .build()
    }

    let output_group = OutputGroup::builder()
        .output_group_settings(
            OutputGroupSettings::builder()
                .r#type(OutputGroupType::FileGroupSettings)
                .file_group_settings(
                    FileGroupSettings::builder()
                        .destination(&output_prefix)
                        .build(),
                )
                .build(),
        )
        .outputs(make_output("360p",  360,  1_500_000))
        .outputs(make_output("720p",  720,  4_000_000))
        .outputs(make_output("1080p", 1080, 8_000_000))
        .build();

    let settings = JobSettings::builder()
        .inputs(input)
        .output_groups(output_group)
        .build();

    // ── Submit job ────────────────────────────────────────────────────────────

    let job_id = cfg
        .mediaconvert
        .create_job()
        .role(&cfg.mediaconvert_role_arn)
        .settings(settings)
        .send()
        .await?
        .job()
        .and_then(|j| j.id())
        .ok_or_else(|| anyhow::anyhow!("MediaConvert returned no job ID"))?
        .to_string();

    tracing::info!(id, job_id, "MediaConvert job submitted");

    // ── Poll until complete ───────────────────────────────────────────────────

    loop {
        sleep(Duration::from_secs(15)).await;

        let job = cfg
            .mediaconvert
            .get_job()
            .id(&job_id)
            .send()
            .await?
            .job()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("job disappeared"))?;

        match job.status() {
            Some(aws_sdk_mediaconvert::types::JobStatus::Complete) => {
                tracing::info!(id, job_id, "MediaConvert job complete");
                break;
            }
            Some(aws_sdk_mediaconvert::types::JobStatus::Error) => {
                let msg = job.error_message().unwrap_or("unknown error");
                anyhow::bail!("MediaConvert error: {msg}");
            }
            Some(status) => {
                tracing::debug!(id, job_id, ?status, "job in progress");
            }
            None => {}
        }
    }

    // ── Discover which qualities were actually produced ────────────────────────
    // List objects under videos/{id}/ to find the mp4 files MediaConvert wrote.

    let listed = cfg
        .s3
        .list_objects_v2()
        .bucket(&cfg.video_bucket)
        .prefix(format!("videos/{}/", id))
        .send()
        .await?;

    let mut qualities: Vec<String> = listed
        .contents()
        .iter()
        .filter_map(|obj| {
            let key = obj.key()?;
            let filename = key.split('/').last()?;
            // MediaConvert produces e.g. "360p.mp4", "720p.mp4", "1080p.mp4"
            let quality = filename.strip_suffix(".mp4")?;
            Some(quality.to_owned())
        })
        .filter(|q| q != "status")
        .collect();

    // Sort by ascending resolution for the frontend quality picker.
    let order = ["360p", "720p", "1080p"];
    qualities.sort_by_key(|q| order.iter().position(|&o| o == q).unwrap_or(99));

    tracing::info!(id, ?qualities, "renditions available");

    // ── Clean up raw file ─────────────────────────────────────────────────────

    cfg.s3
        .delete_object()
        .bucket(&cfg.raw_bucket)
        .key(format!("uploads/{}.orig", id))
        .send()
        .await?;

    // ── Done ──────────────────────────────────────────────────────────────────

    write_status(cfg, id, json!({ "status": "done", "qualities": qualities })).await?;
    tracing::info!(id, "job complete");

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn write_status(cfg: &Config, id: &str, body: serde_json::Value) -> anyhow::Result<()> {
    let bytes = serde_json::to_vec(&body)?;
    cfg.s3
        .put_object()
        .bucket(&cfg.video_bucket)
        .key(format!("videos/{}/status.json", id))
        .content_type("application/json")
        .body(ByteStream::from(bytes))
        .send()
        .await?;
    Ok(())
}
