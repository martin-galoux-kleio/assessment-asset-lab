<script lang="ts">
  import { untrack } from 'svelte';
  import { Upload } from '@lucide/svelte';
  import ErrorMessage from '$lib/atoms/ErrorMessage.svelte';
  import { navigate } from '$lib/router';
  import { apiPath } from '$lib/constants/api';
  import {
    MAX_VIDEO_SIZE_BYTES,
    isVideoMimeType,
    isWithinSizeLimit
  } from '$lib/constants/video';

  /** Set VITE_ADMIN_TOKEN in .env (must match backend ADMIN_TOKEN) */
  const ADMIN_TOKEN = import.meta.env.VITE_ADMIN_TOKEN ?? '';

  let errorMessage = $state<string>('');
  let selectedFile = $state<File | null>(null);
  let previewUrl = $state<string>('');
  let isDragging = $state(false);
  let inputId = $state('video-upload-input');
  let uploading = $state(false);
  let uploadProgress = $state(0);
  let successId = $state<string | null>(null);
  let successKey = $state<string | null>(null);

  const MAX_SIZE_GB = 1;
  const acceptTypes = 'video/*';

  // Fixed height so empty, selected, uploading, and success states match
  const CARD_MIN_HEIGHT = '20rem';

  let lastCreatedUrl = '';
  $effect(() => {
    const f = selectedFile;
    if (!f) {
      if (lastCreatedUrl) {
        URL.revokeObjectURL(lastCreatedUrl);
        lastCreatedUrl = '';
      }
      untrack(() => (previewUrl = ''));
      return;
    }
    const url = URL.createObjectURL(f);
    if (lastCreatedUrl) URL.revokeObjectURL(lastCreatedUrl);
    lastCreatedUrl = url;
    untrack(() => (previewUrl = url));
    return () => {
      URL.revokeObjectURL(url);
      lastCreatedUrl = '';
    };
  });

  function getWatchUrl(id: string): string {
    if (typeof window === 'undefined') return `/watch/${id}`;
    return `${window.location.origin}/watch/${id}`;
  }

  async function copyWatchLink(): Promise<void> {
    if (!successId) return;
    const url = getWatchUrl(successId);
    try {
      await navigator.clipboard.writeText(url);
    } catch {
      errorMessage = 'Could not copy to clipboard.';
    }
  }

  function validateAndSetFile(file: File | null): void {
    errorMessage = '';
    selectedFile = null;
    successId = null;
    successKey = null;
    if (!file) return;

    if (!isVideoMimeType(file.type)) {
      errorMessage = 'Please select a video file (e.g. MP4, WebM, MOV).';
      return;
    }
    if (!isWithinSizeLimit(file.size)) {
      errorMessage = `File size must be ${MAX_SIZE_GB} GB or less. This file is ${(file.size / MAX_VIDEO_SIZE_BYTES).toFixed(2)} GB.`;
      return;
    }
    selectedFile = file;
  }

  function handleChange(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0] ?? null;
    validateAndSetFile(file);
    input.value = '';
  }

  function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragging = false;
    const file = event.dataTransfer?.files?.[0] ?? null;
    validateAndSetFile(file);
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy';
    isDragging = true;
  }

  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    isDragging = false;
  }

  function openFilePicker() {
    document.getElementById(inputId)?.click();
  }

  function submitUpload() {
    const file = selectedFile;
    if (!file || uploading) return;

    if (!ADMIN_TOKEN) {
      errorMessage =
        'Upload token not set. Add VITE_ADMIN_TOKEN to frontend .env (same value as backend ADMIN_TOKEN), then restart the dev server.';
      return;
    }

    errorMessage = '';
    successId = null;
    successKey = null;
    uploading = true;
    uploadProgress = 0;

    const formData = new FormData();
    formData.append('file', file);

    const xhr = new XMLHttpRequest();

    xhr.upload.addEventListener('progress', (e) => {
      if (e.lengthComputable) {
        uploadProgress = Math.round((e.loaded / e.total) * 100);
      }
    });

    xhr.addEventListener('load', () => {
      uploading = false;
      uploadProgress = 0;
      if (xhr.status === 201) {
        try {
          const json = JSON.parse(xhr.responseText) as { id: string; key: string };
          successId = json.id;
          successKey = json.key;
        } catch {
          errorMessage = 'Invalid response from server.';
        }
      } else if (xhr.status === 401) {
        errorMessage =
          'Unauthorized. Set VITE_ADMIN_TOKEN in frontend .env to match backend ADMIN_TOKEN, then restart the dev server.';
      } else if (xhr.status === 500) {
        try {
          const json = JSON.parse(xhr.responseText) as { error?: string };
          const msg = json.error ?? 'Server error.';
          errorMessage =
            msg.includes('auth not configured') || msg.includes('Server auth')
              ? 'Server auth not configured. Add ADMIN_TOKEN to backend/.env and start the backend from the backend/ directory (e.g. cd backend && cargo run).'
              : `Upload failed (500): ${msg}`;
        } catch {
          errorMessage =
            'Server error (500). Ensure backend has ADMIN_TOKEN in backend/.env and was started from backend/ (cd backend && cargo run).';
        }
      } else {
        try {
          const json = JSON.parse(xhr.responseText) as { error?: string };
          errorMessage = json.error ?? `Upload failed (${xhr.status}).`;
        } catch {
          errorMessage = `Upload failed (${xhr.status}).`;
        }
      }
    });

    xhr.addEventListener('error', () => {
      uploading = false;
      uploadProgress = 0;
      errorMessage = 'Network error. Is the backend running?';
    });

    xhr.addEventListener('abort', () => {
      uploading = false;
      uploadProgress = 0;
    });

    xhr.open('POST', apiPath('/api/upload'));
    xhr.setRequestHeader('Authorization', `Bearer ${ADMIN_TOKEN}`);
    xhr.send(formData);
  }
