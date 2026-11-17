import { open, save } from '@tauri-apps/plugin-dialog';

import { exportRecipe, importRecipe } from '$lib/api/recipes';
import type { Recipe } from '$lib/types';

const FILTERS = [{ name: 'Llama Recipe', extensions: ['ripe'] }];

export function ripeFileName(name: string): string {
  const slug = name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
  return `${slug || 'recipe'}.ripe`;
}

/** Resolves `false` if the user dismissed the save dialog. */
export async function exportRecipeToFile(recipe: Recipe): Promise<boolean> {
  const path = await save({ defaultPath: ripeFileName(recipe.name), filters: FILTERS });
  if (!path) return false;
  await exportRecipe(recipe.id, path);
  return true;
}

/** Resolves `null` if the user dismissed the open dialog. */
export async function importRecipeFromFile(): Promise<Recipe | null> {
  const path = await open({ multiple: false, directory: false, filters: FILTERS });
  if (typeof path !== 'string') return null;
  return importRecipe(path);
}
