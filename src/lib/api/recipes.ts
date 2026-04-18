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
