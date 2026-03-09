<script lang="ts">
  interface Props {
    message: string;
  }
  let { message }: Props = $props();
</script>

{#if message}
  <div class="error" role="alert">
    {message}
  </div>
{/if}

<style>
  .error {
    margin-top: 0.75rem;
    padding: 0.75rem 1rem;
    border-radius: 8px;
    background-color: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.5);
    color: #fca5a5;
    font-size: 0.875rem;
    text-align: left;
  }
  :global([data-theme='light']) .error,
  :global(.light) .error {
    background-color: rgba(239, 68, 68, 0.1);
    border-color: rgba(239, 68, 68, 0.4);
    color: #b91c1c;
  }
</style>
