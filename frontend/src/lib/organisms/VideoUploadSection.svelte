<script lang="ts">
  import { Upload, Video, FileCheck } from '@lucide/svelte';
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

<div class="upload-section">
  <div
    class="drop-zone"
    class:dragging={isDragging}
    class:has-file={!!selectedFile}
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
      class="input"
      aria-hidden="true"
      tabindex="-1"
    />
    {#if selectedFile}
      <div class="state has-file">
        <span class="icon icon-success">
          <FileCheck size={48} strokeWidth={1.5} />
        </span>
        <span class="filename">{selectedFile.name}</span>
        <span class="filesize">{(selectedFile.size / 1024 / 1024).toFixed(2)} MB</span>
      </div>
    {:else}
      <div class="state empty">
        <div class="icon-wrap">
          <span class="icon icon-video">
            <Video size={40} strokeWidth={1.5} />
          </span>
          <span class="icon icon-upload">
            <Upload size={28} strokeWidth={2} />
          </span>
        </div>
        <span class="prompt">Drop your video here or click to browse</span>
        <span class="hint">MP4, WebM, MOV — max 1 GB</span>
      </div>
    {/if}
  </div>
  <div id="upload-error" aria-live="polite">
    <ErrorMessage message={errorMessage} />
  </div>
</div>

<style>
  .upload-section {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: 1rem;
    width: 100%;
    max-width: 36rem;
  }

  .drop-zone {
    position: relative;
    min-height: 14rem;
    border-radius: 16px;
    border: 2px dashed rgba(255, 255, 255, 0.2);
    background: linear-gradient(145deg, rgba(30, 30, 35, 0.95) 0%, rgba(22, 22, 26, 0.98) 100%);
    box-shadow:
      0 4px 6px -1px rgba(0, 0, 0, 0.3),
      0 10px 20px -10px rgba(0, 0, 0, 0.4),
      inset 0 1px 0 rgba(255, 255, 255, 0.04);
    cursor: pointer;
    transition:
      border-color 0.2s,
      background 0.2s,
      box-shadow 0.2s,
      transform 0.15s;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .drop-zone:hover {
    border-color: rgba(100, 108, 255, 0.5);
    background: linear-gradient(145deg, rgba(35, 35, 42, 0.98) 0%, rgba(26, 26, 32, 0.99) 100%);
    box-shadow:
      0 8px 16px -4px rgba(0, 0, 0, 0.35),
      0 20px 40px -15px rgba(0, 0, 0, 0.45),
      inset 0 1px 0 rgba(255, 255, 255, 0.06);
  }

  .drop-zone.dragging {
    border-color: #646cff;
    background: linear-gradient(145deg, rgba(60, 60, 80, 0.98) 0%, rgba(45, 45, 65, 0.99) 100%);
    box-shadow:
      0 0 0 3px rgba(100, 108, 255, 0.2),
      0 12px 24px -8px rgba(0, 0, 0, 0.4);
    transform: scale(1.01);
  }

  .drop-zone.has-file {
    border-style: solid;
    border-color: rgba(34, 197, 94, 0.4);
    background: linear-gradient(145deg, rgba(20, 35, 28, 0.95) 0%, rgba(15, 28, 22, 0.98) 100%);
  }

  .drop-zone.has-file:hover {
    border-color: rgba(34, 197, 94, 0.6);
  }

  .input {
    position: absolute;
    width: 0.1px;
    height: 0.1px;
    opacity: 0;
    pointer-events: none;
  }

  .state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 2rem;
    text-align: center;
  }

  .icon-wrap {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1.25rem;
  }

  .icon {
    display: inline-flex;
    transition: color 0.2s;
  }
  .icon :global(svg) {
    color: inherit;
  }

  .icon-video {
    color: rgba(255, 255, 255, 0.5);
  }

  .icon-upload {
    color: #646cff;
  }

  .drop-zone:hover .icon-video {
    color: rgba(100, 108, 255, 0.8);
  }

  .drop-zone:hover .icon-upload {
    color: #818cf8;
  }

  .icon-success {
    color: #22c55e;
  }

  .prompt {
    font-size: 1rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.9);
  }

  .hint {
    font-size: 0.8125rem;
    color: rgba(255, 255, 255, 0.45);
  }

  .filename {
    font-size: 0.9375rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.95);
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .filesize {
    font-size: 0.8125rem;
    color: rgba(34, 197, 94, 0.9);
  }

  :global([data-theme='light']) .drop-zone,
  :global(.light) .drop-zone {
    border-color: rgba(0, 0, 0, 0.12);
    background: linear-gradient(145deg, #fafafa 0%, #f0f0f0 100%);
    box-shadow:
      0 4px 6px -1px rgba(0, 0, 0, 0.06),
      0 10px 20px -10px rgba(0, 0, 0, 0.08),
      inset 0 1px 0 rgba(255, 255, 255, 0.8);
  }

  :global([data-theme='light']) .drop-zone:hover,
  :global(.light) .drop-zone:hover {
    border-color: rgba(100, 108, 255, 0.4);
    background: linear-gradient(145deg, #f5f5f5 0%, #ebebeb 100%);
  }

  :global([data-theme='light']) .drop-zone.dragging,
  :global(.light) .drop-zone.dragging {
    border-color: #646cff;
    background: linear-gradient(145deg, #eef 0%, #e8e8f5 100%);
  }

  :global([data-theme='light']) .icon-video,
  :global(.light) .icon-video {
    color: rgba(0, 0, 0, 0.4);
  }

  :global([data-theme='light']) .icon-upload,
  :global(.light) .icon-upload {
    color: #6366f1;
  }

  :global([data-theme='light']) .drop-zone:hover .icon-video,
  :global(.light) .drop-zone:hover .icon-video {
    color: rgba(100, 108, 255, 0.7);
  }

  :global([data-theme='light']) .drop-zone:hover .icon-upload,
  :global(.light) .drop-zone:hover .icon-upload {
    color: #4f46e5;
  }

  :global([data-theme='light']) .icon-success,
  :global(.light) .icon-success {
    color: #16a34a;
  }

  :global([data-theme='light']) .prompt,
  :global(.light) .prompt {
    color: #1a1a1a;
  }

  :global([data-theme='light']) .hint,
  :global(.light) .hint {
    color: rgba(0, 0, 0, 0.45);
  }

  :global([data-theme='light']) .drop-zone.has-file,
  :global(.light) .drop-zone.has-file {
    border-color: rgba(34, 197, 94, 0.5);
    background: linear-gradient(145deg, #f0fdf4 0%, #dcfce7 100%);
  }

  :global([data-theme='light']) .filesize,
  :global(.light) .filesize {
    color: #16a34a;
  }
</style>
