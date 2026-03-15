# AssetLab — Backend

Rust/Axum API for video uploads and playback. Uses two S3 buckets: one for raw uploads, one for transcoded outputs.

## API

| Method | Path | Description |
|--------|------|-------------|
| **POST** | `/api/upload` | Upload a video to S3. Multipart form field `file`. Streamed via S3 multipart upload (no full buffering; supports files up to 1 GB). **Auth:** Bearer token (`ADMIN_TOKEN`). Returns the video **ID** (and key). |
| **GET** | `/api/video/:id` | Raw video by ID. Returns a short-lived presigned S3 URL for the original file in the raw bucket. No auth. |
| **GET** | `/api/video/:id/status` | Transcode status for a video. Returns `{ status, qualities?, error? }` (`pending` \| `processing` \| `done` \| `error`). 404 if the video has no transcode metadata. No auth. |
| **GET** | `/api/video/:id/:quality` | Presigned URL for a specific transcoded quality (e.g. `720p`, `1080p`). No auth. |

Raw files are stored in the raw bucket under keys like `uploads/<id>.orig`. Transcoded outputs live in the video bucket; the worker (see repo root) is responsible for transcoding and writing there.

## Environment variables

| Variable | Required | Description |
|----------|----------|-------------|
| **AWS_ACCESS_KEY_ID** | Yes | AWS credentials |
| **AWS_SECRET_ACCESS_KEY** | Yes | AWS credentials |
| **AWS_DEFAULT_REGION** | No | Default `eu-west-3` |
| **S3_RAW_BUCKET** | Yes | S3 bucket for raw uploads |
| **S3_VIDEO_BUCKET** | Yes | S3 bucket for transcoded outputs |
| **ADMIN_TOKEN** | Yes | Bearer token for `/api/upload` |
| **CORS_ORIGINS** | No | Comma-separated allowed origins. If unset, all origins are allowed. |
| **PORT** | No | Listen port (default `3000`) |
| **RUST_LOG** | No | Log level (default `info`) |

`.env` in the backend directory is loaded in development (via `dotenvy`). In production, use the platform’s environment (no `.env` file).

## Run Locally 

```bash
cd backend
cargo run
```

Server listens on `http://0.0.0.0:3000`. Use the frontend dev server with proxy `/api` → backend for local development.

### Deployed on [railway.app](https://railway.app)
Two instances running, /worker and /backend
Dockerfile to containerize service