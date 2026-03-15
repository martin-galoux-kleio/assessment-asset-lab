# AssetLab — Frontend

Svelte 5 + TypeScript + Vite app for uploading and watching videos.

## Pages

1. **Upload** (`/upload`)  
   Upload a video file:
   - **Max size:** 1 GB  
   - **Formats:** MP4, WebM, MOV (and other `video/*` MIME types)  
   After a successful upload, the backend returns a video **ID**. Use **Watch** with that ID to view the video.

2. **Watch** (`/watch/:id`)  
   Watch a video by ID. The page:
   - Fetches transcode **status** from the backend (`/api/video/:id/status`).
   - When transcoding is **done**, offers a **quality selector** (e.g. 720p, 1080p) and uses presigned URLs for each quality (`/api/video/:id/:quality`).
   - **Fallback:** If the video has no transcode data or transcoding failed, the raw file is played via `/api/video/:id`.

## Tech

- [Svelte](https://svelte.dev/) 5 + [TypeScript](https://www.typescriptlang.org/) + [Vite](https://vitejs.dev/)
- [Tailwind CSS](https://tailwindcss.com/) v4
- [Lucide](https://lucide.dev/) icons

## Run Locally

```bash
pnpm install
pnpm run dev
```

The dev server proxies `/api` to `http://localhost:3000` (see `vite.config.ts`). Run the [backend](../backend/README.md) on port 3000 for full local use.

## Production Hosting

The frontend is deployed on [Vercel](https://vercel.com) 
On [this URL](https://streamvault-ui.vercel.app/)
