<script lang="ts">
  import { Film } from '@lucide/svelte';
  import VideoUploadSection from '$lib/organisms/VideoUploadSection.svelte';
</script>

<main class="page">
  <header class="header">
    <div class="logo">
      <Film size={28} strokeWidth={1.75} aria-hidden="true" />
    </div>
    <h1>Upload video</h1>
    <p class="lead">Drop a file or click to choose. Max size 1 GB.</p>
  </header>
  <VideoUploadSection />
</main>

<style>
  .page {
    padding: 2.5rem 1.5rem;
    max-width: 42rem;
    margin: 0 auto;
    text-align: left;
  }

  .header {
    margin-bottom: 2rem;
  }

  .logo {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 3rem;
    height: 3rem;
    border-radius: 12px;
    background: linear-gradient(145deg, rgba(100, 108, 255, 0.2) 0%, rgba(100, 108, 255, 0.08) 100%);
    color: #818cf8;
    margin-bottom: 1.25rem;
  }

  h1 {
    margin: 0 0 0.5rem;
    font-size: 1.75rem;
    font-weight: 600;
    letter-spacing: -0.02em;
    color: rgba(255, 255, 255, 0.95);
  }

  .lead {
    margin: 0;
    font-size: 0.9375rem;
    color: rgba(255, 255, 255, 0.55);
    line-height: 1.5;
  }

  :global([data-theme='light']) h1,
  :global(.light) h1 {
    color: #1a1a1a;
  }

  :global([data-theme='light']) .lead,
  :global(.light) .lead {
    color: rgba(0, 0, 0, 0.55);
  }

  :global([data-theme='light']) .logo,
  :global(.light) .logo {
    background: linear-gradient(145deg, rgba(100, 108, 255, 0.15) 0%, rgba(100, 108, 255, 0.06) 100%);
    color: #6366f1;
  }
</style>
