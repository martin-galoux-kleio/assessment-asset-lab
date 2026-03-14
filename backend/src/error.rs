use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    MissingFile,
    NotFound,
    S3(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::MissingFile => (StatusCode::BAD_REQUEST, "Missing file field".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            AppError::S3(msg) => (StatusCode::BAD_GATEWAY, format!("S3 upload failed: {}", msg)),
        };
        (
            status,
            Json(serde_json::json!({ "error": message })),
        )
            .into_response()
    }
}

#[derive(Serialize)]
pub struct UploadSuccess {
    pub id: String,
    pub key: String,
}
