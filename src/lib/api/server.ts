import { invoke } from '@tauri-apps/api/core';
import type { LogLine, ServerStatus } from '$lib/types';

export function startServer(
  recipeId: string,
  command: string,
  modelPath: string,
  mmprojPath: string,
): Promise<void> {
  return invoke('start_server', { recipeId, command, modelPath, mmprojPath });
}

export function stopServer(): Promise<void> {
  return invoke('stop_server');
}

export function getServerStatus(): Promise<ServerStatus | null> {
  return invoke('get_server_status');
}

export function getServerLogs(): Promise<LogLine[]> {
  return invoke('get_server_logs');
}

export function clearServerLogs(): Promise<void> {
  return invoke('clear_server_logs');
}
