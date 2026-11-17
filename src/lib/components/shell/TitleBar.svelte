<script lang="ts">
  import ChevronRight from '@lucide/svelte/icons/chevron-right';
  import Download from '@lucide/svelte/icons/download';
  import Search from '@lucide/svelte/icons/search';

  import StatusDot from '$lib/components/common/StatusDot.svelte';
  import { Kbd } from '$lib/components/ui/kbd';
  import { appStore } from '$lib/stores/app.svelte';
  import { recipesStore } from '$lib/stores/recipes.svelte';
  import { serverStore } from '$lib/stores/server.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { updaterStore } from '$lib/stores/updater.svelte';
  import { cn } from '$lib/utils/cn';
  import { isMac, modKey } from '$lib/utils/platform';

  const SECTION_LABELS = {
    recipes: 'Recipes',
    models: 'Models',
    community: 'Community',
    settings: 'Settings',
  } as const;

  const crumbs = $derived.by(() => {
    const out: string[] = [SECTION_LABELS[appStore.section]];
    if (appStore.section !== 'recipes') return out;
    if (appStore.view === 'new') out.push('New recipe');
    else if (recipesStore.selected) {
      out.push(recipesStore.selected.name);
      if (appStore.view === 'edit') out.push('Edit');
    }
    return out;
  });

  const status = $derived.by(() => {
    const port = settingsStore.current.port;
    switch (serverStore.phase) {
      case 'ready':
        return { dot: 'ready', label: `Running on :${port}` } as const;
      case 'starting':
        return { dot: 'warming', label: 'Starting…' } as const;
      case 'stopping':
        return { dot: 'warming', label: 'Stopping…' } as const;
      default:
        return { dot: 'idle', label: 'Server idle' } as const;
    }
  });

  const runningName = $derived(
    recipesStore.items.find((r) => r.id === serverStore.runningRecipeId())?.name ?? null,
  );
</script>

<header
  data-tauri-drag-region
  class={cn(
    'relative flex h-11 shrink-0 items-center gap-3 border-b bg-sidebar pr-3',
    isMac ? 'pl-[84px]' : 'pl-3',
  )}
>
  <nav
    class="pointer-events-none flex min-w-0 items-center gap-1 text-[13px] text-muted-foreground"
  >
    {#each crumbs as crumb, i (i)}
      {#if i > 0}<ChevronRight class="size-3.5 shrink-0 text-faint" />{/if}
      <span class={cn('truncate', i === crumbs.length - 1 && 'font-medium text-foreground')}>
        {crumb}
      </span>
    {/each}
  </nav>

  <button
    type="button"
    class="absolute left-1/2 flex h-7 w-[min(360px,36vw)] -translate-x-1/2 items-center gap-2 rounded-md border bg-background/60 px-2.5 text-muted-foreground transition-colors hover:border-input hover:text-foreground"
    onclick={() => (appStore.paletteOpen = true)}
  >
    <Search class="size-3.5" />
    <span class="flex-1 text-left text-xs">Search recipes, models, actions…</span>
    <Kbd>{modKey}K</Kbd>
  </button>

  <div class="ml-auto flex items-center gap-2">
    {#if updaterStore.status === 'available' || updaterStore.status === 'downloading'}
      <button
        type="button"
        class="flex h-7 items-center gap-1.5 rounded-md px-2 text-xs font-medium text-primary hover:bg-primary/10"
        onclick={() => (appStore.aboutOpen = true)}
      >
        <Download class="size-3.5" />
        {updaterStore.status === 'downloading' ? 'Updating…' : 'Update'}
      </button>
    {/if}
    <button
      type="button"
      class="flex h-7 items-center gap-2 rounded-md px-2 text-xs text-muted-foreground hover:bg-accent hover:text-foreground disabled:pointer-events-none"
      disabled={!runningName}
      title={runningName ? `Go to ${runningName}` : undefined}
      onclick={() => appStore.jumpToRunning()}
    >
      <StatusDot state={status.dot} />
      <span>{status.label}</span>
    </button>
  </div>
</header>
