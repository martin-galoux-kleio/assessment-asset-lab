# Backend

Rust/Axum API. Video uploads and playback use a **single S3 bucket** (for now).

## API

- **POST `/api/upload`** — Upload a video to S3.  
  Multipart form field `file`. Streamed to S3 via multipart upload (no full buffering; supports files up to 1 GB).  
  Protected by Bearer token (`ADMIN_TOKEN`).  
  Returns the generated video **ID** (and key); use it to watch the video.

- **GET `/api/video/:id`** — Fetch a video by ID.  
  Returns a short-lived presigned S3 URL so the client can play the file. No auth required for this endpoint.

All video objects are stored in one bucket under keys like `uploads/<id>.orig`.

## Environment variables

- **AWS_ACCESS_KEY_ID**, **AWS_SECRET_ACCESS_KEY** – AWS credentials
- **AWS_DEFAULT_REGION** – e.g. `eu-west-3`
- **S3_RAW_BUCKET** – single S3 bucket name for uploads and video URLs (required)
- **ADMIN_TOKEN** – Bearer token required for `/api/upload`

## Run

```bash
cd backend && cargo run
```

Server listens on `http://0.0.0.0:3000`. Use the frontend dev server (with proxy `/api` → backend) to upload.

## Deploy (API)

The backend is a single binary listening on port 3000. Set the same [environment variables](#environment-variables) in your host; the app does not read `.env` in production (it uses the platform’s env).

### Option A: Fly.io

1. Install [flyctl](https://fly.io/docs/hands-on/install-flyctl/) and log in: `fly auth login`.
2. From the **backend** directory: `fly launch` (choose app name, region; when asked for a Dockerfile, use the one in `backend/`).
3. Set secrets (env vars):  
   `fly secrets set AWS_ACCESS_KEY_ID=... AWS_SECRET_ACCESS_KEY=... AWS_DEFAULT_REGION=eu-west-3 S3_RAW_BUCKET=your-bucket ADMIN_TOKEN=...`
4. Deploy: `fly deploy`.  
   Your API will be at `https://<your-app>.fly.dev`. Point the frontend’s `/api` (or `VITE_API_URL`) to that URL.

### Deploy on Railway (step by step)

1. **Sign in**  
   Go to [railway.app](https://railway.app) and sign in (e.g. with GitHub).

2. **New project from repo**  
   Click **New Project** → **Deploy from GitHub repo**. Select your **AssetLab** repo (grant Railway access if asked). Railway will create a project and try to deploy; we’ll point it at the backend next.

3. **Use the backend folder**  
   In the new service, open **Settings**. Under **Source**, set **Root Directory** to `backend` and save. Railway will re-detect the app from that folder.

4. **Use the Dockerfile**  
   In the same service, under **Settings → Build**, ensure **Builder** is **Dockerfile** (Railway should pick up `backend/Dockerfile` automatically when root is `backend`). You don’t need to set a custom build or start command.

5. **Add variables**  
   Open the **Variables** tab. Add these (same names as [above](#environment-variables)); replace values with your real ones:
   - `AWS_ACCESS_KEY_ID`
   - `AWS_SECRET_ACCESS_KEY`
   - `AWS_DEFAULT_REGION` (e.g. `eu-west-3`)
   - `S3_RAW_BUCKET` (your S3 bucket name)
   - `ADMIN_TOKEN` (same token your frontend uses for uploads)

6. **Generate a public URL**  
   In **Settings**, open **Networking** → **Generate Domain**. Railway will assign a URL like `https://assetlab-backend-production-xxxx.up.railway.app`.

7. **Deploy**  
   Push a commit or click **Redeploy** in the **Deployments** tab. Wait for the build (Docker build then start). The app listens on Railway’s `PORT` automatically.

8. **Check the API**  
   Open `https://<your-domain>/api/video/some-id` in a browser (you’ll get an error if the id doesn’t exist; that’s fine — it means the service is up). Use this base URL in your Vercel frontend (rewrite `/api` to this URL or set `VITE_API_URL`); see [DEPLOY_VERCEL.md](../DEPLOY_VERCEL.md).

### Docker (any platform)

From the **backend** directory:

```bash
docker build -t assetlab-backend .
docker run -p 3000:3000 -e AWS_ACCESS_KEY_ID=... -e AWS_SECRET_ACCESS_KEY=... -e AWS_DEFAULT_REGION=... -e S3_RAW_BUCKET=... -e ADMIN_TOKEN=... assetlab-backend
```

Use the same env vars on ECS, Cloud Run, or any host that runs Docker.

After the API is deployed, configure the [Vercel frontend](https://vercel.com) to call it (e.g. rewrite `/api` to your backend URL or set `VITE_API_URL`); see [DEPLOY_VERCEL.md](../DEPLOY_VERCEL.md).

---

## AWS CLI setup

The bucket script requires the [AWS CLI](https://aws.amazon.com/cli/). Install it with one of these:

### Option 1: Homebrew (recommended)

```bash
brew install awscli
```

If you get permission errors, fix Homebrew ownership first:

```bash
sudo chown -R $(whoami) /opt/homebrew /opt/homebrew/Cellar /opt/homebrew/Frameworks /opt/homebrew/bin /opt/homebrew/etc /opt/homebrew/include /opt/homebrew/lib /opt/homebrew/opt /opt/homebrew/sbin /opt/homebrew/share /opt/homebrew/var
```

Then run `brew install awscli` again.

### Option 2: Official installer (macOS)

```bash
curl "https://awscli.amazonaws.com/AWSCLIV2.pkg" -o "AWSCLIV2.pkg"
sudo installer -pkg AWSCLIV2.pkg -target /
rm AWSCLIV2.pkg
```

### Option 3: pip (user install, no sudo)

```bash
pip3 install awscli --user
```

Ensure `~/.local/bin` (or your Python user bin) is in your `PATH`.

---

After installing, configure credentials (if not already):

```bash
aws configure
```

Then create the buckets:

```bash
./backend/scripts/create-buckets.sh
```
