import { invoke } from '@tauri-apps/api/core';

export type GgufKind = 'Model' | 'Mmproj';

export interface ScannedModel {
  path: string;
  name: string;
  size_bytes: number;
  kind: GgufKind;
}

export function scanModels(directory: string, filter: 'model' | 'mmproj' | 'all' = 'model'): Promise<ScannedModel[]> {
  return invoke('scan_models', { directory, filter });
}

export function cleanupOrphanParts(directory: string): Promise<void> {
  return invoke('cleanup_orphan_parts', { directory });
}

export interface HfModelFile {
  name: string;
  size_bytes: number;
}

export function listHfModelFiles(
  repoId: string,
  hfToken: string,
  filter: 'model' | 'mmproj' | 'all' = 'model',
): Promise<HfModelFile[]> {
  return invoke('list_hf_model_files', { repoId, hfToken, filter });
}

export interface DownloadProgress {
  repo_id: string;
  filename: string;
  bytes_downloaded: number;
  total_bytes: number;
}

export function downloadHfModel(
  repoId: string,
  filename: string,
  hfToken: string,
  destDir: string,
): Promise<string> {
  return invoke('download_hf_model', { repoId, filename, hfToken, destDir });
}
