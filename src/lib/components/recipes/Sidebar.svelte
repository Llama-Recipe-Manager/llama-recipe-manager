<script lang="ts">
  import ArrowDownUp from '@lucide/svelte/icons/arrow-down-up';
  import FileDown from '@lucide/svelte/icons/file-down';
  import FileUp from '@lucide/svelte/icons/file-up';
  import Plus from '@lucide/svelte/icons/plus';
  import Search from '@lucide/svelte/icons/search';

  import RecipeListItem from './RecipeListItem.svelte';
  import { Button } from '$lib/components/ui/button';
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
  import { Input } from '$lib/components/ui/input';
  import * as Tooltip from '$lib/components/ui/tooltip';
  import { appStore } from '$lib/stores/app.svelte';
  import { recipesStore } from '$lib/stores/recipes.svelte';
  import { modKey } from '$lib/utils/platform';
  import { RECIPE_SORTS, isRecipeSort } from '$lib/utils/recipeSort';

  let query = $state('');

  const filtered = $derived(recipesStore.filter(query));
  const pinned = $derived(filtered.filter((r) => r.pinned));
  const rest = $derived(filtered.filter((r) => !r.pinned));
  const activeId = $derived(appStore.view === 'detail' ? recipesStore.selectedId : null);
  const sortLabel = $derived(RECIPE_SORTS.find((s) => s.value === recipesStore.sort)?.label);
</script>

<aside class="flex w-[268px] shrink-0 flex-col border-r bg-sidebar">
  <div class="flex items-center gap-1 px-3 pt-3 pb-2">
    <h2 class="flex-1 text-sm font-semibold">Recipes</h2>
    <Tooltip.Root>
      <Tooltip.Trigger>
        {#snippet child({ props })}
          <Button
            {...props}
            variant="ghost"
            size="icon-sm"
            aria-label="Import recipe"
            onclick={() => appStore.importRecipe()}
          >
            <FileDown />
          </Button>
        {/snippet}
      </Tooltip.Trigger>
      <Tooltip.Content>Import .ripe file</Tooltip.Content>
    </Tooltip.Root>
    <Tooltip.Root>
      <Tooltip.Trigger>
        {#snippet child({ props })}
          <Button
            {...props}
            variant="ghost"
            size="icon-sm"
            aria-label="Export all recipes as INI"
            onclick={() => appStore.exportAllRecipesIni()}
          >
            <FileUp />
          </Button>
        {/snippet}
      </Tooltip.Trigger>
      <Tooltip.Content>Export all as INI preset file</Tooltip.Content>
    </Tooltip.Root>
    <Tooltip.Root>
      <Tooltip.Trigger>
        {#snippet child({ props })}
          <Button
            {...props}
            variant="ghost"
            size="icon-sm"
            aria-label="New recipe"
            onclick={() => appStore.startNew()}
          >
            <Plus />
          </Button>
        {/snippet}
      </Tooltip.Trigger>
      <Tooltip.Content>New recipe <span class="ml-1 opacity-60">{modKey}N</span></Tooltip.Content>
    </Tooltip.Root>
  </div>

  <div class="flex items-center gap-1.5 px-3 pb-2">
    <div class="relative flex-1">
      <Search
        class="pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2 text-faint"
      />
      <Input
        id="recipe-search"
        bind:value={query}
        placeholder="Filter recipes"
        class="h-7 pl-7 text-xs"
        aria-keyshortcuts="Meta+F Control+F"
      />
    </div>
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        {#snippet child({ props })}
          <Button
            {...props}
            variant="ghost"
            size="icon-sm"
            aria-label="Sort recipes"
            title="Sort: {sortLabel}"
          >
            <ArrowDownUp />
          </Button>
        {/snippet}
      </DropdownMenu.Trigger>
      <DropdownMenu.Content align="end" class="w-44">
        <DropdownMenu.Label>Sort by</DropdownMenu.Label>
        <DropdownMenu.RadioGroup
          value={recipesStore.sort}
          onValueChange={(v) => isRecipeSort(v) && recipesStore.setSort(v)}
        >
          {#each RECIPE_SORTS as option (option.value)}
            <DropdownMenu.RadioItem value={option.value}>{option.label}</DropdownMenu.RadioItem>
          {/each}
        </DropdownMenu.RadioGroup>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  </div>

  <div class="flex-1 overflow-y-auto px-2 pb-3">
    {#if recipesStore.items.length === 0}
      <p class="px-2.5 py-6 text-center text-xs text-faint">No recipes yet.</p>
    {:else if filtered.length === 0}
      <p class="px-2.5 py-6 text-center text-xs text-faint">No recipes match “{query}”.</p>
    {/if}
    {#if pinned.length}
      <h3 class="px-2.5 pt-2 pb-1 text-[10.5px] font-semibold tracking-wider text-faint uppercase">
        Pinned
      </h3>
      {#each pinned as recipe (recipe.id)}
        <RecipeListItem {recipe} active={activeId === recipe.id} />
      {/each}
    {/if}
    {#if rest.length}
      {#if pinned.length}
        <h3
          class="px-2.5 pt-3 pb-1 text-[10.5px] font-semibold tracking-wider text-faint uppercase"
        >
          All recipes
        </h3>
      {/if}
      {#each rest as recipe (recipe.id)}
        <RecipeListItem {recipe} active={activeId === recipe.id} />
      {/each}
    {/if}
  </div>
</aside>
