<script lang="ts">
  let { message }: { message: string } = $props();
</script>

{#if message}
  <div
    class="mt-3 rounded-lg border border-red-500/40 bg-red-500/10 px-4 py-3 text-left text-sm text-red-800 dark:border-red-500/50 dark:bg-red-500/15 dark:text-red-300"
    role="alert"
  >
    {message}
  </div>
{/if}
