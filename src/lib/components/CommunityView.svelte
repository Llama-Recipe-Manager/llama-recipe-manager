<script lang="ts">
  import type { CommunityFilters } from '$lib/types';

  let { filters }: { filters: CommunityFilters } = $props();

  const activeFilterCount = $derived(
    (filters.search ? 1 : 0) +
      (filters.backend !== 'any' ? 1 : 0) +
      (filters.capability !== 'any' ? 1 : 0) +
      (filters.vramMinGib !== null ? 1 : 0),
  );
</script>

<div class="community">
  <header class="header">
    <div>
      <h1>Community recipes</h1>
      <p class="subtitle">
        Browse, fork, and validate recipes shared by the community.
        {#if activeFilterCount > 0}
          <span class="filter-meta"
            >{activeFilterCount} filter{activeFilterCount === 1 ? '' : 's'} active</span
          >
        {/if}
      </p>
    </div>
  </header>

  <div class="placeholder">
    <div class="placeholder-card">
      <div class="badge-row">
        <span class="badge soon">Coming soon</span>
      </div>
      <h2>Community recipes are on the way</h2>
      <p>
        Sign in to publish your own recipes, fork what others share, and validate their performance
        claims on your hardware. The browse experience and filters above are wired up; data lands in
        a follow-up release.
      </p>
      <div class="actions">
        <button type="button" class="btn primary" disabled>Sign in (coming soon)</button>
        <a
          class="btn ghost"
          href="https://github.com/llama-recipe-manager"
          target="_blank"
          rel="noreferrer"
        >
          Read the design doc
        </a>
      </div>
    </div>

    <div class="sample-grid" aria-hidden="true">
      <article class="sample-card">
        <div class="sample-card-head">
          <span class="sample-name">Llama-3.1-8B Q4_K_M</span>
          <span class="badge backend">CUDA</span>
        </div>
        <p class="sample-author">@example · RTX 4090 · 24 GiB</p>
        <div class="sample-stats">
          <div><strong>132</strong><span>tg t/s</span></div>
          <div><strong>3 240</strong><span>pp t/s</span></div>
          <div><strong>87</strong><span>forks</span></div>
        </div>
      </article>
      <article class="sample-card">
        <div class="sample-card-head">
          <span class="sample-name">Qwen2-VL-7B vision</span>
          <span class="badge backend">Vulkan</span>
        </div>
        <p class="sample-author">@example · RX 7900 XTX · 24 GiB</p>
        <div class="sample-stats">
          <div><strong>71</strong><span>tg t/s</span></div>
          <div><strong>1 980</strong><span>pp t/s</span></div>
          <div><strong>42</strong><span>forks</span></div>
        </div>
      </article>
      <article class="sample-card">
        <div class="sample-card-head">
          <span class="sample-name">Mistral-Small-24B Q5</span>
          <span class="badge backend">Metal</span>
        </div>
        <p class="sample-author">@example · M3 Max · 64 GiB</p>
        <div class="sample-stats">
          <div><strong>38</strong><span>tg t/s</span></div>
          <div><strong>910</strong><span>pp t/s</span></div>
          <div><strong>118</strong><span>forks</span></div>
        </div>
      </article>
    </div>
  </div>
</div>

<style>
  .community {
    flex: 1;
    overflow-y: auto;
    padding: 32px 40px 48px;
  }

  .header h1 {
    font-size: 24px;
    font-weight: 700;
    letter-spacing: -0.4px;
  }

  .subtitle {
    font-size: 13px;
    color: var(--text-secondary);
    margin-top: 4px;
  }

  .filter-meta {
    margin-left: 8px;
    color: var(--accent);
  }

  .placeholder {
    margin-top: 28px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .placeholder-card {
    background: var(--bg-secondary);
    border: 1px dashed var(--border);
    border-radius: var(--radius-lg);
    padding: 28px;
    max-width: 720px;
  }

  .badge-row {
    margin-bottom: 12px;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.3px;
    text-transform: uppercase;
  }

  .badge.soon {
    background: var(--warning);
    color: #1d1d1f;
  }

  .badge.backend {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    text-transform: none;
    letter-spacing: 0;
    font-size: 11px;
  }

  .placeholder-card h2 {
    font-size: 18px;
    font-weight: 600;
    margin-bottom: 8px;
  }

  .placeholder-card p {
    font-size: 14px;
    color: var(--text-secondary);
    line-height: 1.6;
    max-width: 56ch;
  }

  .actions {
    display: flex;
    gap: 8px;
    margin-top: 16px;
  }

  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 7px 16px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 500;
    text-decoration: none;
  }

  .btn.primary {
    background: var(--accent);
    color: white;
  }

  .btn.primary:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .btn.ghost {
    background: transparent;
    color: var(--text-primary);
    border: 1px solid var(--border);
  }

  .btn.ghost:hover {
    background: var(--bg-tertiary);
  }

  .sample-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 12px;
    opacity: 0.55;
    pointer-events: none;
  }

  .sample-card {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .sample-card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .sample-name {
    font-weight: 600;
    font-size: 13px;
  }

  .sample-author {
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .sample-stats {
    display: flex;
    gap: 12px;
    margin-top: 4px;
  }

  .sample-stats > div {
    display: flex;
    flex-direction: column;
    line-height: 1.1;
  }

  .sample-stats strong {
    font-size: 14px;
    font-weight: 600;
  }

  .sample-stats span {
    font-size: 10px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }
</style>
