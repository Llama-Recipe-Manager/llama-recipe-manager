<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import {
    listHfModelFiles,
    downloadHfModel,
    type DownloadProgress,
    type HfModelFile,
  } from '$lib/api/models';

  let {
    destDir,
    hfToken,
    onSelect,
    onClose,
    filter = 'model',
  }: {
    destDir: string;
    hfToken: string;
    onSelect: (path: string) => void;
    onClose: () => void;
    filter: 'model' | 'mmproj';
  } = $props();

  let repoId = $state('');
  let files = $state<HfModelFile[]>([]);
  let loading = $state(false);
  let error = $state('');
  let downloading = $state<string | null>(null);
  let progress = $state<DownloadProgress | null>(null);
  let showAll = $state(false);

  let unlisten: (() => void) | null = null;

  $effect(() => {
    return () => {
      unlisten?.();
    };
  });

  async function browseFiles() {
    const id = repoId.trim();
    if (!id) return;
    loading = true;
    error = '';
    files = [];
    try {
      files = await listHfModelFiles(id, hfToken, showAll ? 'all' : filter);
      if (files.length === 0) {
        error = 'No .gguf files found in this repository.';
      }
    } catch (e) {
      error = String(e);
    }
    loading = false;
  }

  function formatBytes(n: number): string {
    if (n === 0) return '—';
    if (n < 1_000_000_000) return `${(n / 1_000_000).toFixed(0)} MB`;
    return `${(n / 1_000_000_000).toFixed(2)} GB`;
  }

  async function startDownload(file: HfModelFile) {
    downloading = file.name;
    progress = null;
    error = '';
    unlisten?.();

    unlisten = await listen<DownloadProgress>('download-progress', (event) => {
      if (event.payload.filename === file.name) {
        progress = event.payload;
      }
    });

    try {
      const path = await downloadHfModel(repoId.trim(), file.name, hfToken, destDir);
      unlisten?.();
      onSelect(path);
    } catch (e) {
      unlisten?.();
      downloading = null;
      error = String(e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onClose}>
  <div
    class="downloader"
    onclick={(e) => e.stopPropagation()}
    role="dialog"
    aria-label="Download from HuggingFace"
  >
    <div class="downloader-header">
      <h3>Download {filter === 'mmproj' ? 'mmproj' : 'Model'}</h3>
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

    <div class="downloader-body">
      <div class="form-group">
        <label for="repo-id">HuggingFace Repo ID</label>
        <div class="repo-input-row">
          <input
            id="repo-id"
            type="text"
            bind:value={repoId}
            placeholder="e.g. Qwen/Qwen2.5-1.5B-Instruct-GGUF"
            disabled={!!downloading}
          />
          <button
            class="btn secondary"
            onclick={browseFiles}
            disabled={loading || !repoId.trim() || !!downloading}
          >
            {loading ? 'Loading…' : 'Browse files'}
          </button>
        </div>
        <span class="form-hint">Enter a HuggingFace model repo that contains .gguf files.</span>
      </div>

      <label class="show-all-label">
        <input type="checkbox" bind:checked={showAll} />
        <span>View all ggufs</span>
      </label>

      {#if error}
        <div class="error-msg">{error}</div>
      {/if}

      {#if loading}
        <div class="status-msg">Querying HuggingFace Hub…</div>
      {/if}

      {#if files.length > 0}
        <div class="file-list-header">
          <span>Available files ({files.length})</span>
        </div>
        <div class="file-list">
          {#each files as file (file.name)}
            <div class="file-item">
              <div class="file-info">
                <div class="file-name">{file.name}</div>
                <div class="file-size">{formatBytes(file.size_bytes)}</div>
              </div>
              <button
                class="btn primary small"
                onclick={() => startDownload(file)}
                disabled={!!downloading}
              >
                {#if downloading === file.name}
                  {#if progress}
                    {Math.round((progress.bytes_downloaded / progress.total_bytes) * 100)}%
                  {:else}
                    Starting…
                  {/if}
                {:else}
                  Download
                {/if}
              </button>
            </div>
          {/each}
        </div>
      {/if}

      {#if downloading && progress}
        <div class="progress-bar-wrap">
          <div
            class="progress-bar-fill"
            style="width: {Math.min(
              100,
              (progress.bytes_downloaded / Math.max(1, progress.total_bytes)) * 100,
            )}%"
          ></div>
        </div>
        <div class="progress-text">
          {formatBytes(progress.bytes_downloaded)} / {formatBytes(progress.total_bytes)}
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

  .downloader {
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

  .downloader-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px 8px;
  }

  .downloader-header h3 {
    font-size: 16px;
    font-weight: 700;
  }

  .downloader-body {
    flex: 1;
    overflow-y: auto;
    padding: 8px 20px 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-height: 200px;
    max-height: 480px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .form-group label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .form-hint {
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .repo-input-row {
    display: flex;
    gap: 8px;
  }

  .repo-input-row input {
    flex: 1;
  }

  .show-all-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .show-all-label input[type='checkbox'] {
    width: 14px;
    height: 14px;
    accent-color: var(--accent);
  }

  .error-msg {
    font-size: 13px;
    color: var(--danger);
    padding: 8px 12px;
    background: rgba(255, 59, 48, 0.08);
    border-radius: var(--radius-sm);
  }

  .status-msg {
    font-size: 13px;
    color: var(--text-tertiary);
    text-align: center;
    padding: 16px 0;
  }

  .file-list-header {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .file-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 240px;
    overflow-y: auto;
  }

  .file-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    transition: background 0.1s;
  }

  .file-item:hover {
    background: var(--bg-secondary);
  }

  .file-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }

  .file-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    word-break: break-all;
  }

  .file-size {
    font-size: 11px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .progress-bar-wrap {
    width: 100%;
    height: 6px;
    background: var(--bg-tertiary);
    border-radius: 3px;
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 3px;
    transition: width 0.3s ease;
  }

  .progress-text {
    font-size: 11px;
    color: var(--text-tertiary);
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 7px 16px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 500;
    transition: all 0.15s;
    white-space: nowrap;
    border: none;
    cursor: pointer;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn.small {
    padding: 5px 12px;
    font-size: 12px;
  }

  .btn.primary {
    background: var(--accent);
    color: white;
  }

  .btn.primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .btn.secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .btn.secondary:hover:not(:disabled) {
    background: var(--border);
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
