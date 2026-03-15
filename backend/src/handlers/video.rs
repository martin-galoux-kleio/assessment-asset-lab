//! Video handlers:
//!   GET /api/video/:id          — presigned URL for raw upload (S3_RAW_BUCKET, fallback)
//!   GET /api/video/:id/status   — transcoding status JSON (S3_VIDEO_BUCKET)
//!   GET /api/video/:id/:quality — presigned URL for a transcoded rendition (S3_VIDEO_BUCKET)

use axum::{
    extract::{Path, State},
    response::Response,
    http::header,
    body::Body,
    Json,
};
use aws_sdk_s3::presigning::PresigningConfig;
use serde::Serialize;
use std::time::Duration;

use crate::error::AppError;
use crate::state::AppState;

const PRESIGN_TTL_SECS: u64 = 3600; // 1 hour

#[derive(Serialize)]
pub struct VideoUrlResponse {
    pub url: String,
}

/// GET /api/video/:id
/// Returns a presigned URL for the raw upload in S3_RAW_BUCKET.
/// Used as an immediate fallback while transcoding is in progress.
pub async fn video_url(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<VideoUrlResponse>, AppError> {
    let key = format!("uploads/{}.orig", id);

    let presigning_config = PresigningConfig::expires_in(Duration::from_secs(PRESIGN_TTL_SECS))
        .map_err(|e| AppError::S3(e.to_string()))?;

    let presigned = state
        .s3
        .get_object()
        .bucket(state.raw_bucket())
        .key(&key)
        .presigned(presigning_config)
        .await
        .map_err(|e| AppError::S3(e.to_string()))?;

    Ok(Json(VideoUrlResponse {
        url: presigned.uri().to_string(),
    }))
}

/// GET /api/video/:id/status
/// Fetches videos/{id}/status.json from S3_VIDEO_BUCKET and returns it verbatim.
/// Returns 404 if the object does not exist (video not yet queued or unknown id).
pub async fn video_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response<Body>, AppError> {
    let key = format!("videos/{}/status.json", id);

    let result = state
        .s3
        .get_object()
        .bucket(state.video_bucket())
        .key(&key)
        .send()
        .await;

    match result {
        Ok(output) => {
            let bytes = output
                .body
                .collect()
                .await
                .map_err(|e| AppError::S3(e.to_string()))?
                .into_bytes();

            let response = Response::builder()
                .status(200)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(bytes))
                .map_err(|e| AppError::S3(e.to_string()))?;

            Ok(response)
        }
        Err(e) => {
            // NoSuchKey means the video hasn't been queued yet or the id is unknown.
            let is_not_found = e
                .as_service_error()
                .map(|se| se.is_no_such_key())
                .unwrap_or(false);

            if is_not_found {
                Err(AppError::NotFound)
            } else {
                Err(AppError::S3(e.to_string()))
            }
        }
    }
}

/// GET /api/video/:id/:quality
/// Returns a CloudFront URL for a transcoded rendition (e.g. 360p, 720p, 1080p).
/// No presigning needed — the bucket is private but CloudFront OAC handles access.
/// UUIDs are unguessable so the URL itself acts as the access token.
pub async fn video_quality_url(
    State(state): State<AppState>,
    Path((id, quality)): Path<(String, String)>,
) -> Result<Json<VideoUrlResponse>, AppError> {
    let url = format!(
        "https://{}/videos/{}/{}.mp4",
        state.cloudfront_domain(),
        id,
        quality,
    );

    Ok(Json(VideoUrlResponse { url }))
}
