<script lang="ts">
  import Icon from './Icon.svelte';
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
    <Icon name="scan" size={14} />
    Scan
  </button>
  <button
    class="btn secondary"
    onclick={() => (dialog = 'download')}
    title="Download from HuggingFace"
  >
    <Icon name="download" size={14} />
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
</style>
