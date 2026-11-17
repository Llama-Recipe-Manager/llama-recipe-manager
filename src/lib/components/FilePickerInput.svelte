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
</style>
