import { describe, expect, it } from 'vitest';

import type { Recipe } from '$lib/types';
import { isRecipeSort, sortRecipes } from '$lib/utils/recipeSort';

function recipe(name: string, overrides: Partial<Recipe> = {}): Recipe {
  return {
    id: name,
    name,
    description: '',
    command: '',
    model_path: '',
    mmproj_path: '',
    gpu_info: '',
    tags: '',
    created_at: '',
    updated_at: '',
    pinned: false,
    last_used_at: '',
    ...overrides,
  };
}

const names = (list: Recipe[]) => list.map((r) => r.name);

describe('sortRecipes', () => {
  const a = recipe('alpha', { updated_at: '2026-01-01', last_used_at: '2026-03-01' });
  const b = recipe('Beta', { updated_at: '2026-02-01' });
  const c = recipe('charlie', { updated_at: '2026-01-15', last_used_at: '2026-02-01' });

  it('orders by most recent update', () => {
    expect(names(sortRecipes([a, b, c], 'updated'))).toEqual(['Beta', 'charlie', 'alpha']);
  });

  it('orders by most recent use, never-used last', () => {
    expect(names(sortRecipes([a, b, c], 'used'))).toEqual(['alpha', 'charlie', 'Beta']);
  });

  it('orders by name ignoring case', () => {
    expect(names(sortRecipes([c, b, a], 'name'))).toEqual(['alpha', 'Beta', 'charlie']);
  });

  it('keeps pinned recipes on top', () => {
    const pinned = { ...c, pinned: true };
    expect(names(sortRecipes([a, b, pinned], 'name'))).toEqual(['charlie', 'alpha', 'Beta']);
  });

  it('does not mutate the input', () => {
    const input = [a, b, c];
    sortRecipes(input, 'name');
    expect(names(input)).toEqual(['alpha', 'Beta', 'charlie']);
  });
});

describe('isRecipeSort', () => {
  it('accepts only known modes', () => {
    expect(isRecipeSort('used')).toBe(true);
    expect(isRecipeSort('size')).toBe(false);
    expect(isRecipeSort(null)).toBe(false);
  });
});
