# AssetLab — Frontend

Svelte + TypeScript + Vite app with two pages.

## Pages

1. **Upload** (`/upload`)  
   User can upload a video file:
   - **Max size:** 1 GB  
   - **Formats:** MP4, WebM, MOV (and other `video/*` MIME types)  
   After a successful upload, the backend returns a video **ID**; the user can go to `/watch/<id>` to watch it.

2. **Watch** (`/watch/:id`)  
   User can watch an uploaded video by ID. The page fetches a short-lived presigned URL from the backend and plays the video in a `<video>` player.

## Tech

- [Svelte](https://svelte.dev/) + [TypeScript](https://www.typescriptlang.org/) + [Vite](https://vitejs.dev/)

## Run

```bash
npm install
npm run dev
```

Use the dev server with the Vite proxy so `/api` requests go to the backend (see `vite.config.ts`).

## Hosting

The frontend is deployed on [Vercel](https://vercel.com) (root directory: `frontend`, Vite build). See [DEPLOY_VERCEL.md](../DEPLOY_VERCEL.md) in the repo root for step-by-step deployment and how to point `/api` at the backend.

## IDE

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) is recommended.
