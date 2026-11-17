<script lang="ts">
  import { onDestroy, onMount, tick, untrack } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { ask } from '@tauri-apps/plugin-dialog';
  import FilePlus from '@lucide/svelte/icons/file-plus-2';
  import { toast } from 'svelte-sonner';

  import AboutDialog from '$lib/components/common/AboutDialog.svelte';
  import ConfirmHost from '$lib/components/common/ConfirmHost.svelte';
  import EmptyState from '$lib/components/common/EmptyState.svelte';
  import CommunitySidebar from '$lib/components/community/CommunitySidebar.svelte';
  import CommunityView from '$lib/components/community/CommunityView.svelte';
  import ModelsView from '$lib/components/models/ModelsView.svelte';
  import RecipeDetail from '$lib/components/recipes/RecipeDetail.svelte';
  import RecipeForm from '$lib/components/recipes/RecipeForm.svelte';
  import Sidebar from '$lib/components/recipes/Sidebar.svelte';
  import SettingsView from '$lib/components/settings/SettingsView.svelte';
  import CommandPalette from '$lib/components/shell/CommandPalette.svelte';
  import NavRail from '$lib/components/shell/NavRail.svelte';
  import TitleBar from '$lib/components/shell/TitleBar.svelte';
  import { Button } from '$lib/components/ui/button';
  import { cleanupOrphanParts } from '$lib/api/models';
  import { FEATURES } from '$lib/featureFlags';
  import { appStore } from '$lib/stores/app.svelte';
  import { liveStats } from '$lib/stores/liveStats.svelte';
  import { recipesStore } from '$lib/stores/recipes.svelte';
  import { serverStore } from '$lib/stores/server.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { themeStore } from '$lib/stores/theme.svelte';
  import { updaterStore } from '$lib/stores/updater.svelte';
  import { createTray, showMainWindow } from '$lib/tray';
  import type { CommunityFilters } from '$lib/types';
  import { errorMessage } from '$lib/utils/format';
  import { notifyError } from '$lib/utils/notify';
  import { trayStatus } from '$lib/utils/tray';

  let communityFilters = $state<CommunityFilters>({
    search: '',
    backend: 'any',
    capability: 'any',
    vramMinGib: null,
    sort: 'recent',
  });

  const selected = $derived(recipesStore.selected);

  let unlistenClose: (() => void) | null = null;
  let tray = $state.raw<Awaited<ReturnType<typeof createTray>> | null>(null);
  let lastStartedId = $state<string | null>(null);

  $effect(() => {
    const id = serverStore.runningRecipeId();
    if (id) lastStartedId = id;
  });

  $effect(() => {
    if (serverStore.phase !== 'ready') return;
    untrack(() => liveStats.start());
    return () => liveStats.stop();
  });

  $effect(() => {
    if (!tray) return;
    const runningId = serverStore.runningRecipeId();
    const phase = serverStore.phase;
    const targetId = runningId ?? lastStartedId ?? selected?.id ?? null;
    const target = recipesStore.items.find((r) => r.id === targetId) ?? null;
    tray.update({
      status: trayStatus(phase, target?.name ?? null),
      action: runningId
        ? {
            text: phase === 'starting' ? 'Cancel start' : 'Stop server',
            enabled: phase !== 'stopping',
            run: () => void fromTray(() => serverStore.stop()),
          }
        : target
          ? {
              text: `Start ${target.name}`,
              enabled: true,
              run: () => void fromTray(() => serverStore.start(target.id)),
            }
          : null,
    });
  });

  async function fromTray(fn: () => Promise<void>) {
    try {
      await fn();
    } catch (e) {
      notifyError(e);
      await showMainWindow();
    }
  }

  onMount(async () => {
    createTray()
      .then((t) => (tray = t))
      .catch(() => {});
    try {
      await Promise.all([recipesStore.refresh(), serverStore.refresh(), settingsStore.refresh()]);
    } catch (e) {
      notifyError(e);
    }
    if (settingsStore.current.model_dir.trim()) {
      cleanupOrphanParts(settingsStore.current.model_dir).catch(() => {});
    }
    await themeStore.subscribe();
    await serverStore.subscribe();
    void updaterStore.check().then(() => {
      if (updaterStore.status !== 'available' || !updaterStore.info) return;
      toast(`Update available — v${updaterStore.info.version}`, {
        duration: 10_000,
        action: { label: 'View', onClick: () => (appStore.aboutOpen = true) },
      });
    });

    // The Rust ExitRequested handler does the actual teardown once we let the close through.
    const win = getCurrentWindow();
    unlistenClose = await win.onCloseRequested(async (event) => {
      const unsaved = appStore.formDirty && appStore.editing;
      if (!serverStore.anyActive() && !unsaved) return;
      event.preventDefault();
      const starting = serverStore.phase === 'starting';
      const keep = settingsStore.current.keep_server_on_exit;
      const serverMessage = !serverStore.anyActive()
        ? 'Quit Llama Recipe Manager?'
        : starting
          ? 'llama-server is still starting. Quit anyway and cancel the launch?'
          : keep
            ? 'Quit Llama Recipe Manager? llama-server will keep running in the background (per your settings).'
            : 'llama-server is running. Quit Llama Recipe Manager? The server will be stopped.';
      const message = unsaved
        ? `You have unsaved recipe changes that will be lost.\n\n${serverMessage}`
        : serverMessage;
      const confirmed = await ask(message, { title: 'Quit Llama Recipe Manager', kind: 'warning' });
      if (!confirmed) return;
      unlistenClose?.();
      unlistenClose = null;
      try {
        await win.destroy();
      } catch (e) {
        notifyError(`Couldn't close window: ${errorMessage(e)}`);
        try {
          await win.close();
        } catch {
          /* already surfaced */
        }
      }
    });
  });

  onDestroy(() => {
    serverStore.unsubscribe();
    liveStats.stop();
    unlistenClose?.();
    void tray?.dispose();
  });

  async function handleShortcut(e: KeyboardEvent) {
    if (!(e.metaKey || e.ctrlKey) || e.altKey || e.shiftKey) return;
    const key = e.key.toLowerCase();
    if (key === 'k') {
      e.preventDefault();
      appStore.paletteOpen = !appStore.paletteOpen;
    } else if (key === 'n') {
      e.preventDefault();
      await appStore.startNew();
    } else if (key === 'f') {
      e.preventDefault();
      await appStore.goSection('recipes');
      await tick();
      document.querySelector<HTMLInputElement>('#recipe-search')?.focus();
    } else if (key === ',') {
      e.preventDefault();
      await appStore.goSection('settings');
    }
  }
