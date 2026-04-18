<script lang="ts">
  import type { Backend, Capability, CommunityFilters } from '$lib/types';

  let { filters = $bindable() }: { filters: CommunityFilters } = $props();

  const backends: { id: Backend | 'any'; label: string }[] = [
    { id: 'any', label: 'Any' },
    { id: 'cuda', label: 'CUDA' },
    { id: 'vulkan', label: 'Vulkan' },
    { id: 'rocm', label: 'ROCm' },
    { id: 'metal', label: 'Metal' },
    { id: 'cpu', label: 'CPU' },
  ];

  const capabilities: { id: Capability | 'any'; label: string }[] = [
    { id: 'any', label: 'Any' },
    { id: 'chat', label: 'Chat' },
    { id: 'vision', label: 'Vision' },
    { id: 'embedding', label: 'Embed' },
  ];

  const sorts: { id: CommunityFilters['sort']; label: string }[] = [
    { id: 'recent', label: 'Recent' },
    { id: 'forks', label: 'Most forked' },
    { id: 'validated', label: 'Best validated' },
  ];

  const vramOptions: { id: number | null; label: string }[] = [
    { id: null, label: 'Any VRAM' },
    { id: 6, label: '6 GiB+' },
    { id: 8, label: '8 GiB+' },
    { id: 12, label: '12 GiB+' },
    { id: 16, label: '16 GiB+' },
    { id: 24, label: '24 GiB+' },
    { id: 48, label: '48 GiB+' },
  ];
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <h1 class="app-title">Community</h1>
  </div>

  <div class="search-box">
    <svg
      class="search-icon"
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      ><circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" /></svg
    >
    <input
      type="text"
      placeholder="Search by GPU or model..."
      bind:value={filters.search}
      class="search-input"
    />
  </div>

  <div class="filters">
    <section class="filter-group">
      <h2>Backend</h2>
      <div class="chips">
        {#each backends as opt (opt.id)}
          <button
            type="button"
            class="chip"
            class:active={filters.backend === opt.id}
            onclick={() => (filters.backend = opt.id)}
          >
            {opt.label}
          </button>
        {/each}
      </div>
    </section>

    <section class="filter-group">
      <h2>Capability</h2>
      <div class="chips">
        {#each capabilities as opt (opt.id)}
          <button
            type="button"
            class="chip"
            class:active={filters.capability === opt.id}
            onclick={() => (filters.capability = opt.id)}
          >
            {opt.label}
          </button>
        {/each}
      </div>
    </section>

    <section class="filter-group">
      <h2>VRAM</h2>
      <select bind:value={filters.vramMinGib} class="select">
        {#each vramOptions as opt (opt.id ?? 'any')}
          <option value={opt.id}>{opt.label}</option>
        {/each}
      </select>
    </section>

    <section class="filter-group">
      <h2>Sort</h2>
      <select bind:value={filters.sort} class="select">
        {#each sorts as opt (opt.id)}
          <option value={opt.id}>{opt.label}</option>
        {/each}
      </select>
    </section>
  </div>
</aside>

<style>
  .sidebar {
    width: 280px;
    min-width: 280px;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 16px 8px;
    -webkit-app-region: drag;
  }

  .app-title {
    font-size: 18px;
    font-weight: 700;
    letter-spacing: -0.3px;
  }

  .search-box {
    position: relative;
    padding: 4px 16px 12px;
  }

  .search-icon {
    position: absolute;
    left: 26px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-tertiary);
    margin-top: -4px;
  }

  .search-input {
    width: 100%;
    padding: 6px 10px 6px 30px;
    background: var(--bg-tertiary);
    border: none;
    border-radius: var(--radius-sm);
    font-size: 13px;
  }

  .filters {
    flex: 1;
    overflow-y: auto;
    padding: 4px 16px 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .filter-group h2 {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-tertiary);
    margin-bottom: 8px;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .chip {
    padding: 4px 10px;
    border-radius: 999px;
    font-size: 12px;
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    transition:
      background 0.15s,
      color 0.15s;
  }

  .chip:hover {
    color: var(--text-primary);
  }

  .chip.active {
    background: var(--accent);
    color: white;
  }

  .select {
    width: 100%;
    padding: 6px 8px;
    font-size: 13px;
  }
</style>
