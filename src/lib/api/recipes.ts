import { invoke } from '@tauri-apps/api/core';
import type { CreateRecipe, Recipe, UpdateRecipe } from '$lib/types';

export function listRecipes(): Promise<Recipe[]> {
  return invoke('list_recipes');
}

export function getRecipe(id: string): Promise<Recipe> {
  return invoke('get_recipe', { id });
}

export function createRecipe(input: CreateRecipe): Promise<Recipe> {
  return invoke('create_recipe', { input });
}

export function updateRecipe(input: UpdateRecipe): Promise<Recipe> {
  return invoke('update_recipe', { input });
}

export function deleteRecipe(id: string): Promise<void> {
  return invoke('delete_recipe', { id });
}

export function duplicateRecipe(id: string): Promise<Recipe> {
  return invoke('duplicate_recipe', { id });
}

export function setRecipePinned(id: string, pinned: boolean): Promise<Recipe> {
  return invoke('set_recipe_pinned', { id, pinned });
}

export function exportRecipe(id: string, path: string): Promise<void> {
  return invoke('export_recipe', { id, path });
}

export function exportRecipeIni(id: string, path: string): Promise<void> {
  return invoke('export_recipe_ini', { id, path });
}

export function exportAllRecipesIni(path: string): Promise<void> {
  return invoke('export_all_recipes_ini', { path });
}

export function importRecipe(path: string): Promise<Recipe> {
  return invoke('import_recipe', { path });
}
