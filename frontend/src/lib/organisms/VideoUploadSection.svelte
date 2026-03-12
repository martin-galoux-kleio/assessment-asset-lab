<script lang="ts">
  import { Upload, FileCheck } from '@lucide/svelte';
  import ErrorMessage from '$lib/atoms/ErrorMessage.svelte';
  import {
    MAX_VIDEO_SIZE_BYTES,
    isVideoMimeType,
    isWithinSizeLimit
  } from '$lib/constants/video';

  let errorMessage = $state<string>('');
  let selectedFile = $state<File | null>(null);
  let isDragging = $state(false);
  let inputId = $state('video-upload-input');

  const MAX_SIZE_GB = 1;
  const acceptTypes = 'video/*';

  function validateAndSetFile(file: File | null): void {
    errorMessage = '';
    selectedFile = null;
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
</script>

<div class="flex w-full max-w-[36rem] flex-col gap-4">
  <div
    class="relative flex min-h-[14rem] cursor-pointer items-center justify-center overflow-hidden rounded-2xl border-2 border-dashed py-8 transition-[border-color,background,box-shadow,transform] duration-200 {selectedFile
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
    {#if selectedFile}
      <div class="flex flex-col items-center justify-center gap-3 p-8 text-center">
        <span class="inline-flex text-green-600 dark:text-green-500">
          <FileCheck size={48} strokeWidth={1.5} />
        </span>
        <span class="max-w-full truncate text-[0.9375rem] font-medium text-[#1a1a1a] dark:text-white/95">
          {selectedFile.name}
        </span>
        <span class="text-[0.8125rem] text-green-700 dark:text-green-400">
          {(selectedFile.size / 1024 / 1024).toFixed(2)} MB
        </span>
      </div>
    {:else}
      <div class="flex flex-col items-center justify-center gap-3 p-8 text-center">
        <div class="flex items-center justify-center gap-5">
          <span class="inline-flex text-indigo-600 transition-colors hover:text-indigo-400 dark:text-indigo-500 dark:hover:text-indigo-400">
            <Upload size={28} strokeWidth={2} />
          </span>
        </div>
        <span class="text-base font-medium text-[#1a1a1a] dark:text-white/90">
          Drop your video here or click to browse
        </span>
        <span class="text-[0.8125rem] text-black/45 dark:text-white/45">
          MP4, WebM, MOV — max 1 GB
        </span>
      </div>
    {/if}
  </div>

  <div id="upload-error" aria-live="polite">
    <ErrorMessage message={errorMessage} />
  </div>
</div>
