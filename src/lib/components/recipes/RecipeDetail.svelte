<script lang="ts">
  import CopyIcon from '@lucide/svelte/icons/copy';
  import Ellipsis from '@lucide/svelte/icons/ellipsis';
  import ExternalLink from '@lucide/svelte/icons/external-link';
  import FileUp from '@lucide/svelte/icons/file-up';
  import Pencil from '@lucide/svelte/icons/pencil';
  import Pin from '@lucide/svelte/icons/pin';
  import PinOff from '@lucide/svelte/icons/pin-off';
  import Play from '@lucide/svelte/icons/play';
  import Square from '@lucide/svelte/icons/square';
  import Trash from '@lucide/svelte/icons/trash-2';
  import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
  import X from '@lucide/svelte/icons/x';

  import LiveStatsPanel from './LiveStatsPanel.svelte';
  import LogsPanel from './LogsPanel.svelte';
  import RecipeOverview from './RecipeOverview.svelte';
  import StatusDot from '$lib/components/common/StatusDot.svelte';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
  import * as Tabs from '$lib/components/ui/tabs';
  import { appStore, type RecipeTab } from '$lib/stores/app.svelte';
  import { serverStore } from '$lib/stores/server.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import type { Recipe } from '$lib/types';
  import { formatDate } from '$lib/utils/format';
  import { formatUptime } from '$lib/utils/server';

  let { recipe }: { recipe: Recipe } = $props();

  const isReady = $derived(serverStore.isReady(recipe.id));
  const isStarting = $derived(serverStore.isStarting(recipe.id));
  const isStopping = $derived(serverStore.isStopping(recipe.id));
  const isActive = $derived(serverStore.isActive(recipe.id));
  const blockedByOther = $derived(serverStore.anyActive() && !isActive);
  const tags = $derived(
    recipe.tags
      .split(',')
      .map((t) => t.trim())
      .filter(Boolean),
  );
  const logCount = $derived(serverStore.logs.filter((l) => l.recipe_id === recipe.id).length);

  const crash = $derived.by(() => {
    const exit = serverStore.lastExit;
    if (!exit || exit.intentional || exit.recipe_id !== recipe.id || isActive) return null;
    const parts: string[] = [];
    if (exit.code !== null) parts.push(`exit ${exit.code}`);
    if (exit.signal !== null) parts.push(`signal ${exit.signal}`);
    return parts.length ? parts.join(' · ') : 'unknown reason';
  });

  $effect(() => {
    if (!isReady && appStore.tab === 'stats') appStore.tab = 'overview';
  });

  let now = $state(Date.now());
  $effect(() => {
    if (!isReady || serverStore.startedAt === null) return;
    now = Date.now();
    const t = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(t);
  });
  const uptime = $derived(
    isReady && serverStore.startedAt !== null ? formatUptime(now - serverStore.startedAt) : '',
  );

  const status = $derived.by(() => {
    const pid = serverStore.status?.pid;
    if (isStopping)
      return { dot: 'warming', label: 'Stopping', meta: 'draining requests' } as const;
    if (isReady)
      return { dot: 'ready', label: 'Ready', meta: `${uptime} · PID ${pid ?? '—'}` } as const;
    if (isStarting)
      return { dot: 'warming', label: 'Starting', meta: 'waiting for /health' } as const;
    if (blockedByOther)
      return { dot: 'idle', label: 'Another server is running', meta: '' } as const;
    return {
      dot: 'idle',
      label: 'Stopped',
      meta: `updated ${formatDate(recipe.updated_at)}`,
    } as const;
  });
</script>

