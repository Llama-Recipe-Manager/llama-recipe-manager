<script lang="ts">
  import type { Recipe } from '$lib/types';

  let {
    recipe,
    active,
    running,
    onSelect,
  }: {
    recipe: Recipe;
    active: boolean;
    running: boolean;
    onSelect: (id: string) => void;
  } = $props();
</script>

<button class="recipe-item" class:active onclick={() => onSelect(recipe.id)}>
  <div class="recipe-item-header">
    <span class="recipe-item-name">{recipe.name}</span>
    {#if running}
      <span class="status-dot running" title="Running"></span>
    {/if}
  </div>
  {#if recipe.gpu_info}
    <span class="recipe-item-meta">{recipe.gpu_info}</span>
  {:else if recipe.tags}
    <span class="recipe-item-meta">{recipe.tags}</span>
  {/if}
</button>

<style>
  .recipe-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    margin-bottom: 2px;
    transition: background 0.1s;
  }

  .recipe-item:hover {
    background: var(--bg-tertiary);
  }

  .recipe-item.active {
    background: var(--accent);
    color: white;
  }

  .recipe-item.active .recipe-item-meta {
    color: rgba(255, 255, 255, 0.7);
  }

  .recipe-item-header {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .recipe-item-name {
    font-weight: 500;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .recipe-item-meta {
    font-size: 11px;
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .status-dot.running {
    background: var(--success);
    box-shadow: 0 0 4px var(--success);
  }
</style>
