<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { ask } from '@tauri-apps/plugin-dialog';

  import AboutDialog from '$lib/components/AboutDialog.svelte';
  import CommunitySidebar from '$lib/components/CommunitySidebar.svelte';
  import CommunityView from '$lib/components/CommunityView.svelte';
  import NavRail from '$lib/components/NavRail.svelte';
  import { FEATURES } from '$lib/featureFlags';
  import RecipeDetail from '$lib/components/RecipeDetail.svelte';
  import RecipeForm from '$lib/components/RecipeForm.svelte';
  import SettingsForm from '$lib/components/SettingsForm.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import UpdateBanner from '$lib/components/UpdateBanner.svelte';
  import { recipesStore } from '$lib/stores/recipes.svelte';
  import { serverStore } from '$lib/stores/server.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { updaterStore } from '$lib/stores/updater.svelte';
  import type { CommunityFilters, Section, View } from '$lib/types';
  import { errorMessage } from '$lib/utils/format';

  let section = $state<Section>('recipes');
  let view = $state<View>('list');
  let searchQuery = $state('');
  let error = $state('');
  let aboutOpen = $state(false);

  let communityFilters = $state<CommunityFilters>({
    search: '',
    backend: 'any',
    capability: 'any',
    vramMinGib: null,
    sort: 'recent',
  });

  const selected = $derived(recipesStore.selected);
  const isServerRunning = $derived(serverStore.anyRunning());

  let unlistenClose: (() => void) | null = null;

  onMount(async () => {
    try {
      await Promise.all([recipesStore.refresh(), serverStore.refresh(), settingsStore.refresh()]);
    } catch (e) {
      error = errorMessage(e);
    }
    await serverStore.subscribe();

    // Fire-and-forget update check — failures are surfaced inside the store
    // and the rest of the app keeps working if the updater is unconfigured.
    void updaterStore.check();

    // Intercept the window close so we can ask the user before tearing down
    // a running server (or cancelling a still-starting one). If they say no
    // we keep the window open; if they say yes we let the close proceed,
    // and the Rust-side ExitRequested handler does the actual cleanup.
    const win = getCurrentWindow();
    unlistenClose = await win.onCloseRequested(async (event) => {
      if (!serverStore.anyActive()) return;
      event.preventDefault();
      const starting = serverStore.phase === 'starting';
      const keep = settingsStore.current.keep_server_on_exit;
      const message = starting
        ? 'llama-server is still starting. Quit anyway and cancel the launch?'
        : keep
          ? 'Quit Llama Recipe Manager? llama-server will keep running in the background (per your settings).'
          : 'llama-server is running. Quit Llama Recipe Manager? The server will be stopped.';
      const confirmed = await ask(message, {
        title: 'Quit Llama Recipe Manager',
        kind: 'warning',
      });
      if (!confirmed) return;
      // Tear down our own listener first: a `close()` retry below would
      // otherwise re-enter this handler. `destroy()` is the preferred path
      // (skips CloseRequested entirely) but if it ever fails — capability
      // missing, plugin error — fall back to a plain close that now goes
      // through unimpeded.
      unlistenClose?.();
      unlistenClose = null;
      try {
        await win.destroy();
      } catch (e) {
        error = `Couldn't close window: ${errorMessage(e)}`;
        try {
          await win.close();
        } catch {
          /* nothing more we can do — error is already surfaced */
        }
      }
    });
  });

  onDestroy(() => {
    serverStore.unsubscribe();
    unlistenClose?.();
  });

  function goSection(next: Section) {
    section = next;
    error = '';
    if (next === 'recipes') {
      view = 'list';
    }
  }

  function jumpToRunningRecipe() {
    const runningId = serverStore.runningRecipeId();
    if (!runningId) return;
    section = 'recipes';
    view = 'list';
    recipesStore.select(runningId);
  }

  function selectRecipe(id: string) {
    recipesStore.select(id);
    view = 'list';
    serverStore.resetLogView();
  }

  function startNew() {
    recipesStore.select(null);
    view = 'new';
  }

  function startEdit() {
    if (selected) view = 'edit';
  }

  function cancelEdit() {
    view = 'list';
  }

  async function handleStart() {
    if (!selected) return;
    error = '';
    try {
      await serverStore.start(
        selected.id,
        selected.command,
        selected.model_path,
        selected.mmproj_path,
      );
    } catch (e) {
      error = errorMessage(e);
    }
  }

  async function handleStop() {
    error = '';
    if (!serverStore.anyActive()) return;
    const starting = serverStore.phase === 'starting';
    const confirmed = await ask(
      starting
        ? 'Cancel starting llama-server? The process will be terminated.'
        : 'Stop the running llama-server? It will be asked to drain in-flight requests, then terminated if it does not exit within 8 seconds.',
      { title: starting ? 'Cancel start' : 'Stop server', kind: 'warning' },
    );
    if (!confirmed) return;
    try {
      await serverStore.stop();
    } catch (e) {
      error = errorMessage(e);
    }
  }

  function setError(msg: string) {
    error = msg;
  }

  async function onSaved(id: string) {
    recipesStore.select(id);
    view = 'list';
  }

  function onDeleted() {
    recipesStore.select(null);
  }

  function onDuplicated(id: string) {
    recipesStore.select(id);
  }

  $effect(() => {
    if (section === 'settings') {
      settingsStore.checkLlamaPath();
    }
  });
