import { listRecipes } from '$lib/api';
import type { Recipe } from '$lib/types';

class RecipesStore {
  items = $state<Recipe[]>([]);
  selectedId = $state<string | null>(null);

  selected = $derived(this.items.find((r) => r.id === this.selectedId) ?? null);

  async refresh(): Promise<void> {
    this.items = await listRecipes();
  }

  select(id: string | null): void {
    this.selectedId = id;
  }

  filter(query: string): Recipe[] {
    if (!query) return this.items;
    const q = query.toLowerCase();
    return this.items.filter(
      (r) =>
        r.name.toLowerCase().includes(q) ||
        r.tags.toLowerCase().includes(q) ||
        r.gpu_info.toLowerCase().includes(q),
    );
  }
}

export const recipesStore = new RecipesStore();
