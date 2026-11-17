<script lang="ts">
  import Download from '@lucide/svelte/icons/download';
  import X from '@lucide/svelte/icons/x';

  import { listHfModelFiles, type HfModelFile } from '$lib/api/models';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Progress } from '$lib/components/ui/progress';
  import { Switch } from '$lib/components/ui/switch';
  import { downloadsStore } from '$lib/stores/downloads.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { cn } from '$lib/utils/cn';
  import { errorMessage } from '$lib/utils/format';
  import { formatBytes, quantOf } from '$lib/utils/model';

  let {
    filter = 'model',
    onDownloaded,
    class: className = '',
  }: {
    filter?: 'model' | 'mmproj';
    onDownloaded?: (path: string) => void;
    class?: string;
  } = $props();

  let repoId = $state('');
  let files = $state<HfModelFile[]>([]);
  let loading = $state(false);
  let error = $state('');
  let showAll = $state(false);
  let searched = $state(false);

  const busy = $derived(downloadsStore.active !== null);
  const noDir = $derived(!settingsStore.current.model_dir.trim());

  async function browse(e?: SubmitEvent) {
    e?.preventDefault();
    const id = repoId.trim();
    if (!id) return;
    loading = true;
    error = '';
    files = [];
    try {
      files = await listHfModelFiles(id, settingsStore.current.hf_token, showAll ? 'all' : filter);
      searched = true;
    } catch (err) {
      error = errorMessage(err);
    } finally {
      loading = false;
    }
  }

  async function download(file: HfModelFile) {
    const path = await downloadsStore.start(repoId.trim(), file.name);
    if (path) onDownloaded?.(path);
  }
</script>

<div class={cn('flex min-h-0 flex-col gap-3', className)}>
  <form class="flex items-center gap-2" onsubmit={browse}>
    <Input
      bind:value={repoId}
      placeholder="owner/repo, e.g. unsloth/Qwen3-30B-A3B-GGUF"
      class="h-8 flex-1 font-mono text-xs"
      spellcheck="false"
      autocomplete="off"
    />
    <Button type="submit" variant="secondary" disabled={loading || !repoId.trim()}>
      {loading ? 'Searching…' : 'List files'}
    </Button>
  </form>
  <div class="flex items-center gap-2">
    <Switch id="hf-show-all" bind:checked={showAll} />
    <Label for="hf-show-all" class="text-xs text-muted-foreground">Include all GGUFs</Label>
    {#if noDir}
      <span class="ml-auto text-xs text-warning">Set a model directory in Settings first.</span>
    {/if}
  </div>

  {#if downloadsStore.active}
    {@const a = downloadsStore.active}
    <div class="rounded-lg border bg-card p-3">
      <div class="flex items-center gap-2 text-xs">
        <Download class="size-3.5 text-primary" />
        <span class="flex-1 truncate font-mono">{a.file}</span>
        <span class="font-mono text-muted-foreground">
          {#if a.progress}
            {formatBytes(a.progress.bytes_downloaded)} / {formatBytes(a.progress.total_bytes)}
          {:else}
            Starting…
          {/if}
        </span>
        <Button
          variant="ghost"
          size="icon-xs"
          aria-label="Cancel download"
          onclick={() => downloadsStore.cancel()}
        >
          <X />
        </Button>
      </div>
      <Progress value={downloadsStore.percent ?? 0} class="mt-2 h-1.5" />
    </div>
  {/if}

  {#if error}
    <p class="text-xs text-destructive">{error}</p>
  {/if}

  {#if files.length}
    <ul class="min-h-0 flex-1 divide-y overflow-y-auto rounded-lg border bg-card">
      {#each files as file (file.name)}
        {@const quant = quantOf(file.name)}
        <li class="flex items-center gap-3 px-3 py-2">
          <span class="min-w-0 flex-1 truncate font-mono text-xs" title={file.name}>
            {file.name}
          </span>
          {#if quant}<span class="font-mono text-[11px] text-muted-foreground">{quant}</span>{/if}
          <span class="w-16 text-right font-mono text-[11px] text-faint">
            {file.size_bytes ? formatBytes(file.size_bytes) : '—'}
          </span>
          <Button
            size="xs"
            variant="outline"
            disabled={busy || noDir}
            onclick={() => download(file)}
          >
            <Download /> Get
          </Button>
        </li>
      {/each}
    </ul>
  {:else if searched && !loading && !error}
    <p class="text-xs text-faint">No .gguf files found in this repository.</p>
  {/if}
</div>