</script>

<div class="flex w-full md:w-[36rem] flex-col gap-4">
  <!-- Success state: same-size card -->
  {#if successId}
    <div
      class="flex w-full md:w-[36rem] flex-col items-center justify-center gap-4 overflow-hidden rounded-2xl border-2 border-solid border-green-500/40 bg-gradient-to-br from-emerald-950/95 to-emerald-900/98 py-8 dark:from-[#14231c] dark:to-[#0f1c16]"
      style="min-height: {CARD_MIN_HEIGHT}"
    >
      <p class="text-lg font-semibold text-green-700 dark:text-green-400">Your video is ready!</p>
      <p class="w-full max-w-md truncate px-4 text-center text-[0.9375rem] text-black/80 dark:text-white/80" title={getWatchUrl(successId)}>
        {getWatchUrl(successId)}
      </p>
      <div class="flex flex-wrap items-center justify-center gap-3">
        <button
          type="button"
          onclick={copyWatchLink}
          class="rounded-xl border border-black/20 bg-white/80 px-4 py-2 text-sm font-medium text-[#1a1a1a] shadow-sm transition hover:bg-white dark:border-white/20 dark:bg-white/10 dark:text-white/95 dark:hover:bg-white/20"
        >
          Copy link
        </button>
        <button
          type="button"
          onclick={() => navigate(`/watch/${successId}`)}
          class="rounded-xl bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm transition hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:focus:ring-offset-[#0f0f0f]"
        >
          Watch video
        </button>
      </div>
    </div>
  {:else if uploading}
    <!-- Uploading state: same-size card -->
    <div
      class="flex flex-col items-center justify-center gap-4 overflow-hidden rounded-2xl border-2 border-dashed border-indigo-500/50 bg-gradient-to-br from-indigo-100 to-indigo-50 py-8 dark:from-[#3c3c50] dark:to-[#2d2d41]"
      style="min-height: {CARD_MIN_HEIGHT}"
      role="status"
      aria-live="polite"
    >
      <p class="text-lg font-semibold text-indigo-700 dark:text-indigo-300">Uploading your video</p>
      <div class="w-full max-w-xs space-y-2">
        <div class="h-2 w-full overflow-hidden rounded-full bg-black/10 dark:bg-white/10">
          <div
            class="h-full rounded-full bg-indigo-500 transition-[width] duration-200"
            style="width: {uploadProgress}%"
          ></div>
        </div>
        <p class="text-center text-[0.8125rem] text-black/60 dark:text-white/60">{uploadProgress}%</p>
      </div>
    </div>
  {:else}
    <!-- Drop zone: empty or file selected (with thumbnail) -->
    <div
      class="relative flex min-h-[20rem] cursor-pointer flex-col items-center justify-center overflow-hidden rounded-2xl border-2 border-dashed py-8 transition-[border-color,background,box-shadow,transform] duration-200 {selectedFile
        ? 'border-solid border-green-500/40 bg-gradient-to-br from-emerald-950/95 to-emerald-900/98 hover:border-green-500/60 dark:border-green-500/40 dark:from-[#14231c] dark:to-[#0f1c16] dark:hover:border-green-500/60'
        : isDragging
          ? 'scale-[1.01] border-indigo-500 bg-gradient-to-br from-indigo-100 to-indigo-50 shadow-[0_0_0_3px_rgba(100,108,255,0.2)] dark:from-[#3c3c50] dark:to-[#2d2d41]'
          : 'border-black/12 bg-gradient-to-br from-[#fafafa] to-[#f0f0f0] shadow-[0_4px_6px_-1px_rgba(0,0,0,0.06),0_10px_20px_-10px_rgba(0,0,0,0.08),inset_0_1px_0_rgba(255,255,255,0.8)] hover:border-indigo-500/40 hover:from-[#f5f5f5] hover:to-[#ebebeb] dark:border-white/20 dark:from-[#1e1e23] dark:to-[#16161a] dark:shadow-[0_4px_6px_-1px_rgba(0,0,0,0.3),0_10px_20px_-10px_rgba(0,0,0,0.4),inset_0_1px_0_rgba(255,255,255,0.04)] dark:hover:border-indigo-500/50 dark:hover:from-[#23232a] dark:hover:to-[#1a1a20] dark:hover:shadow-[0_8px_16px_-4px_rgba(0,0,0,0.35),0_20px_40px_-15px_rgba(0,0,0,0.45),inset_0_1px_0_rgba(255,255,255,0.06)]'}"
      role="button"
      tabindex="0"
      onclick={openFilePicker}
      onkeydown={(e) => e.key === 'Enter' && openFilePicker()}
      ondragover={handleDragOver}
      ondragleave={handleDragLeave}
      ondrop={handleDrop}
      aria-label="Drop video file or click to browse"
    >
      <input
        id={inputId}
        type="file"
        accept={acceptTypes}
        onchange={handleChange}
        class="pointer-events-none absolute h-[0.1px] w-[0.1px] opacity-0"
        aria-hidden="true"
        tabindex="-1"
      />
      {#if selectedFile && previewUrl}
        <div class="flex flex-col items-center justify-center gap-3 p-4 text-center">
          <video
            src={previewUrl}
            muted
            class="max-h-[10rem] w-auto max-w-full rounded-lg object-contain shadow-md"
            preload="metadata"
            aria-hidden="true"
          ></video>
          <span class="max-w-full truncate text-[0.9375rem] font-medium text-[#1a1a1a] dark:text-white/95">
            {selectedFile.name}
          </span>
          <span class="text-[0.8125rem] text-green-700 dark:text-green-400">
            {(selectedFile.size / 1024 / 1024).toFixed(2)} MB
          </span>
        </div>
      {:else}
        <div class="flex flex-col items-center justify-center gap-3 p-8 text-center">
          <span class="inline-flex text-indigo-600 transition-colors hover:text-indigo-400 dark:text-indigo-500 dark:hover:text-indigo-400">
            <Upload size={28} strokeWidth={2} />
          </span>
          <span class="text-base font-medium text-[#1a1a1a] dark:text-white/90">
            Pick a video
          </span>
          <span class="text-[0.8125rem] text-black/45 dark:text-white/45">
            Drop your video here or click to browse · MP4, WebM, MOV — max 1 GB
          </span>
        </div>
      {/if}
    </div>

    {#if selectedFile}
      <button
        type="button"
        onclick={submitUpload}
        class="rounded-xl bg-indigo-600 px-5 py-2.5 text-sm font-medium text-white shadow-sm transition hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:focus:ring-offset-[#0f0f0f]"
      >
        Start Uploading 
      </button>
    {/if}
  {/if}

  <div id="upload-error" aria-live="polite">
    <ErrorMessage message={errorMessage} />
  </div>
</div>
