<script lang="ts">
  import HfDownloader from './HfDownloader.svelte';
  import LocalModelList from './LocalModelList.svelte';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as Tabs from '$lib/components/ui/tabs';
  import { settingsStore } from '$lib/stores/settings.svelte';

  let {
    open = $bindable(false),
    kind,
    onSelect,
  }: {
    open: boolean;
    kind: 'model' | 'mmproj';
    onSelect: (path: string) => void;
  } = $props();

  function pick(path: string) {
    open = false;
    onSelect(path);
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="flex h-[560px] flex-col sm:max-w-2xl">
    <Dialog.Header>
      <Dialog.Title>Choose {kind === 'mmproj' ? 'a vision projector' : 'a model'}</Dialog.Title>
      <Dialog.Description>
        Pick a .gguf file from your model directory, or download one from Hugging Face.
      </Dialog.Description>
    </Dialog.Header>
    <Tabs.Root value="local" class="flex min-h-0 flex-1 flex-col gap-3">
      <Tabs.List>
        <Tabs.Trigger value="local" class="px-3">On disk</Tabs.Trigger>
        <Tabs.Trigger value="hf" class="px-3">Hugging Face</Tabs.Trigger>
      </Tabs.List>
      <Tabs.Content value="local" class="flex min-h-0 flex-1 flex-col">
        <LocalModelList
          directory={settingsStore.current.model_dir}
          filter={kind}
          class="flex-1"
          onPick={(m) => pick(m.path)}
        />
      </Tabs.Content>
      <Tabs.Content value="hf" class="flex min-h-0 flex-1 flex-col">
        <HfDownloader filter={kind} class="flex-1" onDownloaded={pick} />
      </Tabs.Content>
    </Tabs.Root>
  </Dialog.Content>
</Dialog.Root>