</script>

<svelte:window onkeydown={handleShortcut} />

<div class="flex h-screen flex-col overflow-hidden">
  <TitleBar />
  <div class="flex min-h-0 flex-1">
    <NavRail />

    {#if appStore.section === 'recipes'}
      <Sidebar />
    {:else if appStore.section === 'community' && FEATURES.community}
      <CommunitySidebar bind:filters={communityFilters} />
    {/if}

    <main class="flex min-w-0 flex-1 flex-col overflow-hidden">
      {#if appStore.section === 'settings'}
        <SettingsView />
      {:else if appStore.section === 'models'}
        <ModelsView />
      {:else if appStore.section === 'community' && FEATURES.community}
        <CommunityView filters={communityFilters} />
      {:else if appStore.view === 'new'}
        <RecipeForm mode="new" initialModel={appStore.draftModel} />
      {:else if appStore.view === 'edit' && selected}
        {#key selected.id}
          <RecipeForm mode="edit" initial={selected} />
        {/key}
      {:else if selected}
        <RecipeDetail recipe={selected} />
      {:else}
        <EmptyState
          icon={FilePlus}
          title={recipesStore.items.length ? 'No recipe selected' : 'Create your first recipe'}
          description="A recipe pairs a GGUF model with the llama-server flags you want to run it with."
        >
          <Button onclick={() => appStore.startNew()}>New recipe</Button>
          <Button variant="outline" onclick={() => appStore.goSection('models')}>
            Browse models
          </Button>
        </EmptyState>
      {/if}
    </main>
  </div>
</div>

<CommandPalette />
<ConfirmHost />
<AboutDialog />
