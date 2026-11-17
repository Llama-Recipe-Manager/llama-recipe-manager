<script lang="ts">
  import Icon from './Icon.svelte';
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
          <Icon name="home" size={20} strokeWidth={1.8} />
        {:else if item.id === 'community'}
          <Icon name="globe" size={20} strokeWidth={1.8} />
        {:else}
          <Icon name="settings" size={20} strokeWidth={1.8} />
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
      <Icon name="info" size={20} strokeWidth={1.8} />
    </button>
    {#if FEATURES.account}
      <button
        type="button"
        class="rail-btn"
        title="Sign in (coming soon)"
        aria-label="Sign in"
        disabled
      >
        <Icon name="user" size={20} strokeWidth={1.8} />
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
