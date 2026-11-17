import { listRecipes, setRecipePinned } from '$lib/api';
import type { Recipe } from '$lib/types';
import { isRecipeSort, sortRecipes, type RecipeSort } from '$lib/utils/recipeSort';

const SORT_KEY = 'recipe-sort';

function loadSort(): RecipeSort {
  const saved = globalThis.localStorage?.getItem(SORT_KEY);
  return isRecipeSort(saved) ? saved : 'updated';
}

class RecipesStore {
  items = $state<Recipe[]>([]);
  selectedId = $state<string | null>(null);
  sort = $state<RecipeSort>(loadSort());

  selected = $derived(this.items.find((r) => r.id === this.selectedId) ?? null);

  async refresh(): Promise<void> {
    this.items = await listRecipes();
  }

  select(id: string | null): void {
    this.selectedId = id;
  }

  setSort(sort: RecipeSort): void {
    this.sort = sort;
    globalThis.localStorage?.setItem(SORT_KEY, sort);
  }

  async setPinned(id: string, pinned: boolean): Promise<void> {
    const updated = await setRecipePinned(id, pinned);
    this.items = this.items.map((r) => (r.id === id ? updated : r));
  }

  filter(query: string): Recipe[] {
    const q = query.toLowerCase();
    const matches = q
      ? this.items.filter(
          (r) =>
            r.name.toLowerCase().includes(q) ||
            r.tags.toLowerCase().includes(q) ||
            r.gpu_info.toLowerCase().includes(q),
        )
      : this.items;
    return sortRecipes(matches, this.sort);
  }
}

export const recipesStore = new RecipesStore();
