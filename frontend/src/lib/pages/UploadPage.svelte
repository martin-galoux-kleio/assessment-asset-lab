<script lang="ts">
  import { Film } from '@lucide/svelte';
  import VideoUploadSection from '$lib/organisms/VideoUploadSection.svelte';
</script>

<main class="mx-auto max-w-[42rem] px-6 py-10 text-left">
  <header class="mb-8">
    <div
      class="mb-5 inline-flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-indigo-500/20 to-indigo-500/10 text-indigo-400 dark:from-indigo-500/20 dark:to-indigo-500/10 dark:text-indigo-400"
    >
      <Film size={28} strokeWidth={1.75} aria-hidden="true" />
    </div>
    <h1 class="mb-2 text-[1.75rem] font-semibold tracking-tight text-[#1a1a1a] dark:text-white/95">
      Upload video
    </h1>
    <p class="m-0 text-[0.9375rem] leading-normal text-black/55 dark:text-white/55">
      Drop a file or click to choose. Max size 1 GB.
    </p>
  </header>
  <VideoUploadSection />
</main>