</script>

<div class="app-shell">
<UpdateBanner />
<div class="app">
  <NavRail
    {section}
    serverRunning={isServerRunning}
    onSelect={goSection}
    onJumpToRunning={jumpToRunningRecipe}
    onAbout={() => (aboutOpen = true)}
  />

  {#if section === 'recipes'}
    <Sidebar
      bind:searchQuery
      activeId={view === 'list' ? recipesStore.selectedId : null}
      onSelect={selectRecipe}
      onNew={startNew}
      onStop={handleStop}
    />
  {:else if section === 'community' && FEATURES.community}
    <CommunitySidebar bind:filters={communityFilters} />
  {/if}

  <main class="content">
    {#if error}
      <div class="error-bar">
        <span>{error}</span>
        <button onclick={() => (error = '')}>Dismiss</button>
      </div>
    {/if}

    {#if section === 'settings'}
      <SettingsForm onBack={() => goSection('recipes')} onError={setError} />
    {:else if section === 'community' && FEATURES.community}
      <CommunityView filters={communityFilters} />
    {:else if view === 'new'}
      <RecipeForm mode="new" onCancel={cancelEdit} {onSaved} onError={setError} />
    {:else if view === 'edit' && selected}
      <RecipeForm
        mode="edit"
        initial={selected}
        onCancel={cancelEdit}
        {onSaved}
        onError={setError}
      />
    {:else if selected}
      <RecipeDetail
        recipe={selected}
        onEdit={startEdit}
        onStart={handleStart}
        onStop={handleStop}
        {onDeleted}
        {onDuplicated}
        onError={setError}
      />
    {:else}
      <div class="empty-state">
        <div class="empty-icon">
          <svg
            width="48"
            height="48"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
          >
            <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" /><polyline
              points="14,2 14,8 20,8"
            /><line x1="12" y1="18" x2="12" y2="12" /><line x1="9" y1="15" x2="15" y2="15" />
          </svg>
        </div>
        <h2>No recipe selected</h2>
        <p>Select a recipe from the sidebar or create a new one to get started.</p>
        <button class="btn primary" onclick={startNew}>Create Recipe</button>
      </div>
    {/if}
  </main>
</div>
</div>

<AboutDialog open={aboutOpen} onClose={() => (aboutOpen = false)} onError={setError} />

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .app {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .error-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 20px;
    background: var(--danger);
    color: white;
    font-size: 13px;
  }

  .error-bar button {
    color: white;
    text-decoration: underline;
    font-size: 13px;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--text-secondary);
  }

  .empty-icon {
    color: var(--text-tertiary);
    opacity: 0.5;
    margin-bottom: 8px;
  }

  .empty-state h2 {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .empty-state p {
    font-size: 14px;
    max-width: 300px;
    text-align: center;
  }

  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 7px 16px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 500;
    transition: all 0.15s;
    white-space: nowrap;
  }

  .btn.primary {
    background: var(--accent);
    color: white;
  }

  .btn.primary:hover {
    background: var(--accent-hover);
  }
</style>
