use axum::{
    extract::Request,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use axum::middleware::Next;

/// Middleware: rejects the request if Authorization header is not Bearer <ADMIN_TOKEN>.
pub async fn require_admin_bearer(req: Request, next: Next) -> Response {
    let admin_token = std::env::var("ADMIN_TOKEN").unwrap_or_else(|_| String::new());
    if admin_token.is_empty() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": "Server auth not configured" })),
        )
            .into_response();
    }
    let auth = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !auth.starts_with("Bearer ") {
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({ "error": "Missing or invalid Authorization header" })),
        )
            .into_response();
    }
    let token = auth["Bearer ".len()..].trim();
    if token != admin_token {
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({ "error": "Invalid token" })),
        )
            .into_response();
    }
    next.run(req).await
}