<div class="flex min-h-0 flex-1 flex-col overflow-y-auto">
  <div class="mx-auto flex w-full max-w-[1200px] flex-1 flex-col gap-5 px-8 pt-6 pb-8">
    {#if crash}
      <div
        role="alert"
        class="flex items-center gap-3 rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-2.5 text-sm"
      >
        <TriangleAlert class="size-4 shrink-0 text-destructive" />
        <div class="flex-1">
          <span class="font-medium text-destructive">Server crashed</span>
          <span class="text-muted-foreground">
            — llama-server exited unexpectedly ({crash}).
          </span>
          <Button variant="link" size="xs" class="px-1" onclick={() => (appStore.tab = 'logs')}>
            View logs
          </Button>
        </div>
        <Button
          variant="ghost"
          size="icon-xs"
          aria-label="Dismiss"
          onclick={() => serverStore.dismissExit()}
        >
          <X />
        </Button>
      </div>
    {/if}

    <header class="flex items-start gap-4">
      <div class="min-w-0 flex-1">
        <h1 class="truncate text-xl font-semibold tracking-tight">{recipe.name}</h1>
        {#if recipe.description}
          <p class="mt-1 line-clamp-2 text-muted-foreground">{recipe.description}</p>
        {/if}
        <div class="mt-2.5 flex flex-wrap items-center gap-1.5">
          <Badge variant="outline" class="gap-1.5">
            <StatusDot state={status.dot} />
            {status.label}
            {#if status.meta}<span class="font-normal text-faint">· {status.meta}</span>{/if}
          </Badge>
          {#if recipe.gpu_info}
            <Badge variant="secondary" class="font-mono">{recipe.gpu_info}</Badge>
          {/if}
          {#if recipe.mmproj_path}
            <Badge variant="secondary">vision</Badge>
          {/if}
          {#each tags as tag (tag)}
            <Badge variant="outline" class="text-muted-foreground">{tag}</Badge>
          {/each}
        </div>
      </div>

      <div class="flex shrink-0 items-center gap-2">
        {#if isReady && settingsStore.current.webui_enabled}
          <Button variant="outline" onclick={() => appStore.openWebUi()}>
            <ExternalLink /> Open WebUI
          </Button>
        {/if}
        <Button variant="outline" onclick={() => appStore.startEdit()} disabled={isActive}>
          <Pencil /> Edit
        </Button>
        {#if isActive}
          <Button variant="destructive" onclick={() => appStore.stop()} disabled={isStopping}>
            <Square />
            {isStopping ? 'Stopping…' : isStarting ? 'Cancel' : 'Stop'}
          </Button>
        {:else}
          <Button onclick={() => appStore.start(recipe.id)} disabled={blockedByOther}>
            <Play /> Start
          </Button>
        {/if}
        <DropdownMenu.Root>
          <DropdownMenu.Trigger>
            {#snippet child({ props })}
              <Button {...props} variant="ghost" size="icon" aria-label="More actions">
                <Ellipsis />
              </Button>
            {/snippet}
          </DropdownMenu.Trigger>
          <DropdownMenu.Content align="end" class="w-44">
            <DropdownMenu.Item onclick={() => appStore.togglePin(recipe)}>
              {#if recipe.pinned}<PinOff /> Unpin{:else}<Pin /> Pin to top{/if}
            </DropdownMenu.Item>
            <DropdownMenu.Item onclick={() => appStore.duplicate(recipe)}>
              <CopyIcon /> Duplicate
            </DropdownMenu.Item>
            <DropdownMenu.Item onclick={() => appStore.exportRecipe(recipe)}>
              <FileUp /> Export .ripe…
            </DropdownMenu.Item>
            <DropdownMenu.Item onclick={() => appStore.exportRecipeIni(recipe)}>
              <FileUp /> Export as INI preset…
            </DropdownMenu.Item>
            <DropdownMenu.Separator />
            <DropdownMenu.Item
              variant="destructive"
              disabled={isActive}
              onclick={() => appStore.remove(recipe)}
            >
              <Trash /> Delete…
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Root>
      </div>
    </header>

    <Tabs.Root
      value={appStore.tab}
      onValueChange={(v) => (appStore.tab = v as RecipeTab)}
      class="flex min-h-0 flex-1 flex-col gap-4"
    >
      <Tabs.List variant="line" class="h-9 w-full justify-start gap-4 rounded-none border-b px-0">
        <Tabs.Trigger value="overview" class="flex-none px-0.5 after:bg-primary">
          Overview
        </Tabs.Trigger>
        <Tabs.Trigger
          value="stats"
          class="flex-none px-0.5 after:bg-primary"
          disabled={!isReady}
          title={isReady ? undefined : 'Available while the server is running'}
        >
          Live stats
        </Tabs.Trigger>
        <Tabs.Trigger value="logs" class="flex-none px-0.5 after:bg-primary">
          Logs
          {#if logCount}
            <span class="rounded bg-muted px-1.5 font-mono text-[10px] text-muted-foreground">
              {logCount > 999 ? '999+' : logCount}
            </span>
          {/if}
        </Tabs.Trigger>
      </Tabs.List>
      <Tabs.Content value="overview">
        <RecipeOverview {recipe} />
      </Tabs.Content>
      <Tabs.Content value="stats">
        {#if isReady}<LiveStatsPanel />{/if}
      </Tabs.Content>
      <Tabs.Content value="logs" class="flex min-h-[420px] flex-1 flex-col">
        <LogsPanel recipeId={recipe.id} />
      </Tabs.Content>
    </Tabs.Root>
  </div>
</div>
