# AssetLab

Upload videos (up to 1 GB, e.g. MP4, WebM, MOV) and watch them by ID. The frontend offers an upload page and a watch page; the backend stores files in a single S3 bucket and serves them via presigned URLs.

## Repo layout

Check those READMEs for more detail in the links !
- **[frontend/](frontend/README.md)** — Svelte + Vite app: upload page and watch-by-ID page.
- **[backend/](backend/README.md)** — Rust/Axum API: POST upload to S3, GET presigned video URL.

Run both (backend on port 3000, frontend dev server with `/api` proxied to the backend) to use the app locally.
