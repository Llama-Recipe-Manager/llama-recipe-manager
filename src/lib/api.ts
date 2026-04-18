import { invoke } from '@tauri-apps/api/core';

export interface Recipe {
  id: string;
  name: string;
  description: string;
  command: string;
  model_path: string;
  mmproj_path: string;
  gpu_info: string;
  tags: string;
  created_at: string;
  updated_at: string;
}

export interface CreateRecipe {
  name: string;
  description: string;
  command: string;
  model_path: string;
  mmproj_path: string;
  gpu_info: string;
  tags: string;
}

export interface UpdateRecipe {
  id: string;
  name: string;
  description: string;
  command: string;
  model_path: string;
  mmproj_path: string;
  gpu_info: string;
  tags: string;
}

export interface Settings {
  host: string;
  port: number;
  model_dir: string;
  llama_server_path: string;
}

export interface ServerStatus {
  recipe_id: string;
  running: boolean;
  pid: number | null;
}

export interface LogLine {
  recipe_id: string;
  line: string;
  is_stderr: boolean;
}

export interface GpuDevice {
  name: string;
  vram_mib: number;
  compute_capability: string;
}

export interface LlamaServerInfo {
  version: string;
  compiler: string;
  gpu_devices: GpuDevice[];
  raw_output: string;
}

// Settings
export async function getSettings(): Promise<Settings> {
  return invoke('get_settings');
}

export async function updateSettings(settings: Settings): Promise<Settings> {
  return invoke('update_settings', { settings });
}

// Server info
export async function getLlamaServerInfo(): Promise<LlamaServerInfo> {
  return invoke('get_llama_server_info');
}

// Recipes
export async function listRecipes(): Promise<Recipe[]> {
  return invoke('list_recipes');
}

export async function getRecipe(id: string): Promise<Recipe> {
  return invoke('get_recipe', { id });
}

export async function createRecipe(input: CreateRecipe): Promise<Recipe> {
  return invoke('create_recipe', { input });
}

export async function updateRecipe(input: UpdateRecipe): Promise<Recipe> {
  return invoke('update_recipe', { input });
}

export async function deleteRecipe(id: string): Promise<void> {
  return invoke('delete_recipe', { id });
}

export async function duplicateRecipe(id: string): Promise<Recipe> {
  return invoke('duplicate_recipe', { id });
}

// Server
export async function startServer(recipeId: string, command: string, modelPath: string, mmprojPath: string): Promise<void> {
  return invoke('start_server', { recipeId, command, modelPath, mmprojPath });
}

export async function stopServer(): Promise<void> {
  return invoke('stop_server');
}

export async function getServerStatus(): Promise<ServerStatus | null> {
  return invoke('get_server_status');
}

export async function getServerLogs(): Promise<LogLine[]> {
  return invoke('get_server_logs');
}
