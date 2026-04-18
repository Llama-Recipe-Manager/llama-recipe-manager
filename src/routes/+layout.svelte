<script lang="ts">
  import '../app.css';
  import { onMount, onDestroy, type Snippet } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  let { children }: { children: Snippet } = $props();

  let unlistenTheme: UnlistenFn | undefined;

  function applyTheme(theme: 'light' | 'dark' | null) {
    const root = document.documentElement;
    if (theme === 'dark' || theme === 'light') {
      root.setAttribute('data-theme', theme);
    } else {
      root.removeAttribute('data-theme');
    }
  }

  onMount(async () => {
    try {
      const win = getCurrentWindow();
      // Initial theme from the OS (via Tauri)
      const current = await win.theme();
      applyTheme(current ?? null);
      // Listen for subsequent OS theme changes
      unlistenTheme = await win.onThemeChanged(({ payload }) => {
        applyTheme(payload ?? null);
      });
    } catch {
      // Non-Tauri / fallback: rely on CSS prefers-color-scheme
    }
  });

  onDestroy(() => {
    unlistenTheme?.();
  });
</script>

{@render children()}
