import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { toast } from 'svelte-sonner';

import {
  cancelDownload,
  DOWNLOAD_CANCELLED,
  downloadHfModel,
  type DownloadProgress,
} from '$lib/api/models';
import { settingsStore } from '$lib/stores/settings.svelte';
import { errorMessage } from '$lib/utils/format';
import { fileName } from '$lib/utils/model';

export interface ActiveDownload {
  repoId: string;
  file: string;
  progress: DownloadProgress | null;
}

class DownloadsStore {
  active = $state<ActiveDownload | null>(null);
  /** Bumped after each successful download so model lists can rescan. */
  completed = $state(0);

  get percent(): number | null {
    const p = this.active?.progress;
    if (!p || !p.total_bytes) return null;
    return Math.min(100, (p.bytes_downloaded / p.total_bytes) * 100);
  }

  /** Resolves to the downloaded path, or null if cancelled or failed. */
  async start(repoId: string, file: string): Promise<string | null> {
    if (this.active) return null;
    const { hf_token, model_dir } = settingsStore.current;
    this.active = { repoId, file, progress: null };
    const base = fileName(file);
    let unlisten: UnlistenFn | null = null;
    try {
      unlisten = await listen<DownloadProgress>('download-progress', (e) => {
        if (this.active && e.payload.filename === base) this.active.progress = e.payload;
      });
      const path = await downloadHfModel(repoId, file, hf_token, model_dir);
      this.completed++;
      toast.success(`Downloaded ${base}`);
      return path;
    } catch (e) {
      if (errorMessage(e) !== DOWNLOAD_CANCELLED) toast.error(errorMessage(e));
      return null;
    } finally {
      unlisten?.();
      this.active = null;
    }
  }

  cancel(): void {
    if (this.active) void cancelDownload(this.active.file, settingsStore.current.model_dir);
  }
}

export const downloadsStore = new DownloadsStore();
