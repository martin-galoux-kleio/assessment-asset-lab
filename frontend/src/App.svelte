<script lang="ts">
  import { route } from '$lib/router';
  import UploadPage from '$lib/pages/UploadPage.svelte';
  import WatchPage from '$lib/pages/WatchPage.svelte';
</script>

{#if $route.name === 'watch'}
  {@const watchId = $route.params.id}
  <WatchPage id={watchId} />
{:else}
  <UploadPage />
{/if}
