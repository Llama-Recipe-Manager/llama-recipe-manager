<script lang="ts">
  import type { Snippet } from 'svelte';
  import Box from '@lucide/svelte/icons/box';
  import Eye from '@lucide/svelte/icons/eye';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';
  import Search from '@lucide/svelte/icons/search';

  import { scanModels, type ScannedModel } from '$lib/api/models';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';
  import { downloadsStore } from '$lib/stores/downloads.svelte';
  import { cn } from '$lib/utils/cn';
  import { errorMessage } from '$lib/utils/format';
  import { formatBytes, quantOf } from '$lib/utils/model';

  let {
    directory,
    filter = 'model',
    onPick,
    actions,
    class: className = '',
  }: {
    directory: string;
    filter?: 'model' | 'mmproj';
    onPick?: (model: ScannedModel) => void;
    actions?: Snippet<[ScannedModel]>;
    class?: string;
  } = $props();

  const PAGE_SIZE = 100;

  let models = $state<ScannedModel[]>([]);
  let total = $state(0);
  let page = $state(0);
  let loading = $state(true);
  let loadingMore = $state(false);
  let error = $state('');
  let search = $state('');
  let showAll = $state(false);
  let body = $state<HTMLDivElement | undefined>(undefined);
  let requestToken = 0;

  async function load(reset: boolean): Promise<void> {
    if (!directory.trim()) {
      loading = false;
      models = [];
      total = 0;
      return;
    }
    if (!reset && (loading || loadingMore)) return;
    const token = ++requestToken;
    if (reset) loading = true;
    else loadingMore = true;
    error = '';
    try {
      const res = await scanModels(
        directory,
        showAll ? 'all' : filter,
        reset ? 1 : page + 1,
        PAGE_SIZE,
      );
      if (token !== requestToken) return;
      models = reset ? res.items : [...models, ...res.items];
      total = res.total;
      page = res.page;
    } catch (e) {
      if (token === requestToken) error = errorMessage(e);
    } finally {
      if (token === requestToken) {
        loading = false;
        loadingMore = false;
      }
    }
  }

  $effect(() => {
    void directory;
    void showAll;
    void filter;
    void downloadsStore.completed;
    void load(true);
  });

  const hasMore = $derived(!error && !loading && models.length < total);
  const filtered = $derived.by(() => {
    const q = search.trim().toLowerCase();
    return q ? models.filter((m) => m.name.toLowerCase().includes(q)) : models;
  });

  function onScroll() {
    if (!body || !hasMore) return;
    const { scrollTop, scrollHeight, clientHeight } = body;
    if (scrollHeight - (scrollTop + clientHeight) < 120) void load(false);
  }
</script>

<div class={cn('flex min-h-0 flex-col gap-3', className)}>
  <div class="flex items-center gap-2">
    <div class="relative flex-1">
      <Search
        class="pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2 text-faint"
      />
      <Input bind:value={search} placeholder="Filter files" class="h-8 pl-7" />
    </div>
    <div class="flex items-center gap-2">
      <Switch id="models-show-all" bind:checked={showAll} />
      <Label for="models-show-all" class="text-xs text-muted-foreground">All GGUFs</Label>
    </div>
    <Button
      variant="ghost"
      size="icon-sm"
      aria-label="Rescan"
      title="Rescan"
      onclick={() => load(true)}
    >
      <RefreshCw class={loading ? 'animate-spin' : ''} />
    </Button>
  </div>

  <div
    bind:this={body}
    onscroll={onScroll}
    class="min-h-0 flex-1 overflow-y-auto rounded-lg border bg-card"
  >
    {#if !directory.trim()}
      <p class="p-8 text-center text-xs text-faint">
        Set a model directory in Settings to scan for local models.
      </p>
    {:else if loading}
      <p class="p-8 text-center text-xs text-faint">Scanning {directory}…</p>
    {:else if error}
      <p class="p-8 text-center text-xs text-destructive">{error}</p>
    {:else if filtered.length === 0}
      <p class="p-8 text-center text-xs text-faint">
        {search ? 'No files match your filter.' : 'No matching .gguf files in this directory.'}
      </p>
    {:else}
      <ul class="divide-y">
        {#each filtered as m (m.path)}
          {@const quant = quantOf(m.name)}
          <li class="group flex items-center gap-3 px-3 py-2 hover:bg-accent/50">
            <svelte:element
              this={onPick ? 'button' : 'div'}
              type={onPick ? 'button' : undefined}
              role={onPick ? undefined : 'presentation'}
              class="flex min-w-0 flex-1 items-center gap-3 text-left"
              onclick={onPick ? () => onPick(m) : undefined}
            >
              <div
                class="grid size-8 shrink-0 place-items-center rounded-md bg-muted text-muted-foreground"
              >
                {#if m.kind === 'Mmproj'}<Eye class="size-4" />{:else}<Box class="size-4" />{/if}
              </div>
              <div class="min-w-0 flex-1">
                <div class="truncate font-medium" title={m.name}>{m.name}</div>
                <div class="truncate font-mono text-[11px] text-faint" title={m.path}>
                  {m.path}
                </div>
              </div>
              <div
                class="flex shrink-0 items-center gap-3 font-mono text-[11px] text-muted-foreground"
              >
                {#if m.kind === 'Mmproj'}<span class="text-primary">mmproj</span>{/if}
                {#if quant}<span>{quant}</span>{/if}
                <span class="w-16 text-right">{m.size_bytes ? formatBytes(m.size_bytes) : '—'}</span
                >
              </div>
            </svelte:element>
            {#if actions}
              <div class="flex shrink-0 items-center gap-1">{@render actions(m)}</div>
            {/if}
          </li>
        {/each}
      </ul>
      {#if loadingMore}
        <p class="p-3 text-center text-xs text-faint">Loading more…</p>
      {:else if total > models.length}
        <p class="p-3 text-center text-xs text-faint">{models.length} of {total}</p>
      {/if}
    {/if}
  </div>
</div>
