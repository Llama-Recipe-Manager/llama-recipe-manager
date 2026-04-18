<script lang="ts">
  import { FEATURES } from '$lib/featureFlags';
  import type { Section } from '$lib/types';

  let {
    section,
    serverRunning,
    onSelect,
    onJumpToRunning,
    onAbout,
  }: {
    section: Section;
    serverRunning: boolean;
    onSelect: (s: Section) => void;
    onJumpToRunning: () => void;
    onAbout: () => void;
  } = $props();

  const items = $derived(
    (
      [
        { id: 'recipes', label: 'My Recipes' },
        { id: 'community', label: 'Community' },
        { id: 'settings', label: 'Settings' },
      ] satisfies { id: Section; label: string }[]
    ).filter((item) => item.id !== 'community' || FEATURES.community),
  );
</script>

<nav class="rail" aria-label="Primary">
  <div class="rail-top">
    {#each items as item (item.id)}
      <button
        type="button"
        class="rail-btn"
        class:active={section === item.id}
        title={item.label}
        aria-label={item.label}
        aria-current={section === item.id ? 'page' : undefined}
        onclick={() => onSelect(item.id)}
      >
        {#if item.id === 'recipes'}
          <svg
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M3 11l9-8 9 8" />
            <path d="M5 10v10a1 1 0 001 1h4v-6h4v6h4a1 1 0 001-1V10" />
          </svg>
        {:else if item.id === 'community'}
          <svg
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="12" cy="12" r="9" />
            <path d="M3 12h18M12 3a14 14 0 010 18M12 3a14 14 0 000 18" />
          </svg>
        {:else}
          <svg
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="12" cy="12" r="3" />
            <path
              d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 01-2.83 2.83l-.06-.06A1.65 1.65 0 0015 19.4a1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.6 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.6a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"
            />
          </svg>
        {/if}
      </button>
    {/each}
  </div>

  <div class="rail-bottom">
    {#if serverRunning}
      <button
        type="button"
        class="rail-btn running"
        title="Server is running — jump to recipe"
        aria-label="Server is running"
        onclick={onJumpToRunning}
      >
        <span class="pulse-dot"></span>
      </button>
    {/if}
    <button
      type="button"
      class="rail-btn"
      title="About Llama Recipe Manager"
      aria-label="About"
      onclick={onAbout}
    >
      <svg
        width="20"
        height="20"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <circle cx="12" cy="12" r="9" />
        <path d="M12 8h.01M11 12h1v5h1" />
      </svg>
    </button>
    {#if FEATURES.account}
      <button
        type="button"
        class="rail-btn"
        title="Sign in (coming soon)"
        aria-label="Sign in"
        disabled
      >
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2" />
          <circle cx="12" cy="7" r="4" />
        </svg>
      </button>
    {/if}
  </div>
</nav>

<style>
  .rail {
    width: 56px;
    min-width: 56px;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: space-between;
    padding: 12px 0;
    -webkit-app-region: drag;
  }

  .rail-top,
  .rail-bottom {
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: center;
    -webkit-app-region: no-drag;
  }

  .rail-top {
    margin-top: 28px;
  }

  .rail-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    transition:
      background 0.15s,
      color 0.15s;
  }

  .rail-btn:hover:not(:disabled) {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .rail-btn.active {
    background: var(--accent);
    color: white;
  }

  .rail-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .rail-btn.running {
    background: transparent;
  }

  .pulse-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--success);
    box-shadow: 0 0 0 0 var(--success);
    animation: pulse 1.6s ease-out infinite;
  }

  @keyframes pulse {
    0% {
      box-shadow: 0 0 0 0 rgba(52, 199, 89, 0.55);
    }
    70% {
      box-shadow: 0 0 0 8px rgba(52, 199, 89, 0);
    }
    100% {
      box-shadow: 0 0 0 0 rgba(52, 199, 89, 0);
    }
  }
</style>
