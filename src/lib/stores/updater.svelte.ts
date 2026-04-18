/**
 * Reactive store wrapping the updater plugin. We keep the entire update
 * flow (check → download → install → relaunch) here so multiple UI surfaces
 * can subscribe to the same status without each running its own check.
 *
 * The check runs once on app startup and can be re-triggered manually from
 * the Settings page. Failures are non-fatal — an unconfigured updater (e.g.
 * a dev build with no signing key) silently produces "no update".
 */

import {
  checkForUpdate,
  installPendingUpdate,
  type UpdateInfo,
  type UpdateProgress,
} from '$lib/api/updater';

export type UpdaterStatus = 'idle' | 'checking' | 'available' | 'downloading' | 'ready' | 'error';

class UpdaterStore {
  status = $state<UpdaterStatus>('idle');
  info = $state<UpdateInfo | null>(null);
  progress = $state<UpdateProgress | null>(null);
  error = $state<string | null>(null);
  /** True after the first auto-check completes (success or failure). */
  checkedOnce = $state(false);

  async check(): Promise<void> {
    if (this.status === 'checking' || this.status === 'downloading') return;
    this.status = 'checking';
    this.error = null;
    try {
      const info = await checkForUpdate();
      if (info) {
        this.info = info;
        this.status = 'available';
      } else {
        this.info = null;
        this.status = 'idle';
      }
    } catch (e) {
      this.error = (e as Error).message ?? String(e);
      this.status = 'error';
    } finally {
      this.checkedOnce = true;
    }
  }

  async install(): Promise<void> {
    if (this.status !== 'available') return;
    this.status = 'downloading';
    this.error = null;
    this.progress = { downloaded: 0, total: null };
    try {
      await installPendingUpdate((p) => (this.progress = p));
      // We won't actually reach here — the app relaunches inside install().
      this.status = 'ready';
    } catch (e) {
      this.error = (e as Error).message ?? String(e);
      this.status = 'error';
    }
  }

  dismiss(): void {
    if (this.status === 'available') {
      this.status = 'idle';
    }
  }
}

export const updaterStore = new UpdaterStore();
