<script lang="ts">
  import Search from '@lucide/svelte/icons/search';

  import { Input } from '$lib/components/ui/input';
  import * as Select from '$lib/components/ui/select';
  import type { Backend, Capability, CommunityFilters } from '$lib/types';
  import { cn } from '$lib/utils/cn';

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

  const vram = [null, 6, 8, 12, 16, 24, 48];
  const vramLabel = (v: number | null) => (v === null ? 'Any VRAM' : `${v} GiB+`);
</script>

{#snippet chips(opts: { id: string; label: string }[], value: string, set: (v: string) => void)}
  <div class="flex flex-wrap gap-1">
    {#each opts as opt (opt.id)}
      <button
        type="button"
        class={cn(
          'rounded-full border px-2.5 py-0.5 text-xs text-muted-foreground hover:text-foreground',
          value === opt.id && 'border-primary/40 bg-primary/10 text-primary',
        )}
        onclick={() => set(opt.id)}
      >
        {opt.label}
      </button>
    {/each}
  </div>
{/snippet}

<aside class="flex w-[268px] shrink-0 flex-col gap-4 border-r bg-sidebar p-3">
  <h2 class="text-sm font-semibold">Community</h2>
  <div class="relative">
    <Search
      class="pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2 text-faint"
    />
    <Input
      bind:value={filters.search}
      placeholder="Search by GPU or model"
      class="h-7 pl-7 text-xs"
    />
  </div>
  <section class="flex flex-col gap-1.5">
    <h3 class="text-[10.5px] font-semibold tracking-wider text-faint uppercase">Backend</h3>
    {@render chips(backends, filters.backend, (v) => (filters.backend = v as Backend | 'any'))}
  </section>
  <section class="flex flex-col gap-1.5">
    <h3 class="text-[10.5px] font-semibold tracking-wider text-faint uppercase">Capability</h3>
    {@render chips(
      capabilities,
      filters.capability,
      (v) => (filters.capability = v as Capability | 'any'),
    )}
  </section>
  <section class="flex flex-col gap-1.5">
    <h3 class="text-[10.5px] font-semibold tracking-wider text-faint uppercase">VRAM</h3>
    <Select.Root
      type="single"
      value={String(filters.vramMinGib ?? 'any')}
      onValueChange={(v) => (filters.vramMinGib = v === 'any' ? null : Number(v))}
    >
      <Select.Trigger class="w-full">{vramLabel(filters.vramMinGib)}</Select.Trigger>
      <Select.Content>
        {#each vram as v (v ?? 'any')}
          <Select.Item value={String(v ?? 'any')}>{vramLabel(v)}</Select.Item>
        {/each}
      </Select.Content>
    </Select.Root>
  </section>
  <section class="flex flex-col gap-1.5">
    <h3 class="text-[10.5px] font-semibold tracking-wider text-faint uppercase">Sort</h3>
    <Select.Root
      type="single"
      value={filters.sort}
      onValueChange={(v) => (filters.sort = v as CommunityFilters['sort'])}
    >
      <Select.Trigger class="w-full">
        {sorts.find((s) => s.id === filters.sort)?.label}
      </Select.Trigger>
      <Select.Content>
        {#each sorts as s (s.id)}
          <Select.Item value={s.id}>{s.label}</Select.Item>
        {/each}
      </Select.Content>
    </Select.Root>
  </section>
</aside>
