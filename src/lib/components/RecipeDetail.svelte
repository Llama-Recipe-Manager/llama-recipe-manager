<script lang="ts">
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
    onError,
  }: {
    recipe: Recipe;
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
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
        >
          <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" />
          <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z" />
        </svg>
      </button>
      <button class="btn-icon" onclick={handleDuplicate} title="Duplicate" aria-label="Duplicate">
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
        >
          <rect x="9" y="9" width="13" height="13" rx="2" />
          <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" />
        </svg>
      </button>
      <button
        class="btn-icon danger-icon"
        onclick={handleDelete}
        title="Delete"
        aria-label="Delete"
        disabled={isThisActive}
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
        >
          <polyline points="3,6 5,6 21,6" />
          <path
            d="M19,6v14a2,2,0,0,1-2,2H7a2,2,0,0,1-2-2V6m3,0V4a2,2,0,0,1,2-2h4a2,2,0,0,1,2,2v2"
          />
        </svg>
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
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
        >
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
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
      </div>
    </div>

    <div class="hero-actions">
      {#if isThisActive}
        {#if webUiAvailable}
          <button class="btn primary" onclick={handleOpenWebUi}>
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
            >
              <path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6" />
              <polyline points="15 3 21 3 21 9" />
              <line x1="10" y1="14" x2="21" y2="3" />
            </svg>
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
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <polygon points="6,4 20,12 6,20" />
          </svg>
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
    <pre class="command-block">{previewCommand(
        settingsStore.current,
        recipe.command,
        recipe.model_path,
        recipe.mmproj_path,
      )}</pre>
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
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 7px 14px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 500;
    transition: all 0.15s;
    white-space: nowrap;
  }

  .btn.large {
    padding: 9px 18px;
    font-size: 14px;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn.primary {
    background: var(--accent);
    color: white;
  }

  .btn.primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .btn.danger {
    background: var(--danger);
    color: white;
  }

  .btn.danger:hover {
    background: var(--danger-hover);
  }

  .btn-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    transition: all 0.15s;
  }

  .btn-icon:hover:not(:disabled) {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .btn-icon:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .btn-icon.danger-icon:hover:not(:disabled) {
    background: rgba(255, 59, 48, 0.1);
    color: var(--danger);
  }
</style>
