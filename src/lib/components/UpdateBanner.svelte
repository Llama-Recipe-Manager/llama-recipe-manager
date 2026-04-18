<script lang="ts">
  import { updaterStore } from '$lib/stores/updater.svelte';

  function fmtBytes(n: number | null): string {
    if (n === null) return '?';
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
    return `${(n / 1024 / 1024).toFixed(1)} MB`;
  }

  const pct = $derived.by(() => {
    const p = updaterStore.progress;
    if (!p || !p.total) return null;
    return Math.min(100, Math.round((p.downloaded / p.total) * 100));
  });
</script>

{#if updaterStore.status === 'available' && updaterStore.info}
  <div class="banner" role="status">
    <div class="banner-text">
      <strong>Update available</strong>
      <span>
        v{updaterStore.info.version} is out (you're on v{updaterStore.info.currentVersion}).
      </span>
    </div>
    <div class="banner-actions">
      <button class="btn ghost" onclick={() => updaterStore.dismiss()}>Later</button>
      <button class="btn primary" onclick={() => updaterStore.install()}
        >Install &amp; Restart</button
      >
    </div>
  </div>
{:else if updaterStore.status === 'downloading'}
  <div class="banner" role="status">
    <div class="banner-text">
      <strong>Downloading update…</strong>
      <span>
        {fmtBytes(updaterStore.progress?.downloaded ?? 0)}
        {#if updaterStore.progress?.total}/ {fmtBytes(updaterStore.progress.total)}{/if}
        {#if pct !== null}— {pct}%{/if}
      </span>
    </div>
    {#if pct !== null}
      <div class="bar"><div class="bar-fill" style="width: {pct}%"></div></div>
    {:else}
      <div class="bar"><div class="bar-fill indeterminate"></div></div>
    {/if}
  </div>
{/if}

<style>
  .banner {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 16px;
    background: rgba(10, 132, 255, 0.1);
    border-bottom: 1px solid rgba(10, 132, 255, 0.25);
    font-size: 12px;
    flex-wrap: wrap;
  }

  .banner-text {
    display: flex;
    align-items: baseline;
    gap: 8px;
    flex: 1;
    min-width: 0;
  }

  .banner-text strong {
    color: var(--accent);
    font-weight: 600;
  }

  .banner-text span {
    color: var(--text-secondary);
  }

  .banner-actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }

  .btn {
    padding: 4px 12px;
    font-size: 12px;
    font-weight: 500;
    border-radius: var(--radius-sm);
  }

  .btn.primary {
    background: var(--accent);
    color: white;
  }

  .btn.primary:hover {
    background: var(--accent-hover);
  }

  .btn.ghost {
    color: var(--text-secondary);
  }

  .btn.ghost:hover {
    color: var(--text-primary);
    background: var(--bg-tertiary);
  }

  .bar {
    width: 100%;
    height: 4px;
    background: rgba(10, 132, 255, 0.15);
    border-radius: 2px;
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    background: var(--accent);
    transition: width 0.2s ease;
  }

  .bar-fill.indeterminate {
    width: 30%;
    animation: slide 1.4s infinite ease-in-out;
  }

  @keyframes slide {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(400%);
    }
  }
</style>
