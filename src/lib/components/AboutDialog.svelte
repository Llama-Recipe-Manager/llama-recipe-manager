<script lang="ts">
  import { getTauriVersion, getVersion } from '@tauri-apps/api/app';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { onMount } from 'svelte';

  import { updaterStore } from '$lib/stores/updater.svelte';
  import { errorMessage } from '$lib/utils/format';

  let {
    open,
    onClose,
    onError,
  }: {
    open: boolean;
    onClose: () => void;
    onError: (msg: string) => void;
  } = $props();

  const REPO_URL = 'https://github.com/Llama-Recipe-Manager/llama-recipe-manager';

  let appVersion = $state<string>('');
  let tauriVersion = $state<string>('');
  let lastCheckedAt = $state<number | null>(null);
  let copied = $state(false);

  onMount(async () => {
    try {
      [appVersion, tauriVersion] = await Promise.all([getVersion(), getTauriVersion()]);
    } catch {
      // getVersion / getTauriVersion only work inside the Tauri runtime;
      // fine to ignore in unit tests.
    }
  });

  // Close on Escape while open. Bound at window-level so the dialog
  // doesn't need explicit focus to react.
  $effect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });

  // Refresh the "last checked" timestamp whenever an update check finishes
  // — including the initial auto-check fired at app startup.
  $effect(() => {
    if (updaterStore.checkedOnce) lastCheckedAt = Date.now();
  });

  async function handleCheck() {
    await updaterStore.check();
  }

  async function copyVersion() {
    const text = `Llama Recipe Manager v${appVersion} (Tauri v${tauriVersion}, ${navigator.platform})`;
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch (e) {
      onError(errorMessage(e));
    }
  }

  async function openLink(url: string) {
    try {
      await openUrl(url);
    } catch (e) {
      onError(errorMessage(e));
    }
  }

  function fmtAgo(ms: number | null): string {
    if (ms === null) return 'never';
    const diff = Math.max(0, Date.now() - ms);
    const sec = Math.round(diff / 1000);
    if (sec < 60) return `${sec}s ago`;
    const min = Math.round(sec / 60);
    if (min < 60) return `${min}m ago`;
    const hr = Math.round(min / 60);
    if (hr < 24) return `${hr}h ago`;
    return `${Math.round(hr / 24)}d ago`;
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="backdrop" onclick={onClose}></div>
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="about-title" tabindex="-1">
    <header class="head">
      <div class="brand">
        <img src="/icon.png" alt="" class="logo" aria-hidden="true" />
        <div class="brand-text">
          <h2 id="about-title" class="title">Llama Recipe Manager</h2>
          <div class="version-line">
            <span class="ver">v{appVersion || '—'}</span>
            <button
              class="copy-btn"
              onclick={copyVersion}
              title="Copy version (handy for bug reports)"
              aria-label="Copy version"
            >
              {copied ? 'Copied' : 'Copy'}
            </button>
          </div>
        </div>
      </div>
      <button class="close-btn" onclick={onClose} title="Close" aria-label="Close About dialog">
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M18 6L6 18M6 6l12 12" />
        </svg>
      </button>
    </header>

    <div class="status-row">
      {#if updaterStore.status === 'available' && updaterStore.info}
        <span class="badge update-available">
          Update available — v{updaterStore.info.version}
        </span>
      {:else if updaterStore.status === 'downloading'}
        <span class="badge update-progress">Installing update…</span>
      {:else if updaterStore.status === 'error'}
        <span class="badge update-error">Update check failed</span>
      {:else if updaterStore.checkedOnce}
        <span class="badge update-ok">Up to date</span>
      {/if}
    </div>

    <div class="grid">
      <div class="cell">
        <div class="cell-label">Build</div>
        <div class="cell-value">{tauriVersion ? `Tauri ${tauriVersion}` : '—'}</div>
      </div>
      <div class="cell">
        <div class="cell-label">Update channel</div>
        <div class="cell-value">GitHub Releases (signed)</div>
      </div>
      <div class="cell">
        <div class="cell-label">Last checked</div>
        <div class="cell-value">{fmtAgo(lastCheckedAt)}</div>
      </div>
      <div class="cell">
        <div class="cell-label">License</div>
        <div class="cell-value">MIT</div>
      </div>
    </div>

    {#if updaterStore.error}
      <div class="error">{updaterStore.error}</div>
    {/if}

    <div class="actions">
      <button
        class="btn secondary"
        onclick={handleCheck}
        disabled={updaterStore.status === 'checking' || updaterStore.status === 'downloading'}
      >
        {#if updaterStore.status === 'checking'}
          Checking…
        {:else if updaterStore.status === 'downloading'}
          Installing…
        {:else}
          Check for updates
        {/if}
      </button>
      {#if updaterStore.status === 'available'}
        <button class="btn primary" onclick={() => updaterStore.install()}>
          Install v{updaterStore.info?.version} &amp; restart
        </button>
      {/if}
      <span class="link-spacer"></span>
      <button class="link" onclick={() => openLink(REPO_URL)}>GitHub</button>
      <button class="link" onclick={() => openLink(`${REPO_URL}/issues/new/choose`)}>
        Report an issue
      </button>
      <button class="link" onclick={() => openLink(`${REPO_URL}/blob/main/CHANGELOG.md`)}>
        Changelog
      </button>
    </div>

    <footer class="foot">
      <span>© 2026 Mohammad Ashar Khan</span>
      <span class="dot">·</span>
      <span>Built on llama.cpp, Tauri & SvelteKit</span>
    </footer>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(2px);
    z-index: 50;
    animation: fade-in 0.12s ease-out;
  }

  .dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: min(520px, calc(100vw - 32px));
    max-height: calc(100vh - 64px);
    overflow: auto;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg, 12px);
    padding: 20px 22px 16px;
    z-index: 51;
    box-shadow:
      0 18px 48px rgba(0, 0, 0, 0.32),
      0 4px 12px rgba(0, 0, 0, 0.18);
    animation: pop-in 0.14s ease-out;
  }

  @keyframes fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  @keyframes pop-in {
    from {
      opacity: 0;
      transform: translate(-50%, -48%) scale(0.97);
    }
    to {
      opacity: 1;
      transform: translate(-50%, -50%) scale(1);
    }
  }

  .head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .logo {
    width: 44px;
    height: 44px;
    border-radius: 10px;
    flex-shrink: 0;
  }

  .brand-text {
    min-width: 0;
  }

  .title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .version-line {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 2px;
  }

  .version-line .ver {
    font-size: 13px;
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
    font-family: var(--font-mono);
  }

  .copy-btn {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    background: var(--bg-tertiary);
  }

  .copy-btn:hover {
    color: var(--text-primary);
  }

  .close-btn {
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .close-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .status-row {
    margin-top: 14px;
    min-height: 22px;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    font-size: 11px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 100px;
  }

  .badge.update-available {
    background: rgba(10, 132, 255, 0.13);
    color: var(--accent);
  }
  .badge.update-progress {
    background: rgba(255, 159, 10, 0.13);
    color: var(--warning, #ff9f0a);
  }
  .badge.update-error {
    background: rgba(255, 59, 48, 0.13);
    color: var(--danger);
  }
  .badge.update-ok {
    background: rgba(52, 199, 89, 0.13);
    color: var(--success);
  }

  .grid {
    margin-top: 12px;
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 10px 18px;
    padding: 12px 14px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
  }

  @media (max-width: 460px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }

  .cell-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-tertiary);
    font-weight: 600;
    margin-bottom: 2px;
  }

  .cell-value {
    font-size: 13px;
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .error {
    margin-top: 10px;
    font-size: 12px;
    color: var(--danger);
    background: rgba(255, 59, 48, 0.08);
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    word-break: break-word;
  }

  .actions {
    margin-top: 14px;
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .link-spacer {
    flex: 1;
  }

  .link {
    font-size: 12px;
    color: var(--text-secondary);
    text-decoration: none;
    padding: 4px 6px;
    border-radius: var(--radius-sm);
  }

  .link:hover {
    color: var(--accent);
    background: var(--bg-tertiary);
  }

  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 6px 14px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 500;
    transition: all 0.15s;
    white-space: nowrap;
  }

  .btn.primary {
    background: var(--accent);
    color: white;
  }
  .btn.primary:hover {
    filter: brightness(1.08);
  }

  .btn.secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }
  .btn.secondary:hover:not(:disabled) {
    background: var(--bg-quaternary, var(--bg-tertiary));
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .foot {
    margin-top: 16px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .foot .dot {
    opacity: 0.6;
  }
</style>
