<script lang="ts">
  import { Film, Loader2 } from '@lucide/svelte';
  import { onMount, onDestroy } from 'svelte';
  import { apiPath } from '$lib/constants/api';
  import { navigate } from '$lib/router';

  let { id }: { id: string } = $props();

  type TranscodeStatus = 'pending' | 'processing' | 'done' | 'error';

  // Transcoding state
  let status       = $state<TranscodeStatus | 'unknown'>('unknown');
  let qualities    = $state<string[]>([]);
  let selected     = $state<string>('');

  // Video URLs
  let videoUrl     = $state<string | null>(null); // active src for the player
  let rawUrl       = $state<string | null>(null); // fallback: raw file from raw_bucket

  // UI
  let fetchError   = $state<string | null>(null); // only shown if rawUrl itself fails
  let transcodeErr = $state<string | null>(null); // shown as banner when status=error

  let pollTimer: ReturnType<typeof setInterval> | null = null;

  // ── Helpers ─────────────────────────────────────────────────────────────────

  async function fetchRawUrl(): Promise<string | null> {
    const res = await fetch(apiPath(`/api/video/${encodeURIComponent(id)}`));
    if (!res.ok) return null;
    const data = await res.json();
    return data.url ?? null;
  }

  async function fetchQualityUrl(quality: string): Promise<string | null> {
    const res = await fetch(apiPath(`/api/video/${encodeURIComponent(id)}/${encodeURIComponent(quality)}`));
    if (!res.ok) return null;
    const data = await res.json();
    return data.url ?? null;
  }

  async function checkStatus(): Promise<void> {
    const res = await fetch(apiPath(`/api/video/${encodeURIComponent(id)}/status`));

    if (res.status === 404) {
      // Old video with no transcoding info — use raw directly.
      status = 'unknown';
      return;
    }

    const data = await res.json();
    status = data.status ?? 'unknown';

    if (status === 'done') {
      stopPolling();
      qualities = data.qualities ?? [];
      // Select highest available quality by default.
      selected = qualities[qualities.length - 1] ?? '';
      if (selected) {
        videoUrl = await fetchQualityUrl(selected);
      }
    } else if (status === 'error') {
      stopPolling();
      transcodeErr = data.error ?? 'Transcoding failed.';
      // Keep videoUrl pointed at raw so user can still watch.
      videoUrl = rawUrl;
    } else {
      // pending / processing — keep serving raw, poll continues.
      videoUrl = rawUrl;
    }
  }

  async function selectQuality(q: string): Promise<void> {
    selected = q;
    videoUrl = await fetchQualityUrl(q);
  }

  function startPolling(): void {
    pollTimer = setInterval(async () => {
      await checkStatus();
    }, 3000);
  }

  function stopPolling(): void {
    if (pollTimer !== null) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  // ── Mount ────────────────────────────────────────────────────────────────────

  onMount(async () => {
    // 1. Fetch raw URL immediately so user is never blocked.
    try {
      rawUrl = await fetchRawUrl();
      if (!rawUrl) throw new Error('Video not found.');
      videoUrl = rawUrl; // start with raw as default
    } catch (e) {
      fetchError = e instanceof Error ? e.message : 'Failed to load video.';
      return; // no point checking status if raw doesn't exist
    }

    // 2. Check transcoding status.
    await checkStatus();

    // 3. If still in progress, start polling.
    if (status === 'pending' || status === 'processing') {
      startPolling();
    }
  });

  onDestroy(() => stopPolling());
</script>

<main class="mx-auto max-w-[56rem] px-6 py-10 text-left">
  <header class="mb-8">
    <div class="mb-5 flex items-center justify-between">
      <div
        class="inline-flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-indigo-500/20 to-indigo-500/10 text-indigo-400"
      >
        <Film size={28} strokeWidth={1.75} aria-hidden="true" />
      </div>
      <button
        type="button"
        onclick={() => navigate('/upload')}
        class="rounded-xl bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm transition hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:focus:ring-offset-[#0f0f0f]"
      >
        Upload new video
      </button>
    </div>
    <h1 class="text-[1.75rem] font-semibold tracking-tight text-[#1a1a1a] dark:text-white/95">
      Watch — {id}
    </h1>
  </header>

  {#if fetchError}
    <!-- Raw URL failed entirely — nothing to play -->
    <p class="rounded-lg bg-red-500/10 px-4 py-3 text-sm text-red-400">{fetchError}</p>
  {:else}
    <!-- Processing banner (pending / processing) -->
    {#if status === 'pending' || status === 'processing'}
      <div class="mb-3 flex items-center gap-2 rounded-lg bg-amber-500/10 px-4 py-2.5 text-sm text-amber-600 dark:text-amber-400">
        <Loader2 size={15} class="animate-spin shrink-0" aria-hidden="true" />
        HD versions are being processed — you're watching the original in the meantime.
      </div>
    {/if}

    <!-- Transcoding error banner (still shows raw video below) -->
    {#if status === 'error' && transcodeErr}
      <div class="mb-3 rounded-lg bg-red-500/10 px-4 py-2.5 text-sm text-red-400">
        Transcoding failed: {transcodeErr}
      </div>
    {/if}

    <!-- Quality selector (only shown when done) -->
    {#if status === 'done' && qualities.length > 0}
      <div class="mb-3 flex items-center gap-2" role="group" aria-label="Video quality">
        {#each qualities as q (q)}
          <button
            type="button"
            onclick={() => selectQuality(q)}
            class="rounded-lg px-3 py-1 text-sm font-medium transition focus:outline-none focus:ring-2 focus:ring-indigo-500
              {selected === q
                ? 'bg-indigo-600 text-white shadow-sm'
                : 'border border-black/15 bg-white/80 text-[#1a1a1a] hover:bg-white dark:border-white/15 dark:bg-white/10 dark:text-white/90 dark:hover:bg-white/20'}"
          >
            {q}
          </button>
        {/each}
      </div>
    {/if}

    <!-- Player -->
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
