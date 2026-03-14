use axum::{
    extract::DefaultBodyLimit,
    http::{header::ACCEPT, header::AUTHORIZATION, header::CONTENT_TYPE, HeaderValue, Method},
    middleware,
    routing::{get, post},
    Router,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::cors::{Any, CorsLayer};

mod auth;
mod error;
mod handlers;
mod state;

use handlers::{upload, video_url, video_status, video_quality_url};
use state::AppState;

/// Max upload body size: 1 GB (for multipart/form-data).
const MAX_UPLOAD_BYTES: usize = 1024 * 1024 * 1024;

fn cors_layer() -> CorsLayer {
    let origins = std::env::var("CORS_ORIGINS")
        .ok()
        .map(|s| {
            s.split(',')
                .map(|o| o.trim())
                .filter(|o| !o.is_empty())
                .filter_map(|o| o.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>()
        });
    let layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT]);
    match origins {
        Some(ref o) if !o.is_empty() => layer.allow_origin(o.clone()),
        _ => layer.allow_origin(Any),
    }
}

#[tokio::main]
async fn main() {
    // Load .env from backend directory so AWS_* and ADMIN_TOKEN are set
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let region = aws_types::region::Region::new(
        std::env::var("AWS_DEFAULT_REGION").unwrap_or_else(|_| "eu-west-3".to_string()),
    );
    let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region)
        .load()
        .await;
    let s3_client = aws_sdk_s3::Client::new(&aws_config);
    let raw_bucket = std::env::var("S3_RAW_BUCKET")
        .expect("S3_RAW_BUCKET env var must be set");
    let video_bucket = std::env::var("S3_VIDEO_BUCKET")
        .expect("S3_VIDEO_BUCKET env var must be set");

    let state = AppState {
        s3: s3_client,
        raw_bucket,
        video_bucket,
    };

    let protected = Router::new()
        .route("/api/upload", post(upload))
        .route_layer(middleware::from_fn(auth::require_admin_bearer))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES));

    let app = Router::new()
        // Static segment "/status" must be defined before "/:quality" so Axum
        // matches the literal before the wildcard param.
        .route("/api/video/:id/status", get(video_status))
        .route("/api/video/:id/:quality", get(video_quality_url))
        .route("/api/video/:id", get(video_url))
        .merge(protected)
        .layer(cors_layer())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let listen = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", listen);
    let listener = tokio::net::TcpListener::bind(listen).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
