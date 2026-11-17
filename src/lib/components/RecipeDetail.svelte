<script lang="ts">
  import Icon from './Icon.svelte';
  import { onDestroy } from 'svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';

  import ConfirmDialog from './ConfirmDialog.svelte';
  import LiveStatsPanel from './LiveStatsPanel.svelte';
  import LogsPanel from './LogsPanel.svelte';
  import { deleteRecipe, duplicateRecipe } from '$lib/api';
  import { recipesStore } from '$lib/stores/recipes.svelte';
  import { serverStore } from '$lib/stores/server.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import type { Recipe } from '$lib/types';
  import { copyText } from '$lib/utils/clipboard';
  import { errorMessage, formatDate } from '$lib/utils/format';
  import { previewCommand } from '$lib/utils/preview';
  import { formatUptime, webuiUrl } from '$lib/utils/server';

  let {
    recipe,
    onEdit,
    onStart,
    onStop,
    onDeleted,
    onDuplicated,
    onExport,
    onError,
  }: {
    recipe: Recipe;
    onExport: (recipe: Recipe) => void;
    onEdit: () => void;
    onStart: () => void;
    onStop: () => void;
    onDeleted: () => void;
    onDuplicated: (id: string) => void;
    onError: (msg: string) => void;
  } = $props();

  const isReady = $derived(serverStore.isReady(recipe.id));
  const isStarting = $derived(serverStore.isStarting(recipe.id));
  const isStopping = $derived(serverStore.isStopping(recipe.id));
  const isThisActive = $derived(serverStore.isActive(recipe.id));
  const blockedByOther = $derived(serverStore.anyActive() && !isThisActive);
  const url = $derived(webuiUrl(settingsStore.current));
  const webUiAvailable = $derived(isReady && settingsStore.current.webui_enabled);
  const launchedCommand = $derived(
    previewCommand(settingsStore.current, recipe.command, recipe.model_path, recipe.mmproj_path),
  );

  /** Which element most recently copied to the clipboard ("Copied…" feedback). */
  let copied: 'endpoint' | 'command' | null = $state(null);
  let copyTimer: ReturnType<typeof setTimeout> | null = null;

  async function handleCopy(value: string, target: 'endpoint' | 'command') {
    const ok = await copyText(value);
    if (!ok) {
      onError('Could not copy to clipboard.');
      return;
    }
    copied = target;
    if (copyTimer) clearTimeout(copyTimer);
    copyTimer = setTimeout(() => (copied = null), 1600);
  }

  /** Surface a crash banner when this recipe's last run died unexpectedly. */
  const crashInfo = $derived.by(() => {
    const exit = serverStore.lastExit;
    if (!exit || exit.intentional || exit.recipe_id !== recipe.id || isThisActive) return null;
    const parts: string[] = [];
    if (exit.code !== null) parts.push(`exit ${exit.code}`);
    if (exit.signal !== null) parts.push(`signal ${exit.signal}`);
    return parts.length > 0 ? parts.join(' · ') : 'unknown reason';
  });

  // Re-render uptime once a second while running.
  let now = $state(Date.now());
  let timer: ReturnType<typeof setInterval> | null = null;
  $effect(() => {
    if (isReady && serverStore.startedAt !== null) {
      now = Date.now();
      timer = setInterval(() => (now = Date.now()), 1000);
      return () => {
        if (timer) clearInterval(timer);
        timer = null;
      };
    }
    if (timer) {
      clearInterval(timer);
      timer = null;
    }
  });
  onDestroy(() => {
    if (timer) clearInterval(timer);
    if (copyTimer) clearTimeout(copyTimer);
  });

  const uptime = $derived(
    isReady && serverStore.startedAt !== null ? formatUptime(now - serverStore.startedAt) : '',
  );

  async function handleDelete() {
    if (isThisActive) {
      onError('Stop the server before deleting this recipe.');
      return;
    }
    showDeleteConfirm = true;
  }

  async function confirmDelete() {
    showDeleteConfirm = false;
    try {
      await deleteRecipe(recipe.id);
      await recipesStore.refresh();
      onDeleted();
    } catch (e) {
      onError(errorMessage(e));
    }
  }

  async function handleDuplicate() {
    try {
      const r = await duplicateRecipe(recipe.id);
      await recipesStore.refresh();
      onDuplicated(r.id);
    } catch (e) {
      onError(errorMessage(e));
    }
  }

  async function handleOpenWebUi() {
    try {
      await openUrl(url);
    } catch (e) {
      onError(errorMessage(e));
    }
  }

  let showFullCommand = $state(false);
  let showDeleteConfirm = $state(false);
