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
