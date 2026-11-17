<script lang="ts">
  import { open as openFile } from '@tauri-apps/plugin-dialog';
  import FolderOpen from '@lucide/svelte/icons/folder-open';
  import Library from '@lucide/svelte/icons/library';

  import ModelPickerDialog from '$lib/components/models/ModelPickerDialog.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { notifyError } from '$lib/utils/notify';

  let {
    value = $bindable(''),
    kind,
    id,
    placeholder,
    invalid = false,
  }: {
    value: string;
    kind: 'model' | 'mmproj';
    id?: string;
    placeholder: string;
    invalid?: boolean;
  } = $props();

  let pickerOpen = $state(false);

  async function browse() {
    try {
      const result = await openFile({
        multiple: false,
        filters: [{ name: 'GGUF', extensions: ['gguf'] }],
        defaultPath: settingsStore.current.model_dir || undefined,
      });
      if (typeof result === 'string') value = result;
    } catch (e) {
      notifyError(e);
    }
  }
</script>

<div class="flex items-center gap-2">
  <Input
    {id}
    bind:value
    {placeholder}
    class="h-8 flex-1 font-mono text-xs"
    spellcheck="false"
    aria-invalid={invalid || undefined}
  />
  <Button variant="outline" onclick={() => (pickerOpen = true)}>
    <Library /> Library
  </Button>
  <Button
    variant="ghost"
    size="icon"
    aria-label="Browse files"
    title="Browse files"
    onclick={browse}
  >
    <FolderOpen />
  </Button>
</div>

<ModelPickerDialog bind:open={pickerOpen} {kind} onSelect={(p) => (value = p)} />
