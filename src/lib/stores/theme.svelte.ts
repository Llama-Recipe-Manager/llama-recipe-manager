import type { UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

type Theme = 'light' | 'dark';

function applyTheme(theme: Theme): void {
  if (typeof document === 'undefined') return;
  document.documentElement.setAttribute('data-theme', theme);
}

/**
 * Subscribe to OS-level theme changes via Tauri and apply them to the
 * `<html data-theme="...">` attribute. Returns an unsubscribe function.
 */
export async function subscribeTheme(): Promise<() => void> {
  let unlisten: UnlistenFn | null = null;
  try {
    const win = getCurrentWindow();
    const initial = (await win.theme()) ?? 'light';
    applyTheme(initial as Theme);
    unlisten = await win.onThemeChanged(({ payload }) => {
      applyTheme(payload as Theme);
    });
  } catch {
    // Not running inside Tauri (e.g. Vite SSR/dev preview) — ignore.
  }
  return () => {
    unlisten?.();
  };
}
