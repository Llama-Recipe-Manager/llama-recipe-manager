import { save } from '@tauri-apps/plugin-dialog';

import { exportAllRecipesIni, exportRecipeIni } from '$lib/api/recipes';
import type { Recipe } from '$lib/types';

const FILTERS = [{ name: 'Llama Server Preset', extensions: ['ini'] }];

export function iniFileName(name: string): string {
  const slug = name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
  return `${slug || 'recipe'}.ini`;
}

/** Resolves `false` if the user dismissed the save dialog. */
export async function exportRecipeToIni(recipe: Recipe): Promise<boolean> {
  const path = await save({ defaultPath: iniFileName(recipe.name), filters: FILTERS });
  if (!path) return false;
  await exportRecipeIni(recipe.id, path);
  return true;
}

/** Resolves `false` if the user dismissed the save dialog. */
export async function exportAllRecipesToIni(): Promise<boolean> {
  const path = await save({ defaultPath: 'llama-models.ini', filters: FILTERS });
  if (!path) return false;
  await exportAllRecipesIni(path);
  return true;
}
