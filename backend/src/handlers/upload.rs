//! POST /api/upload: multipart file upload streamed to S3 (multipart upload API).
//! No full buffering in RAM; supports files up to 1GB.
//! Protected by Bearer token (ADMIN_TOKEN).

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use uuid::Uuid;

use crate::error::{AppError, UploadSuccess};
use crate::state::AppState;

/// Minimum S3 part size (5 MiB). Last part can be smaller.
const S3_PART_MIN_BYTES: usize = 5 * 1024 * 1024;

pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    while let Some(mut field) = multipart.next_field().await.map_err(|e| {
        AppError::S3(format!("multipart error: {}", e))
    })? {
        if !field.name().map(|n| n == "file").unwrap_or(false) {
            continue;
        }

        let id = Uuid::new_v4();
        let key = format!("uploads/{}.orig", id);

        let upload_id = state
            .s3
            .create_multipart_upload()
            .bucket(state.bucket())
            .key(&key)
            .send()
            .await
            .map_err(|e| AppError::S3(e.to_string()))?
            .upload_id()
            .ok_or_else(|| AppError::S3("missing upload_id".to_string()))?
            .to_string();

        let mut part_number: i32 = 1;
        let mut completed_parts: Vec<CompletedPart> = Vec::new();
        let mut buffer: Vec<u8> = Vec::with_capacity(S3_PART_MIN_BYTES);

        while let Some(chunk) = field.chunk().await.map_err(|e| {
            AppError::S3(format!("stream error: {}", e))
        })? {
            buffer.extend_from_slice(&chunk);
            while buffer.len() >= S3_PART_MIN_BYTES {
                let part_bytes: Vec<u8> = buffer.drain(..S3_PART_MIN_BYTES).collect();
                let body = ByteStream::from(part_bytes);
                let upload_part_res = state
                    .s3
                    .upload_part()
                    .bucket(state.bucket())
                    .key(&key)
                    .upload_id(&upload_id)
                    .part_number(part_number)
                    .body(body)
                    .send()
                    .await
                    .map_err(|e| AppError::S3(e.to_string()))?;
                let etag = upload_part_res.e_tag().ok_or_else(|| {
                    AppError::S3("upload_part missing etag".to_string())
                })?;
                completed_parts.push(
                    CompletedPart::builder()
                        .part_number(part_number)
                        .e_tag(etag)
                        .build(),
                );
                part_number += 1;
            }
        }

        if !buffer.is_empty() {
            let body = ByteStream::from(buffer);
            let upload_part_res = state
                .s3
                .upload_part()
                .bucket(state.bucket())
                .key(&key)
                .upload_id(&upload_id)
                .part_number(part_number)
                .body(body)
                .send()
                .await
                .map_err(|e| AppError::S3(e.to_string()))?;
            let etag = upload_part_res.e_tag().ok_or_else(|| {
                AppError::S3("upload_part missing etag".to_string())
            })?;
            completed_parts.push(
                CompletedPart::builder()
                    .part_number(part_number)
                    .e_tag(etag)
                    .build(),
            );
        }

        state
            .s3
            .complete_multipart_upload()
            .bucket(state.bucket())
            .key(&key)
            .upload_id(&upload_id)
            .multipart_upload(
                CompletedMultipartUpload::builder()
                    .set_parts(Some(completed_parts))
                    .build(),
            )
            .send()
            .await
            .map_err(|e| AppError::S3(e.to_string()))?;

        return Ok((
            StatusCode::CREATED,
            Json(UploadSuccess {
                id: id.to_string(),
                key,
            }),
        ));
    }

    Err(AppError::MissingFile)
}
