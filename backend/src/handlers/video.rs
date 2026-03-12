//! GET /api/video/:id — returns a short-lived presigned S3 URL for the raw upload.

use axum::{
    extract::{Path, State},
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
        .bucket(state.bucket())
        .key(&key)
        .presigned(presigning_config)
        .await
        .map_err(|e| AppError::S3(e.to_string()))?;

    Ok(Json(VideoUrlResponse {
        url: presigned.uri().to_string(),
    }))
}