</script>

<div class="detail-view">
  <div class="detail-header">
    <div class="detail-title-block">
      <h2 class="detail-title">{recipe.name}</h2>
      {#if recipe.description}
        <p class="detail-desc">{recipe.description}</p>
      {/if}
    </div>
    <div class="detail-actions">
      <button
        class="btn-icon"
        onclick={onEdit}
        title="Edit"
        aria-label="Edit"
        disabled={isThisActive}
      >
        <Icon name="edit" />
      </button>
      <button class="btn-icon" onclick={handleDuplicate} title="Duplicate" aria-label="Duplicate">
        <Icon name="copy" />
      </button>
      <button
        class="btn-icon"
        onclick={() => onExport(recipe)}
        title="Export (.ripe)"
        aria-label="Export"
      >
        <Icon name="upload" />
      </button>
      <button
        class="btn-icon danger-icon"
        onclick={handleDelete}
        title="Delete"
        aria-label="Delete"
        disabled={isThisActive}
      >
        <Icon name="trash" />
      </button>
    </div>
  </div>

  {#if crashInfo}
    <div class="crash-banner" role="alert">
      <div class="crash-text">
        <strong>Server crashed</strong>
        <span
          >llama-server exited unexpectedly ({crashInfo}). Check the logs below for the reason.</span
        >
      </div>
      <button class="crash-dismiss" onclick={() => serverStore.dismissExit()} aria-label="Dismiss">
        <Icon name="close" size={14} strokeWidth={2.5} />
      </button>
    </div>
  {/if}

  <!-- Hero: status / start-stop / open webui -->
  <div class="hero" class:running={isReady} class:starting={isStarting} class:stopping={isStopping}>
    <div class="hero-left">
      <div class="status-row">
        <span class="state-dot" class:on={isReady} class:warming={isStarting || isStopping}></span>
        <span class="state-label">
          {#if isStopping}
            Stopping…
          {:else if isReady}
            Running
          {:else if isStarting}
            Starting…
          {:else if blockedByOther}
            Another server is running
          {:else}
            Stopped
          {/if}
        </span>
        {#if isReady}
          <span class="state-meta">PID {serverStore.status?.pid ?? '—'} · uptime {uptime}</span>
        {:else if isStarting}
          <span class="state-meta">PID {serverStore.status?.pid ?? '—'} · waiting for /health</span>
        {:else if isStopping}
          <span class="state-meta">PID {serverStore.status?.pid ?? '—'} · draining requests</span>
        {/if}
      </div>
      <div class="endpoint">
        <span class="endpoint-label">Endpoint</span>
        <code>{url}</code>
        <button
          class="copy-btn"
          onclick={() => handleCopy(url, 'endpoint')}
          title={copied === 'endpoint' ? 'Copied!' : 'Copy endpoint URL'}
          aria-label={copied === 'endpoint' ? 'Endpoint URL copied' : 'Copy endpoint URL'}
        >
          {#if copied === 'endpoint'}
            <Icon name="check" size={12} strokeWidth={2.5} />
          {:else}
            <Icon name="copy" size={12} />
          {/if}
        </button>
      </div>
    </div>

    <div class="hero-actions">
      {#if isThisActive}
        {#if webUiAvailable}
          <button class="btn primary" onclick={handleOpenWebUi}>
            <Icon name="external" size={14} />
            Open Web UI
          </button>
        {/if}
        <button class="btn danger" onclick={onStop} disabled={isStopping}>
          {#if isStopping}
            Stopping…
          {:else if isStarting}
            Cancel
          {:else}
            Stop Server
          {/if}
        </button>
      {:else}
        <button class="btn primary large" onclick={onStart} disabled={blockedByOther}>
          <Icon name="play" size={14} />
          Start Server
        </button>
      {/if}
    </div>
  </div>

  <div class="meta-row">
    {#if recipe.gpu_info}
      <span class="badge neutral">{recipe.gpu_info}</span>
    {/if}
    {#if recipe.mmproj_path}
      <span class="badge cap">vision</span>
    {/if}
    {#each recipe.tags.split(',').filter((t) => t.trim()) as tag, i (i)}
      <span class="badge tag">{tag.trim()}</span>
    {/each}
    <span class="meta-spacer"></span>
    <span class="meta-date">
      Updated {formatDate(recipe.updated_at)}
    </span>
  </div>

  <div class="info-grid">
    <div class="info-card" class:span-2={!recipe.mmproj_path}>
      <div class="info-label">Model</div>
      <code class="info-value">{recipe.model_path || '(not set)'}</code>
    </div>
    {#if recipe.mmproj_path}
      <div class="info-card">
        <div class="info-label">Mmproj</div>
        <code class="info-value">{recipe.mmproj_path}</code>
      </div>
    {/if}
    <div class="info-card span-2">
      <div class="info-label">Recipe arguments</div>
      <code class="info-value mono">{recipe.command || '(none)'}</code>
    </div>
  </div>

  <details class="full-cmd" bind:open={showFullCommand}>
    <summary>{showFullCommand ? 'Hide' : 'Show'} full command (as launched)</summary>
    <div class="command-copy-row">
      <button
        class="copy-btn text"
        onclick={() => handleCopy(launchedCommand, 'command')}
        title={copied === 'command' ? 'Copied!' : 'Copy full command'}
        aria-label={copied === 'command' ? 'Full command copied' : 'Copy full command'}
      >
        {#if copied === 'command'}
          <Icon name="check" size={12} strokeWidth={2.5} />
          Copied
        {:else}
          <Icon name="copy" size={12} />
          Copy
        {/if}
      </button>
    </div>
    <pre class="command-block">{launchedCommand}</pre>
  </details>

  {#if isReady}
    <LiveStatsPanel />
  {/if}

  <LogsPanel recipeId={recipe.id} {onError} />

  <ConfirmDialog
    open={showDeleteConfirm}
    title="Delete Recipe"
    message="Are you sure you want to delete this recipe? This action cannot be undone."
    confirmLabel="Delete"
    cancelLabel="Cancel"
    onConfirm={confirmDelete}
    onCancel={() => (showDeleteConfirm = false)}
  />
</div>

<style>
  .detail-view {
    padding: 24px 32px 32px;
    max-width: 920px;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .detail-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  .detail-title-block {
    min-width: 0;
  }

  .detail-title {
    font-size: 22px;
    font-weight: 700;
    letter-spacing: -0.3px;
  }

  .detail-desc {
    color: var(--text-secondary);
    font-size: 14px;
    margin-top: 4px;
  }

  .detail-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .crash-banner {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 14px;
    border-radius: var(--radius-md);
    border: 1px solid rgba(255, 59, 48, 0.45);
    background: rgba(255, 59, 48, 0.08);
    color: var(--text-primary);
  }

  .crash-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 13px;
    line-height: 1.45;
  }

  .crash-text strong {
    color: var(--danger);
    font-weight: 600;
  }

  .crash-text span {
    color: var(--text-secondary);
  }

  .crash-dismiss {
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .crash-dismiss:hover {
    background: rgba(255, 59, 48, 0.12);
    color: var(--danger);
  }

  .hero {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 16px 18px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    flex-wrap: wrap;
  }

  .hero.running {
    border-color: rgba(52, 199, 89, 0.45);
    background: linear-gradient(180deg, rgba(52, 199, 89, 0.07) 0%, var(--bg-secondary) 100%);
  }

  .hero.starting,
  .hero.stopping {
    border-color: rgba(255, 159, 10, 0.45);
    background: linear-gradient(180deg, rgba(255, 159, 10, 0.07) 0%, var(--bg-secondary) 100%);
  }

  .hero-left {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
  }

  .status-row {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .state-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--text-tertiary);
  }

  .state-dot.on {
    background: var(--success);
    box-shadow: 0 0 0 0 rgba(52, 199, 89, 0.55);
    animation: pulse 1.6s ease-out infinite;
  }

  .state-dot.warming {
    background: var(--warning, #ff9f0a);
    box-shadow: 0 0 0 0 rgba(255, 159, 10, 0.55);
    animation: pulse-warm 1.2s ease-out infinite;
  }

  @keyframes pulse {
    0% {
      box-shadow: 0 0 0 0 rgba(52, 199, 89, 0.55);
    }
    70% {
      box-shadow: 0 0 0 8px rgba(52, 199, 89, 0);
    }
    100% {
      box-shadow: 0 0 0 0 rgba(52, 199, 89, 0);
    }
  }

  @keyframes pulse-warm {
    0% {
      box-shadow: 0 0 0 0 rgba(255, 159, 10, 0.55);
    }
    70% {
      box-shadow: 0 0 0 8px rgba(255, 159, 10, 0);
    }
    100% {
      box-shadow: 0 0 0 0 rgba(255, 159, 10, 0);
    }
  }

  .state-label {
    font-size: 14px;
    font-weight: 600;
  }

  .state-meta {
    font-size: 12px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .endpoint {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }

  .endpoint-label {
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-weight: 600;
    font-size: 10px;
  }

  .endpoint code {
    font-family: var(--font-mono);
    color: var(--text-secondary);
    background: var(--bg-tertiary);
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    user-select: text;
  }

  .copy-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 3px 7px;
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    transition:
      background 0.15s,
      color 0.15s;
    font-size: 11px;
    font-weight: 600;
  }

  .copy-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .copy-btn.text {
    border: 1px solid var(--border);
    background: var(--bg-primary);
    color: var(--text-secondary);
  }

  .copy-btn.text:hover {
    color: var(--text-primary);
  }

  .hero-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .meta-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
  }

  .meta-spacer {
    flex: 1;
  }

  .meta-date {
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    font-size: 11px;
    font-weight: 500;
    padding: 3px 10px;
    border-radius: 100px;
  }

  .badge.neutral {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
  }

  .badge.tag {
    background: rgba(0, 113, 227, 0.1);
    color: var(--accent);
  }

  .badge.cap {
    background: rgba(175, 82, 222, 0.12);
    color: #af52de;
  }

  .info-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }

  .info-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 10px 14px;
    min-width: 0;
  }

  .info-card.span-2 {
    grid-column: span 2;
  }

  .info-label {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-tertiary);
    margin-bottom: 4px;
  }

  .info-value {
    font-family: var(--font-mono);
    font-size: 12.5px;
    color: var(--text-primary);
    word-break: break-all;
    user-select: text;
    display: block;
  }

  .full-cmd > summary {
    cursor: pointer;
    font-size: 12px;
    color: var(--text-secondary);
    padding: 4px 0;
    user-select: none;
  }

  .full-cmd > summary:hover {
    color: var(--text-primary);
  }

  .command-copy-row {
    display: flex;
    justify-content: flex-end;
    margin-top: 6px;
    margin-bottom: 2px;
  }

  .command-block {
    margin-top: 8px;
    font-family: var(--font-mono);
    font-size: 12.5px;
    line-height: 1.55;
    background: var(--bg-secondary);
    border: 1px dashed var(--border);
    border-radius: var(--radius-md);
    padding: 12px 14px;
    white-space: pre-wrap;
    word-break: break-all;
    user-select: text;
  }

  .btn {
    gap: 6px;
  }

  .btn:not(.large) {
    padding: 7px 14px;
  }

  .btn-icon.danger-icon:hover:not(:disabled) {
    background: rgba(255, 59, 48, 0.1);
    color: var(--danger);
  }
</style>
