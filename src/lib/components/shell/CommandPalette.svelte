<script lang="ts">
  import BookOpen from '@lucide/svelte/icons/book-open';
  import Boxes from '@lucide/svelte/icons/boxes';
  import ExternalLink from '@lucide/svelte/icons/external-link';
  import FileDown from '@lucide/svelte/icons/file-down';
  import FileUp from '@lucide/svelte/icons/file-up';
  import Info from '@lucide/svelte/icons/info';
  import Pencil from '@lucide/svelte/icons/pencil';
  import Play from '@lucide/svelte/icons/play';
  import Plus from '@lucide/svelte/icons/plus';
  import ScrollText from '@lucide/svelte/icons/scroll-text';
  import Settings from '@lucide/svelte/icons/settings';
  import Square from '@lucide/svelte/icons/square';

  import StatusDot from '$lib/components/common/StatusDot.svelte';
  import * as Command from '$lib/components/ui/command';
  import { appStore } from '$lib/stores/app.svelte';
  import { recipesStore } from '$lib/stores/recipes.svelte';
  import { serverStore } from '$lib/stores/server.svelte';
  import { modKey } from '$lib/utils/platform';
  import { recipeMeta } from '$lib/utils/model';

  const selected = $derived(recipesStore.selected);
  const active = $derived(serverStore.anyActive());
  const ready = $derived(serverStore.phase === 'ready');

  function run(fn: () => unknown) {
    appStore.paletteOpen = false;
    void fn();
  }
</script>

<Command.Dialog bind:open={appStore.paletteOpen} class="sm:max-w-xl">
  <Command.Input placeholder="Type a command or search recipes…" />
  <Command.List class="max-h-[420px]">
    <Command.Empty>No results.</Command.Empty>
    <Command.Group heading="Actions">
      <Command.Item value="new recipe" onSelect={() => run(() => appStore.startNew())}>
        <Plus /> New recipe <Command.Shortcut>{modKey}N</Command.Shortcut>
      </Command.Item>
      <Command.Item value="import recipe ripe" onSelect={() => run(() => appStore.importRecipe())}>
        <FileDown /> Import recipe…
      </Command.Item>
      <Command.Item
        value="export all recipes ini"
        onSelect={() => run(() => appStore.exportAllRecipesIni())}
      >
        <FileUp /> Export all recipes as INI…
      </Command.Item>
      {#if active}
        <Command.Item value="stop server" onSelect={() => run(() => appStore.stop())}>
          <Square /> Stop server
        </Command.Item>
      {:else if selected}
        <Command.Item
          value="start server {selected.name}"
          onSelect={() => run(() => appStore.start(selected.id))}
        >
          <Play /> Start {selected.name}
        </Command.Item>
      {/if}
      {#if ready}
        <Command.Item value="open webui browser" onSelect={() => run(() => appStore.openWebUi())}>
          <ExternalLink /> Open WebUI
        </Command.Item>
        <Command.Item
          value="show server logs"
          onSelect={() =>
            run(async () => {
              const id = serverStore.runningRecipeId();
              if (id) await appStore.selectRecipe(id, 'logs');
            })}
        >
          <ScrollText /> Show server logs
        </Command.Item>
      {/if}
      {#if selected && appStore.view === 'detail' && !serverStore.isActive(selected.id)}
        <Command.Item value="edit {selected.name}" onSelect={() => run(() => appStore.startEdit())}>
          <Pencil /> Edit {selected.name}
        </Command.Item>
      {/if}
    </Command.Group>
    <Command.Separator />
    <Command.Group heading="Go to">
      <Command.Item value="go recipes" onSelect={() => run(() => appStore.goSection('recipes'))}>
        <BookOpen /> Recipes
      </Command.Item>
      <Command.Item value="go models" onSelect={() => run(() => appStore.goSection('models'))}>
        <Boxes /> Models
      </Command.Item>
      <Command.Item value="go settings" onSelect={() => run(() => appStore.goSection('settings'))}>
        <Settings /> Settings
      </Command.Item>
      <Command.Item value="about updates" onSelect={() => run(() => (appStore.aboutOpen = true))}>
        <Info /> About &amp; updates
      </Command.Item>
    </Command.Group>
    {#if recipesStore.items.length}
      <Command.Separator />
      <Command.Group heading="Recipes">
        {#each recipesStore.items as recipe (recipe.id)}
          <Command.Item
            value="recipe {recipe.name} {recipe.tags} {recipe.id}"
            onSelect={() => run(() => appStore.selectRecipe(recipe.id))}
          >
            <StatusDot
              state={serverStore.isReady(recipe.id)
                ? 'ready'
                : serverStore.isActive(recipe.id)
                  ? 'warming'
                  : 'idle'}
              class="mx-1"
            />
            <span class="truncate">{recipe.name}</span>
            <span class="ml-auto truncate font-mono text-[11px] text-faint">
              {recipeMeta(recipe)}
            </span>
          </Command.Item>
        {/each}
      </Command.Group>
    {/if}
  </Command.List>
</Command.Dialog>
