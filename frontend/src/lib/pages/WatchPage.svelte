<script lang="ts">
  import { Film } from '@lucide/svelte';
  import { onMount } from 'svelte';

  let { id }: { id: string } = $props();

  let videoUrl = $state<string | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      const res = await fetch(`/api/video/${encodeURIComponent(id)}`);
      if (!res.ok) {
        const body = await res.json().catch(() => ({}));
        throw new Error(body.error ?? `HTTP ${res.status}`);
      }
      const data = await res.json();
      videoUrl = data.url;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load video';
    }
  });
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

  {#if error}
    <p class="rounded-lg bg-red-500/10 px-4 py-3 text-sm text-red-400">{error}</p>
  {:else}
    <div class="aspect-video overflow-hidden rounded-xl bg-black/40">
      <video
        class="block h-full w-full"
        src={videoUrl ?? undefined}
        controls
        preload="metadata"
        playsinline
        aria-label="Video player"
      >
        <track kind="captions" srclang="en" label="No captions" />
        <p>Your browser does not support the video element.</p>
      </video>
    </div>
  {/if}
</main>
