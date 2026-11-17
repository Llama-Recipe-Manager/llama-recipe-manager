<script lang="ts">
  import FolderOpen from '@lucide/svelte/icons/folder-open';
  import Plus from '@lucide/svelte/icons/plus';

  import HfDownloader from './HfDownloader.svelte';
  import LocalModelList from './LocalModelList.svelte';
  import CopyButton from '$lib/components/common/CopyButton.svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Tabs from '$lib/components/ui/tabs';
  import { appStore } from '$lib/stores/app.svelte';
  import { downloadsStore } from '$lib/stores/downloads.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';

  let tab = $state<'local' | 'hf'>('local');
  const dir = $derived(settingsStore.current.model_dir);
</script>

<div class="mx-auto flex min-h-0 w-full max-w-[1100px] flex-1 flex-col gap-5 px-8 pt-6 pb-6">
  <header class="flex items-end gap-4">
    <div class="min-w-0 flex-1">
      <h1 class="text-xl font-semibold tracking-tight">Models</h1>
      <p class="mt-1 flex items-center gap-1.5 text-muted-foreground">
        <FolderOpen class="size-3.5" />
        {#if dir}
          <span class="truncate font-mono text-xs" title={dir}>{dir}</span>
        {:else}
          <span>No model directory configured.</span>
        {/if}
      </p>
    </div>
    <Button variant="outline" onclick={() => appStore.goSection('settings')}
      >Change directory</Button
    >
  </header>

  <Tabs.Root bind:value={tab} class="flex min-h-0 flex-1 flex-col gap-4">
    <Tabs.List>
      <Tabs.Trigger value="local" class="px-3">On disk</Tabs.Trigger>
      <Tabs.Trigger value="hf" class="px-3">
        Hugging Face
        {#if downloadsStore.active}
          <span class="font-mono text-[10px] text-primary">
            {Math.round(downloadsStore.percent ?? 0)}%
          </span>
        {/if}
      </Tabs.Trigger>
    </Tabs.List>
    <Tabs.Content value="local" class="flex min-h-0 flex-1 flex-col">
      <LocalModelList directory={dir} class="flex-1">
        {#snippet actions(m)}
          <CopyButton text={m.path} label="Copy path" />
          {#if m.kind === 'Model'}
            <Button size="xs" variant="ghost" onclick={() => appStore.startNew(m.path)}>
              <Plus /> Recipe
            </Button>
          {/if}
        {/snippet}
      </LocalModelList>
    </Tabs.Content>
    <Tabs.Content value="hf" class="flex min-h-0 flex-1 flex-col">
      <HfDownloader class="flex-1" onDownloaded={() => (tab = 'local')} />
    </Tabs.Content>
  </Tabs.Root>
</div>
