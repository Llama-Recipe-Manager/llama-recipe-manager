import type { UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { settingsStore } from './settings.svelte';
import { applyTheme } from '$lib/utils/theme';
import type { Theme } from '$lib/utils/theme';

class ThemeStore {
  current = $state<Theme>('light');
  private unlisten: UnlistenFn | null = null;

  /** Subscribe (or re-subscribe) to theme changes, respecting the persisted setting. */
  async subscribe(): Promise<void> {
    // Tear down any previous subscription.
    this.unlisten?.();
    this.unlisten = null;

    const pref = settingsStore.current.theme ?? 'system';

    if (pref === 'light') {
      this.current = 'light';
      applyTheme('light');
      return;
    }

    if (pref === 'dark') {
      this.current = 'dark';
      applyTheme('dark');
      return;
    }

    // 'system' — follow OS preference.
    try {
      const win = getCurrentWindow();
      const initial = ((await win.theme()) ?? 'light') as Theme;
      this.current = initial;
      applyTheme(initial);
      this.unlisten = await win.onThemeChanged(({ payload }) => {
        const t = payload as Theme;
        this.current = t;
        applyTheme(t);
      });
    } catch {
      this.current = 'light';
      applyTheme('light');
    }
  }

  unsubscribe(): void {
    this.unlisten?.();
    this.unlisten = null;
  }
}

export const themeStore = new ThemeStore();
