# AssetLab

Upload videos (up to 1 GB, e.g. MP4, WebM, MOV) and watch them by ID. The frontend provides an upload page and a watch page; the backend stores raw uploads in S3 and serves presigned URLs; a separate worker transcodes videos into multiple qualities (360p, 720p, 1080p) and writes them to a second S3 bucket.

## Architecture

### Functional requirements

- **Upload video** — User can upload a single video file (max 1 GB; formats such as MP4, WebM, MOV). Upload is protected by a Bearer token. The backend returns a video ID used for playback.
- **Watch video by ID** — User can open a watch page for a given ID. Playback uses presigned S3 URLs; no auth required for viewing.
- **Multi-quality playback** — After transcoding, the user can choose a quality (e.g. 360p, 720p, 1080p). The watch page polls transcode status and shows a quality selector when ready; until then (or on error), the raw file is played as fallback.
- **Transcode pipeline** — Uploaded files are queued for transcoding. A worker produces H.264/AAC renditions at fixed heights (360, 720, 1080, capped by source resolution) and stores them in the video bucket; status (pending → processing → done / error) is exposed via the API.

### Non-functional requirements

- **Upload efficiency** — No full buffering of the file in memory. The backend streams the request body to S3 using the multipart upload API (5 MiB parts), so large files are handled without loading them entirely into RAM.
- **Worker efficiency** — The worker does not download the raw file. It generates a presigned S3 URL and passes it to FFmpeg, which reads via HTTP Range requests; transcoded outputs are written to temp files then uploaded to S3 and removed.
- **Scalability** — API and worker are stateless. The worker claims jobs by deleting a queue marker in S3, allowing multiple instances to poll without a central queue service (at-most-once processing per marker).
- **Security** — Upload endpoint is protected by `ADMIN_TOKEN`. Playback uses short-lived presigned URLs; CORS is configurable via `CORS_ORIGINS`.

### High-level architecture

```
                    ┌─────────────────────────────────────────────────────────┐
                    │                      Frontend                            │
                    │  (Svelte 5 + Vite)  Upload page · Watch page (by ID)     │
                    └───────────────────────────┬─────────────────────────────┘
                                                │ /api
                                                ▼
                    ┌─────────────────────────────────────────────────────────┐
                    │                      Backend                            │
                    │  (Rust / Axum)  Upload · Presigned URLs · Status        │
                    └───────┬─────────────────────────────────┬───────────────┘
                            │                                 │
          S3 multipart      │ uploads/<id>.orig               │ GET status, GET
          upload +          │ queue/<id>                      │ presigned URLs
          write queue       │                                 │
                            ▼                                 ▼
                    ┌───────────────────┐           ┌───────────────────┐
                    │   S3 raw bucket   │           │  S3 video bucket   │
                    │  uploads/, queue/ │           │  videos/<id>/      │
                    └────────┬──────────┘           │  status.json       │
                             │                      │  <id>/360p.mp4 …   │
                             │ poll queue/          └────────▲───────────┘
                             │ claim by delete                │
                             │ presigned GET                  │ put status, put
                             │ (no download)                  │ renditions
                             ▼                                │
                    ┌─────────────────────────────────────────┴───────────┐
                    │                      Worker                           │
                    │  (Rust)  Poll S3 · FFmpeg via presigned URL · Upload │
                    └─────────────────────────────────────────────────────┘
```

- **Frontend** talks only to the **backend** (proxy or `VITE_API_URL`). It never touches S3 directly.
- **Backend** writes raw files and queue markers to the **raw bucket**, and initial/updated status (and optionally nothing else) to the **video bucket**. It serves presigned URLs for both raw and transcoded objects from the appropriate bucket.
- **Worker** polls the raw bucket’s `queue/` prefix, claims a job by deleting the marker, transcodes using a presigned URL (no local copy of the raw file), uploads renditions to the video bucket, deletes the raw object, and writes final `status.json` (e.g. `done` + qualities or `error`).

## Repo layout

- **[frontend/](frontend/README.md)** — Svelte + Vite app: upload page and watch-by-ID page with quality selector.
- **[backend/](backend/README.md)** — Rust/Axum API: upload (stream to S3), presigned video URLs, transcode status.
- **worker/** — Rust binary: polls S3 queue, runs FFmpeg, uploads renditions to the video bucket.

Run the backend on port 3000 and the frontend dev server (with `/api` proxied to the backend) for local use. Run the worker where it can reach the same S3 buckets (same AWS credentials and bucket names as the backend).
