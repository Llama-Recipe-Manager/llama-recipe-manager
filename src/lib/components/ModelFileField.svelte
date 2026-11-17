<script lang="ts">
  import FilePickerInput from './FilePickerInput.svelte';
  import ModelBrowser from './ModelBrowser.svelte';
  import ModelDownloader from './ModelDownloader.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';

  let {
    value = $bindable(''),
    kind,
    placeholder,
  }: {
    value: string;
    kind: 'model' | 'mmproj';
    placeholder: string;
  } = $props();

  let dialog = $state<'browse' | 'download' | null>(null);

  function select(path: string) {
    value = path;
    dialog = null;
  }
</script>

<div class="row">
  <FilePickerInput
    bind:value
    {placeholder}
    defaultPath={settingsStore.current.model_dir || undefined}
  />
  <button
    class="btn secondary"
    onclick={() => (dialog = 'browse')}
    title={kind === 'mmproj' ? 'Scan for mmproj files' : 'Scan model directory'}
  >
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
    >
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
      <line x1="8" y1="11" x2="14" y2="11" />
      <line x1="11" y1="8" x2="11" y2="14" />
    </svg>
    Scan
  </button>
  <button
    class="btn secondary"
    onclick={() => (dialog = 'download')}
    title="Download from HuggingFace"
  >
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
    >
      <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
      <polyline points="7 10 12 15 17 10" />
      <line x1="12" y1="15" x2="12" y2="3" />
    </svg>
    Download
  </button>
</div>

{#if dialog === 'browse'}
  <ModelBrowser
    filter={kind}
    directory={settingsStore.current.model_dir}
    onSelect={select}
    onClose={() => (dialog = null)}
  />
{:else if dialog === 'download'}
  <ModelDownloader
    filter={kind}
    destDir={settingsStore.current.model_dir}
    hfToken={settingsStore.current.hf_token}
    onSelect={select}
    onClose={() => (dialog = null)}
  />
{/if}

<style>
  .row {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .row :global(.input-with-button) {
    flex: 1;
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
  }

  .btn.secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .btn.secondary:hover {
    background: var(--border);
  }
</style>
