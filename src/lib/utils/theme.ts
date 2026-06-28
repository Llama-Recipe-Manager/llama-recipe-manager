export type Theme = 'light' | 'dark';

export function applyTheme(theme: Theme): void {
  if (typeof document === 'undefined') return;
  document.documentElement.setAttribute('data-theme', theme);
  document.documentElement.style.setProperty(
    '--select-color-scheme',
    theme === 'light' ? 'light' : theme === 'dark' ? 'dark' : 'light dark',
  );
}
