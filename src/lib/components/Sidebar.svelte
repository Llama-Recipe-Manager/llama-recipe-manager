<script lang="ts">
  import RecipeListItem from './RecipeListItem.svelte';
  import ServerStatusBar from './ServerStatusBar.svelte';
  import { recipesStore } from '$lib/stores/recipes.svelte';
  import { serverStore } from '$lib/stores/server.svelte';

  let {
    searchQuery = $bindable(''),
    activeId,
    onSelect,
    onNew,
    onStop,
  }: {
    searchQuery: string;
    activeId: string | null;
    onSelect: (id: string) => void;
    onNew: () => void;
    onStop: () => void;
  } = $props();

  const filtered = $derived(recipesStore.filter(searchQuery));
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <h1 class="app-title">My Recipes</h1>
    <div class="sidebar-actions">
      <button class="btn-icon" onclick={onNew} title="New Recipe" aria-label="New Recipe">
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          ><line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" /></svg
        >
      </button>
    </div>
  </div>

  <div class="search-box">
    <svg
      class="search-icon"
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      ><circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" /></svg
    >
    <input
      type="text"
      placeholder="Search recipes..."
      bind:value={searchQuery}
      class="search-input"
    />
  </div>

  <ServerStatusBar {onStop} />

  <div class="recipe-list">
    {#each filtered as recipe (recipe.id)}
      <RecipeListItem
        {recipe}
        active={activeId === recipe.id}
        running={serverStore.isRunning(recipe.id)}
        {onSelect}
      />
    {/each}

    {#if filtered.length === 0}
      <div class="empty-state-small">
        {searchQuery ? 'No matching recipes' : 'No recipes yet'}
      </div>
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    width: 280px;
    min-width: 280px;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 16px 8px;
    -webkit-app-region: drag;
  }

  .sidebar-header .sidebar-actions {
    display: flex;
    gap: 2px;
    -webkit-app-region: no-drag;
  }

  .app-title {
    font-size: 18px;
    font-weight: 700;
    letter-spacing: -0.3px;
  }

  .search-box {
    position: relative;
    padding: 4px 16px 12px;
  }

  .search-icon {
    position: absolute;
    left: 26px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-tertiary);
    margin-top: -4px;
  }

  .search-input {
    width: 100%;
    padding: 6px 10px 6px 30px;
    background: var(--bg-tertiary);
    border: none;
    border-radius: var(--radius-sm);
    font-size: 13px;
  }

  .recipe-list {
    flex: 1;
    overflow-y: auto;
    padding: 0 8px 8px;
  }

  .empty-state-small {
    text-align: center;
    padding: 24px 16px;
    color: var(--text-tertiary);
    font-size: 13px;
  }

  .btn-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    transition: all 0.15s;
  }

  .btn-icon:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }
</style>
