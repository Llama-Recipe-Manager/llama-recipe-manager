<script lang="ts">
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

  let models = $state<ScannedModel[]>([]);
  let loading = $state(true);
  let error = $state('');
  let search = $state('');
  let showAll = $state(false);

  $effect(() => {
    loading = true;
    error = '';
    scanModels(directory, showAll ? 'all' : filter)
      .then((m) => {
        models = m;
        loading = false;
      })
      .catch((e: unknown) => {
        error = String(e);
        loading = false;
      });
  });

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

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onClose}>
  <div
    class="browser"
    onclick={(e) => e.stopPropagation()}
    role="dialog"
    aria-label="Browse files"
  >
    <div class="browser-header">
      <h3>Browse {filter === 'mmproj' ? 'mmproj' : 'Model'} Files</h3>
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

    <div class="browser-body">
      {#if loading}
        <div class="browser-empty">Scanning directory…</div>
      {:else if error}
        <div class="browser-error">{error}</div>
      {:else if filtered.length === 0}
        <div class="browser-empty">
          {search
            ? 'No files match your filter.'
            : 'No .gguf files found in this directory.'}
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
    padding: 0 20px 16px;
    min-height: 200px;
    max-height: 400px;
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
