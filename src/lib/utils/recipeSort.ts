import type { Recipe } from '$lib/types';

export type RecipeSort = 'updated' | 'used' | 'name';

export const RECIPE_SORTS: { value: RecipeSort; label: string }[] = [
  { value: 'updated', label: 'Recently updated' },
  { value: 'used', label: 'Recently used' },
  { value: 'name', label: 'Name' },
];

export function isRecipeSort(value: unknown): value is RecipeSort {
  return RECIPE_SORTS.some((s) => s.value === value);
}

/** Pinned recipes always come first; `sort` orders within each group. */
export function sortRecipes(recipes: Recipe[], sort: RecipeSort): Recipe[] {
  const compare = (a: Recipe, b: Recipe): number => {
    if (sort === 'name') return a.name.localeCompare(b.name, undefined, { sensitivity: 'base' });
    const key = sort === 'used' ? 'last_used_at' : 'updated_at';
    return b[key].localeCompare(a[key]) || b.updated_at.localeCompare(a.updated_at);
  };
  return [...recipes].sort((a, b) => Number(b.pinned) - Number(a.pinned) || compare(a, b));
}
