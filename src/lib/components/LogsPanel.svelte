<script lang="ts">
  import { tick } from 'svelte';
  import { serverStore } from '$lib/stores/server.svelte';
  import { errorMessage } from '$lib/utils/format';

  let { onError }: { onError?: (msg: string) => void } = $props();

  let logContainer = $state<HTMLDivElement | undefined>(undefined);
  let autoScroll = $state(true);
  let filter = $state('');

  const filteredLogs = $derived.by(() => {
    if (!filter.trim()) return serverStore.logs;
    const q = filter.toLowerCase();
    return serverStore.logs.filter((l) => l.line.toLowerCase().includes(q));
  });

  function onScroll() {
    if (!logContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = logContainer;
    autoScroll = scrollHeight - (scrollTop + clientHeight) < 24;
  }

  $effect(() => {
    void serverStore.logs.length;
    if (autoScroll && logContainer) {
      requestAnimationFrame(() => {
        if (logContainer) logContainer.scrollTop = logContainer.scrollHeight;
      });
    }
  });

  async function handleClear() {
    try {
      await serverStore.clearLogs();
      await tick();
    } catch (e) {
      onError?.(errorMessage(e));
    }
  }

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(serverStore.logs.map((l) => l.line).join('\n'));
    } catch (e) {
      onError?.(errorMessage(e));
    }
  }
</script>

<section class="logs">
  <header class="logs-header">
    <div class="logs-title">
      <span class="dot" class:live={serverStore.anyRunning()}></span>
      <h3>Logs</h3>
      <span class="count"
        >{serverStore.logs.length} {serverStore.logs.length === 1 ? 'line' : 'lines'}</span
      >
    </div>
    <div class="logs-actions">
      <input
        type="search"
        class="filter"
        placeholder="Filter…"
        bind:value={filter}
        spellcheck="false"
      />
      <label class="auto">
        <input type="checkbox" bind:checked={autoScroll} />
        <span>Auto-scroll</span>
      </label>
      <button class="action" onclick={handleCopy} disabled={serverStore.logs.length === 0}>
        Copy
      </button>
      <button class="action" onclick={handleClear} disabled={serverStore.logs.length === 0}>
        Clear
      </button>
    </div>
  </header>

  <div class="viewer" bind:this={logContainer} onscroll={onScroll}>
    {#if serverStore.logs.length === 0}
      <div class="empty">
        {serverStore.anyRunning()
          ? 'Waiting for output…'
          : 'No logs yet. Start the server to see output.'}
      </div>
    {:else if filteredLogs.length === 0}
      <div class="empty">No lines match “{filter}”.</div>
    {:else}
      {#each filteredLogs as log}
        <div class="line" class:stderr={log.is_stderr}>{log.line}</div>
      {/each}
    {/if}
  </div>
</section>

<style>
  .logs {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--bg-secondary);
  }

  .logs-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-tertiary);
    flex-wrap: wrap;
  }

  .logs-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .logs-title h3 {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
  }

  .count {
    font-size: 11px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-tertiary);
  }

  .dot.live {
    background: var(--success);
    box-shadow: 0 0 0 0 rgba(52, 199, 89, 0.5);
    animation: pulse 1.6s ease-out infinite;
  }

  @keyframes pulse {
    0% {
      box-shadow: 0 0 0 0 rgba(52, 199, 89, 0.5);
    }
    70% {
      box-shadow: 0 0 0 6px rgba(52, 199, 89, 0);
    }
    100% {
      box-shadow: 0 0 0 0 rgba(52, 199, 89, 0);
    }
  }

  .logs-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .filter {
    width: 160px;
    padding: 4px 8px;
    font-size: 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .auto {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .auto input {
    width: 14px;
    height: 14px;
    accent-color: var(--accent);
  }

  .action {
    padding: 4px 10px;
    font-size: 12px;
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border);
  }

  .action:hover:not(:disabled) {
    background: var(--bg-tertiary);
  }

  .action:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .viewer {
    background: #0f1115;
    height: 360px;
    overflow-y: auto;
    padding: 10px 12px;
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.55;
  }

  .line {
    color: #d4d4d4;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .line.stderr {
    color: #f87171;
  }

  .empty {
    color: #777;
    text-align: center;
    padding: 48px 0;
    font-family: var(--font-sans, system-ui);
    font-size: 13px;
  }
</style>
