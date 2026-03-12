<script lang="ts">
  import { Film } from '@lucide/svelte';

  let { id }: { id: string } = $props();
</script>

<main class="mx-auto max-w-[56rem] px-6 py-10 text-left">
  <header class="mb-8">
    <div
      class="mb-5 inline-flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-indigo-500/20 to-indigo-500/10 text-indigo-400"
    >
      <Film size={28} strokeWidth={1.75} aria-hidden="true" />
    </div>
    <h1 class="text-[1.75rem] font-semibold tracking-tight text-[#1a1a1a] dark:text-white/95">
      Watch — {id}
    </h1>
  </header>

  <div class="aspect-video overflow-hidden rounded-xl bg-black/40">
    <video
      class="block h-full w-full"
      controls
      preload="metadata"
      playsinline
      aria-label="Video player"
    >
      <p>Your browser does not support the video element.</p>
    </video>
  </div>
</main>
