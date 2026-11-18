<script lang="ts">
  import { Menu } from '@tauri-apps/api/menu';
  import Pin from '@lucide/svelte/icons/pin';

  import StatusDot from '$lib/components/common/StatusDot.svelte';
  import { appStore } from '$lib/stores/app.svelte';
  import { serverStore } from '$lib/stores/server.svelte';
  import type { Recipe } from '$lib/types';
  import { cn } from '$lib/utils/cn';
  import { recipeMeta } from '$lib/utils/model';

  let { recipe, active }: { recipe: Recipe; active: boolean } = $props();

  const dot = $derived(
    serverStore.isReady(recipe.id) ? 'ready' : serverStore.isActive(recipe.id) ? 'warming' : null,
  );
  const meta = $derived(recipeMeta(recipe));

  async function showContextMenu(e: MouseEvent) {
    e.preventDefault();
    const menu = await Menu.new({
      items: [
        {
          id: 'pin',
          text: recipe.pinned ? 'Unpin' : 'Pin to top',
          action: () => appStore.togglePin(recipe),
        },
        { id: 'duplicate', text: 'Duplicate', action: () => appStore.duplicate(recipe) },
        { id: 'export', text: 'Export .ripe…', action: () => appStore.exportRecipe(recipe) },
        {
          id: 'export-ini',
          text: 'Export as INI preset…',
          action: () => appStore.exportRecipeIni(recipe),
        },
        { item: 'Separator' },
        { id: 'delete', text: 'Delete…', action: () => appStore.remove(recipe) },
      ],
    });
    await menu.popup();
  }
</script>

<button
  type="button"
  class={cn(
    'group flex w-full flex-col gap-0.5 rounded-md px-2.5 py-2 text-left transition-colors hover:bg-accent/70',
    active && 'bg-accent hover:bg-accent',
  )}
  aria-current={active ? 'true' : undefined}
  onclick={() => appStore.selectRecipe(recipe.id)}
  oncontextmenu={showContextMenu}
>
  <span class="flex items-center gap-1.5">
    <span class="truncate font-medium">{recipe.name}</span>
    {#if recipe.pinned}
      <Pin class="size-3 shrink-0 text-faint" aria-label="Pinned" />
    {/if}
    {#if dot}
      <StatusDot state={dot} class="ml-auto" />
    {/if}
  </span>
  {#if meta}
    <span class="truncate font-mono text-[11px] text-faint">{meta}</span>
  {/if}
</button>
