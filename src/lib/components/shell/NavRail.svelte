<script lang="ts">
  import type { Component } from 'svelte';
  import BookOpen from '@lucide/svelte/icons/book-open';
  import Boxes from '@lucide/svelte/icons/boxes';
  import Globe from '@lucide/svelte/icons/globe';
  import Info from '@lucide/svelte/icons/info';
  import Settings from '@lucide/svelte/icons/settings';

  import * as Tooltip from '$lib/components/ui/tooltip';
  import { FEATURES } from '$lib/featureFlags';
  import { appStore } from '$lib/stores/app.svelte';
  import type { Section } from '$lib/types';
  import { cn } from '$lib/utils/cn';

  type Item = { id: Section; label: string; icon: Component<{ class?: string }> };

  const top: Item[] = [
    { id: 'recipes', label: 'Recipes', icon: BookOpen },
    { id: 'models', label: 'Models', icon: Boxes },
    ...(FEATURES.community ? [{ id: 'community', label: 'Community', icon: Globe } as const] : []),
  ];
</script>

{#snippet railButton(label: string, Icon: Item['icon'], active: boolean, onclick: () => void)}
  <Tooltip.Root>
    <Tooltip.Trigger
      class={cn(
        'relative grid size-9 place-items-center rounded-lg text-muted-foreground transition-colors hover:bg-accent hover:text-foreground',
        active &&
          'bg-accent text-foreground before:absolute before:top-2 before:-left-2.5 before:h-5 before:w-[3px] before:rounded-r before:bg-primary',
      )}
      aria-label={label}
      aria-current={active ? 'page' : undefined}
      {onclick}
    >
      <Icon class="size-[18px]" />
    </Tooltip.Trigger>
    <Tooltip.Content side="right">{label}</Tooltip.Content>
  </Tooltip.Root>
{/snippet}

<nav aria-label="Primary" class="flex w-14 shrink-0 flex-col items-center gap-1 border-r py-3">
  {#each top as item (item.id)}
    {@render railButton(item.label, item.icon, appStore.section === item.id, () =>
      appStore.goSection(item.id),
    )}
  {/each}
  <div class="flex-1"></div>
  {@render railButton('About', Info, false, () => (appStore.aboutOpen = true))}
  {@render railButton('Settings', Settings, appStore.section === 'settings', () =>
    appStore.goSection('settings'),
  )}
</nav>
