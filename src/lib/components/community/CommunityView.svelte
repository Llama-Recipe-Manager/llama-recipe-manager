<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import type { CommunityFilters } from '$lib/types';

  let { filters }: { filters: CommunityFilters } = $props();

  const activeFilterCount = $derived(
    (filters.search ? 1 : 0) +
      (filters.backend !== 'any' ? 1 : 0) +
      (filters.capability !== 'any' ? 1 : 0) +
      (filters.vramMinGib !== null ? 1 : 0),
  );

  const samples = [
    {
      name: 'Llama-3.1-8B Q4_K_M',
      backend: 'CUDA',
      hw: 'RTX 4090 · 24 GiB',
      tg: 132,
      pp: '3 240',
      forks: 87,
    },
    {
      name: 'Qwen2-VL-7B vision',
      backend: 'Vulkan',
      hw: 'RX 7900 XTX · 24 GiB',
      tg: 71,
      pp: '1 980',
      forks: 42,
    },
    {
      name: 'Mistral-Small-24B Q5',
      backend: 'Metal',
      hw: 'M3 Max · 64 GiB',
      tg: 38,
      pp: '910',
      forks: 118,
    },
  ];
</script>

<div class="flex-1 overflow-y-auto px-8 py-6">
  <h1 class="text-xl font-semibold tracking-tight">Community recipes</h1>
  <p class="mt-1 text-muted-foreground">
    Browse, fork, and validate recipes shared by the community.
    {#if activeFilterCount > 0}
      <span class="ml-2 text-primary">
        {activeFilterCount} filter{activeFilterCount === 1 ? '' : 's'} active
      </span>
    {/if}
  </p>

  <Card.Root class="mt-6 max-w-2xl border-dashed">
    <Card.Header>
      <Badge variant="secondary" class="mb-2">Coming soon</Badge>
      <Card.Title>Community recipes are on the way</Card.Title>
      <Card.Description>
        Sign in to publish your own recipes, fork what others share, and validate their performance
        claims on your hardware.
      </Card.Description>
    </Card.Header>
    <Card.Footer class="gap-2">
      <Button disabled>Sign in (coming soon)</Button>
    </Card.Footer>
  </Card.Root>

  <div class="mt-6 grid grid-cols-1 gap-3 opacity-60 md:grid-cols-3" aria-hidden="true">
    {#each samples as s (s.name)}
      <Card.Root class="gap-2 py-4">
        <Card.Content class="px-4">
          <div class="flex items-center justify-between gap-2">
            <span class="truncate font-medium">{s.name}</span>
            <Badge variant="outline">{s.backend}</Badge>
          </div>
          <p class="mt-1 text-xs text-faint">@example · {s.hw}</p>
          <div class="mt-3 grid grid-cols-3 text-xs">
            <div>
              <div class="font-mono font-semibold">{s.tg}</div>
              <div class="text-faint">tg t/s</div>
            </div>
            <div>
              <div class="font-mono font-semibold">{s.pp}</div>
              <div class="text-faint">pp t/s</div>
            </div>
            <div>
              <div class="font-mono font-semibold">{s.forks}</div>
              <div class="text-faint">forks</div>
            </div>
          </div>
        </Card.Content>
      </Card.Root>
    {/each}
  </div>
</div>
