<script lang="ts">
  import { recipesStore } from '$lib/stores/recipes.svelte';
  import { serverStore } from '$lib/stores/server.svelte';

  let { onStop }: { onStop: () => void } = $props();

  const runningRecipe = $derived(
    recipesStore.items.find((r) => r.id === serverStore.runningRecipeId()) ?? null,
  );
  const starting = $derived(serverStore.phase === 'starting');
  const stopping = $derived(serverStore.phase === 'stopping');
  const transient = $derived(starting || stopping);
</script>

{#if serverStore.anyActive()}
  <div class="server-bar" class:warming={transient}>
    <span class="status-dot" class:running={!transient} class:warming={transient}></span>
    <span class="server-bar-text">
      {#if stopping}
        Stopping
      {:else if starting}
        Starting
      {:else}
        Server running
      {/if}
      {#if runningRecipe}
        &mdash; {runningRecipe.name}
      {/if}
    </span>
    <button class="server-bar-stop" onclick={onStop} disabled={stopping}>
      {#if stopping}
        Stopping…
      {:else if starting}
        Cancel
      {:else}
        Stop
      {/if}
    </button>
  </div>
{/if}

<style>
  .server-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: rgba(52, 199, 89, 0.08);
    border-top: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
  }

  .server-bar-text {
    flex: 1;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .server-bar-stop {
    font-size: 12px;
    font-weight: 600;
    color: var(--danger);
    flex-shrink: 0;
  }

  .server-bar-stop:hover {
    text-decoration: underline;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .status-dot.running {
    background: var(--success);
    box-shadow: 0 0 4px var(--success);
  }

  .status-dot.warming {
    background: var(--warning, #ff9f0a);
    box-shadow: 0 0 4px var(--warning, #ff9f0a);
  }

  .server-bar.warming {
    background: rgba(255, 159, 10, 0.08);
  }
</style>
