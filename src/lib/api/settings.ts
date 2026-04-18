import { invoke } from '@tauri-apps/api/core';
import type { LlamaServerInfo, Settings } from '$lib/types';

export function getSettings(): Promise<Settings> {
  return invoke('get_settings');
}

export function updateSettings(settings: Settings): Promise<Settings> {
  return invoke('update_settings', { settings });
}

export function getLlamaServerInfo(): Promise<LlamaServerInfo> {
  return invoke('get_llama_server_info');
}
