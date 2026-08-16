<script lang="ts">
  import { onMount } from 'svelte';
  import { scanModels, type ScannedModel } from '$lib/api/models';

  let {
    directory,
    onSelect,
    onClose,
    filter = 'model',
  }: {
    directory: string;
    onSelect: (path: string) => void;
    onClose: () => void;
    filter: 'model' | 'mmproj';
  } = $props();

  let dialog = $state<HTMLDivElement | undefined>(undefined);
  let body = $state<HTMLDivElement | undefined>(undefined);

  onMount(() => {
    // Move focus into the dialog on open so keyboard/screen-reader users land
    // inside it immediately. Restoring focus to the trigger happens naturally
    // when the component is destroyed below it in the DOM tree.
    dialog?.focus({ preventScroll: true });
  });

  const PAGE_SIZE = 100;

  let models = $state<ScannedModel[]>([]);
  let total = $state(0);
  let page = $state(0);
  let loading = $state(true);
  let loadingMore = $state(false);
  let error = $state('');
  let search = $state('');
  let showAll = $state(false);

  // Monotonic token so a stale in-flight request can never clobber a newer one
  // (e.g. the user toggles "View all" while a page is still loading).
  let requestToken = 0;

  async function load(reset: boolean): Promise<void> {
    if (!reset && (loading || loadingMore)) return;
    const nextPage = reset ? 1 : page + 1;
    const token = ++requestToken;
    if (reset) {
      loading = true;
    } else {
      loadingMore = true;
    }
    error = '';
    try {
      const res = await scanModels(directory, showAll ? 'all' : filter, nextPage, PAGE_SIZE);
      if (token !== requestToken) return; // superseded by a newer request
      models = reset ? res.items : [...models, ...res.items];
      total = res.total;
      page = res.page;
    } catch (e) {
      if (token !== requestToken) return;
      error = String(e);
    } finally {
      if (token === requestToken) {
        loading = false;
        loadingMore = false;
      }
    }
  }

  // Reload from page 1 whenever the scan inputs change.
  $effect(() => {
    void directory;
    void showAll;
    void filter;
    void load(true);
  });

  const hasMore = $derived(!error && !loading && models.length < total);

  // Load the next page when the user scrolls near the bottom of the list.
  function onScroll() {
    if (!body || !hasMore) return;
    const { scrollTop, scrollHeight, clientHeight } = body;
    if (scrollHeight - (scrollTop + clientHeight) < 120) {
      void load(false);
    }
  }

  function formatSize(bytes: number): string {
    if (bytes === 0) return '—';
    if (bytes < 1_000_000_000) return `${(bytes / 1_000_000).toFixed(1)} MB`;
    return `${(bytes / 1_000_000_000).toFixed(2)} GB`;
  }

  function kindLabel(kind: string): string {
    return kind === 'Mmproj' ? 'mmproj' : 'model';
  }

  const filtered = $derived(
    !search.trim()
      ? models
      : models.filter((m) => m.name.toLowerCase().includes(search.toLowerCase())),
  );

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onClose}>
  <div
    class="browser"
    bind:this={dialog}
    onclick={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    aria-labelledby="browser-title"
    tabindex="-1"
  >
    <div class="browser-header">
      <h3 id="browser-title">
        Browse {filter === 'mmproj' ? 'mmproj' : 'Model'} Files
        {#if !loading && !error}
          <span class="browser-count"
            >({models.length}{total > models.length ? ` / ${total}` : ''})</span
          >
        {/if}
      </h3>
      <button class="btn-icon" onclick={onClose} aria-label="Close">
        <svg
          width="16"
          height="16"
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

    <p class="browser-dir">Scanning: <code>{directory}</code></p>

    <div class="browser-controls">
      <div class="browser-search">
        <input type="text" bind:value={search} placeholder="Filter files…" />
      </div>
      <label class="show-all-label">
        <input type="checkbox" bind:checked={showAll} />
        <span>View all ggufs</span>
      </label>
    </div>

    <div class="browser-body" bind:this={body} onscroll={onScroll}>
      {#if loading}
        <div class="browser-empty">Scanning directory…</div>
      {:else if error}
        <div class="browser-error">{error}</div>
      {:else if filtered.length === 0}
        <div class="browser-empty">
          {search
            ? 'No files match your filter.'
            : showAll
              ? 'No .gguf files found in this directory.'
              : `No ${filter === 'mmproj' ? 'mmproj' : 'model'} files found in this directory.`}
        </div>
      {:else}
        <div class="model-list">
          {#each filtered as model (model.path)}
            <button class="model-item" onclick={() => onSelect(model.path)}>
              <div class="model-item-name">
                {model.name}
                <span class="model-item-kind" class:mmproj={model.kind === 'Mmproj'}
                  >{kindLabel(model.kind)}</span
                >
              </div>
              <div class="model-item-path">{model.path}</div>
              <div class="model-item-size">{formatSize(model.size_bytes)}</div>
            </button>
          {/each}
        </div>
      {/if}
    </div>

    {#if hasMore}
      <div class="browser-footer">
        {#if loadingMore}
          <span class="loading-more">Loading more…</span>
        {:else}
          <button class="load-more" onclick={() => load(false)}>
            Load more ({total - models.length} remaining)
          </button>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .browser {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    width: 560px;
    max-width: calc(100vw - 64px);
    max-height: calc(100vh - 80px);
    display: flex;
    flex-direction: column;
    box-shadow: var(--shadow-md);
  }

  .browser-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px 8px;
  }

  .browser-header h3 {
    font-size: 16px;
    font-weight: 700;
  }

  .browser-count {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .browser-dir {
    font-size: 12px;
    color: var(--text-tertiary);
    padding: 0 20px 12px;
  }

  .browser-dir code {
    font-family: var(--font-mono);
    color: var(--text-secondary);
  }

  .browser-controls {
    padding: 0 20px 12px;
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .browser-search {
    flex: 1;
  }

  .browser-search input {
    width: 100%;
    padding: 7px 12px;
    font-size: 13px;
  }

  .show-all-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-secondary);
    white-space: nowrap;
    cursor: pointer;
  }

  .show-all-label input[type='checkbox'] {
    width: 14px;
    height: 14px;
    accent-color: var(--accent);
  }

  .browser-body {
    flex: 1;
    overflow-y: auto;
    padding: 0 20px;
    min-height: 200px;
    max-height: 400px;
  }

  .browser-footer {
    padding: 10px 20px 16px;
    text-align: center;
  }

  .load-more {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 7px 18px;
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 500;
    transition: background 0.15s;
  }

  .load-more:hover {
    background: var(--border);
  }

  .loading-more {
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .browser-empty,
  .browser-error {
    padding: 32px 0;
    text-align: center;
    font-size: 13px;
    color: var(--text-tertiary);
  }

  .browser-error {
    color: var(--danger);
  }

  .model-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .model-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    text-align: left;
    transition: background 0.1s;
    border: none;
    background: none;
    cursor: pointer;
    width: 100%;
  }

  .model-item:hover {
    background: var(--bg-secondary);
  }

  .model-item:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .model-item-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .model-item-kind {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.4px;
    padding: 1px 6px;
    border-radius: 3px;
    background: var(--bg-tertiary);
    color: var(--text-tertiary);
  }

  .model-item-kind.mmproj {
    background: rgba(128, 90, 213, 0.15);
    color: #a78bfa;
  }

  .model-item-path {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-tertiary);
    word-break: break-all;
  }

  .model-item-size {
    font-size: 11px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .btn-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    transition: all 0.15s;
    border: none;
    background: none;
    cursor: pointer;
  }

  .btn-icon:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }
</style>
