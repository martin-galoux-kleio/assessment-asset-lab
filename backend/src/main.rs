use axum::{
    middleware,
    routing::post,
    Router,
};
use axum::extract::DefaultBodyLimit;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod error;
mod handlers;
mod state;

use handlers::upload;
use state::AppState;

/// Max upload body size: 1 GB (for multipart/form-data).
const MAX_UPLOAD_BYTES: usize = 1024 * 1024 * 1024;

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
    let bucket = std::env::var("S3_RAW_BUCKET")
        .expect("S3_RAW_BUCKET env var must be set");

    let state = AppState {
        s3: s3_client,
        bucket,
    };

    let app = Router::new()
        .route("/api/upload", post(upload))
        .route_layer(middleware::from_fn(auth::require_admin_bearer))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
        .with_state(state);

    let listen = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", listen);
    let listener = tokio::net::TcpListener::bind(listen).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
