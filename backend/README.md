# Backend

Rust/Axum API. Upload endpoint: `POST /api/upload` (multipart form field `file`), streamed to S3 bucket via multipart upload.

## Environment variables

- **AWS_ACCESS_KEY_ID**, **AWS_SECRET_ACCESS_KEY** – AWS credentials
- **AWS_DEFAULT_REGION** – e.g. `eu-west-3`
- **S3_RAW_BUCKET** – bucket name (default: `streamvault-raw`)
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
