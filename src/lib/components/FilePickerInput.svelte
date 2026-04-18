<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';

  let {
    value = $bindable(''),
    placeholder = 'No file selected',
    extensions = ['gguf'],
    filterName = 'GGUF Models',
    defaultPath,
  }: {
    value: string;
    placeholder?: string;
    extensions?: string[];
    filterName?: string;
    defaultPath?: string;
  } = $props();

  async function pick() {
    const result = await open({
      multiple: false,
      filters: [{ name: filterName, extensions }],
      defaultPath,
    });
    if (typeof result === 'string') {
      value = result;
    }
  }
</script>

<div class="input-with-button">
  <input type="text" bind:value {placeholder} class="mono" readonly />
  <button class="btn secondary" onclick={pick}>Browse...</button>
</div>

<style>
  .input-with-button {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .input-with-button input {
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
