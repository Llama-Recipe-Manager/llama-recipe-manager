<script lang="ts">
  import Eraser from '@lucide/svelte/icons/eraser';
  import Search from '@lucide/svelte/icons/search';

  import CopyButton from '$lib/components/common/CopyButton.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Switch } from '$lib/components/ui/switch';
  import { Label } from '$lib/components/ui/label';
  import { confirm } from '$lib/stores/confirm.svelte';
  import { serverStore } from '$lib/stores/server.svelte';
  import { cn } from '$lib/utils/cn';
  import { logLevel } from '$lib/utils/logs';
  import { notifyError } from '$lib/utils/notify';

  let { recipeId }: { recipeId: string } = $props();

  let viewer = $state<HTMLDivElement | undefined>(undefined);
  let autoScroll = $state(true);
  let filter = $state('');

  const recipeLogs = $derived(serverStore.logs.filter((l) => l.recipe_id === recipeId));
  const filtered = $derived.by(() => {
    const q = filter.trim().toLowerCase();
    return q ? recipeLogs.filter((l) => l.line.toLowerCase().includes(q)) : recipeLogs;
  });
  const live = $derived(serverStore.isActive(recipeId));

  function onScroll() {
    if (!viewer) return;
    const { scrollTop, scrollHeight, clientHeight } = viewer;
    autoScroll = scrollHeight - (scrollTop + clientHeight) < 24;
  }

  $effect(() => {
    void filtered.length;
    if (autoScroll && viewer) {
      requestAnimationFrame(() => {
        if (viewer) viewer.scrollTop = viewer.scrollHeight;
      });
    }
  });

  async function clear() {
    const ok = await confirm({
      title: 'Clear logs?',
      message: 'All captured server output will be removed.',
      confirmLabel: 'Clear',
      destructive: true,
    });
    if (!ok) return;
    try {
      await serverStore.clearLogs();
    } catch (e) {
      notifyError(e);
    }
  }
</script>

<div class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border bg-card">
  <div class="flex items-center gap-2 border-b px-3 py-2">
    <div class="relative w-64">
      <Search
        class="pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2 text-faint"
      />
      <Input
        bind:value={filter}
        placeholder="Filter lines"
        class="h-7 pl-7 text-xs"
        spellcheck="false"
      />
    </div>
    <span class="text-xs text-faint">
      {filter.trim() ? `${filtered.length} of ` : ''}{recipeLogs.length}
      {recipeLogs.length === 1 ? 'line' : 'lines'}
    </span>
    <div class="ml-auto flex items-center gap-3">
      <div class="flex items-center gap-2">
        <Switch id="log-autoscroll" bind:checked={autoScroll} />
        <Label for="log-autoscroll" class="text-xs text-muted-foreground">Follow</Label>
      </div>
      <CopyButton
        text={filtered.map((l) => l.line).join('\n')}
        label={filter.trim() ? 'Copy matching' : 'Copy all'}
        showLabel
      />
      <Button variant="ghost" size="xs" onclick={clear} disabled={recipeLogs.length === 0}>
        <Eraser /> Clear
      </Button>
    </div>
  </div>

  <div
    bind:this={viewer}
    onscroll={onScroll}
    class="min-h-0 flex-1 overflow-auto bg-code px-3 py-2 font-mono text-[11.5px] leading-[1.6] select-text"
  >
    {#if recipeLogs.length === 0}
      <p class="py-10 text-center font-sans text-xs text-faint">
        {live ? 'Waiting for output…' : 'No logs yet. Start the server to see output.'}
      </p>
    {:else if filtered.length === 0}
      <p class="py-10 text-center font-sans text-xs text-faint">No lines match “{filter}”.</p>
    {:else}
      {#each filtered as log, i (i)}
        {@const level = logLevel(log.line)}
        <!-- prettier-ignore -->
        <div class={cn('break-all whitespace-pre-wrap', level === 'error' ? 'text-destructive' : level === 'warn' ? 'text-warning' : 'text-muted-foreground')}>{log.line}</div>
      {/each}
    {/if}
  </div>
</div>
