<script lang="ts">
  import Box from '@lucide/svelte/icons/box';
  import Eye from '@lucide/svelte/icons/eye';

  import CopyButton from '$lib/components/common/CopyButton.svelte';
  import Sparkline from '$lib/components/common/Sparkline.svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Progress } from '$lib/components/ui/progress';
  import { appStore } from '$lib/stores/app.svelte';
  import { liveStats } from '$lib/stores/liveStats.svelte';
  import { serverStore } from '$lib/stores/server.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import type { Recipe } from '$lib/types';
  import { cn } from '$lib/utils/cn';
  import { summarizeCommand, tokenize } from '$lib/utils/flags';
  import { logLevel } from '$lib/utils/logs';
  import { fileName, quantOf } from '$lib/utils/model';
  import { previewCommand } from '$lib/utils/preview';
  import { webuiUrl } from '$lib/utils/server';

  let { recipe }: { recipe: Recipe } = $props();

  const settings = $derived(settingsStore.current);
  const isReady = $derived(serverStore.isReady(recipe.id));
  const summary = $derived(summarizeCommand(recipe.command));
  const launched = $derived(
    previewCommand(settings, recipe.command, recipe.model_path, recipe.mmproj_path),
  );
  const launchedTokens = $derived(tokenize(launched).map((t) => t.value));
  const base = $derived(webuiUrl(settings));
  const endpoints = $derived([
    ...(settings.webui_enabled ? [{ label: 'Web', url: base }] : []),
    { label: 'API', url: `${base}/v1` },
  ]);
  const recentLogs = $derived(serverStore.logs.filter((l) => l.recipe_id === recipe.id).slice(-6));
  const nCtx = $derived(liveStats.props?.default_generation_settings?.n_ctx ?? null);
  const totalSlots = $derived(liveStats.props?.total_slots ?? 0);
  const quant = $derived(quantOf(recipe.model_path));
</script>

<div class="grid gap-4 lg:grid-cols-[minmax(0,1fr)_320px]">
  <div class="flex min-w-0 flex-col gap-4">
    <Card.Root class="py-4">
      <Card.Content class="flex items-center gap-3 px-4">
        <div class="grid size-10 shrink-0 place-items-center rounded-lg bg-primary/10 text-primary">
          <Box class="size-5" />
        </div>
        <div class="min-w-0 flex-1">
          <div class="truncate font-medium" title={recipe.model_path}>
            {recipe.model_path ? fileName(recipe.model_path) : 'No model selected'}
          </div>
          <div class="truncate font-mono text-[11px] text-faint" title={recipe.model_path}>
            {recipe.model_path}
          </div>
        </div>
        <div class="flex shrink-0 gap-4 text-xs">
          {#if quant}
            <div class="text-right">
              <div class="text-faint">Quant</div>
              <div class="font-mono">{quant}</div>
            </div>
          {/if}
          {#if nCtx}
            <div class="text-right">
              <div class="text-faint">Context</div>
              <div class="font-mono">{nCtx.toLocaleString()}</div>
            </div>
          {/if}
        </div>
      </Card.Content>
      {#if recipe.mmproj_path}
        <Card.Content class="flex items-center gap-2 border-t px-4 pt-3 text-xs">
          <Eye class="size-3.5 text-primary" />
          <span class="text-muted-foreground">Vision projector</span>
          <span class="truncate font-mono text-faint" title={recipe.mmproj_path}>
            {fileName(recipe.mmproj_path)}
          </span>
        </Card.Content>
      {/if}
    </Card.Root>

    <Card.Root class="gap-3 py-4">
      <Card.Header class="px-4">
        <Card.Title class="text-xs font-medium">Parameters</Card.Title>
      </Card.Header>
      <Card.Content class="px-4">
        {#if summary.params.length === 0 && summary.other.length === 0}
          <p class="text-xs text-faint">No extra flags — llama-server defaults apply.</p>
        {:else}
          <div class="grid grid-cols-2 gap-2 xl:grid-cols-3">
            {#each summary.params as p (p.spec.key)}
              <div class="rounded-md border bg-background/40 px-3 py-2">
                <div class="text-[11px] text-faint">{p.spec.label}</div>
                <div class="flex items-baseline justify-between gap-2">
                  <span class="truncate font-mono text-sm">{p.value}</span>
                  <code class="font-mono text-[10.5px] text-faint">{p.spec.flags[0]}</code>
                </div>
              </div>
            {/each}
          </div>
          {#if summary.other.length}
            <div class="mt-3 flex flex-wrap gap-1.5">
              {#each summary.other as flag, i (i)}
                <code
                  class="rounded bg-code px-1.5 py-0.5 font-mono text-[11px] text-muted-foreground"
                  >{flag}</code
                >
              {/each}
            </div>
          {/if}
        {/if}
      </Card.Content>
    </Card.Root>

    <Card.Root class="gap-2 py-4">
      <Card.Header class="flex items-center justify-between px-4">
        <Card.Title class="text-xs font-medium">Command</Card.Title>
        <CopyButton text={launched} label="Copy" showLabel />
      </Card.Header>
      <Card.Content class="px-4">
        <pre
          class="overflow-x-auto rounded-md bg-code p-3 font-mono text-[11.5px] leading-relaxed wrap-anywhere whitespace-pre-wrap select-text">{#each launchedTokens as tok, i (i)}<span
              class={tok.startsWith('-') ? 'text-primary' : ''}>{tok}</span
            >{i < launchedTokens.length - 1 ? ' ' : ''}{/each}</pre>
      </Card.Content>
    </Card.Root>
  </div>

  <div class="flex flex-col gap-4">
    <Card.Root class="gap-2 py-4">
      <Card.Header class="px-4">
        <Card.Title class="text-xs font-medium">Generation</Card.Title>
      </Card.Header>
      <Card.Content class="px-4">
        {#if isReady && settings.metrics_enabled}
          <div class="flex items-baseline gap-1">
            <span class="font-mono text-3xl font-semibold tabular-nums">
              {liveStats.recent.genTokPerSec.toFixed(1)}
            </span>
            <span class="text-xs text-muted-foreground">tok/s</span>
          </div>
          <Sparkline values={liveStats.series.gen.slice(-30)} height={36} class="my-2" />
          <dl class="grid grid-cols-2 gap-2 text-xs">
            <div>
              <dt class="text-faint">Prompt</dt>
              <dd class="font-mono">{liveStats.recent.promptTokPerSec.toFixed(1)} t/s</dd>
            </div>
            <div>
              <dt class="text-faint">Slots busy</dt>
              <dd class="font-mono">{liveStats.activeSlots} / {totalSlots || '—'}</dd>
            </div>
          </dl>
          {#if totalSlots}
            <Progress value={(liveStats.activeSlots / totalSlots) * 100} class="mt-3 h-1.5" />
          {/if}
          <Button
            variant="link"
            size="xs"
            class="mt-2 px-0"
            onclick={() => (appStore.tab = 'stats')}
          >
            Open live stats
          </Button>
        {:else if isReady}
          <p class="text-xs text-faint">
            Enable the Prometheus metrics endpoint in Settings to see throughput.
          </p>
        {:else}
          <p class="text-xs text-faint">Start the server to see live throughput.</p>
        {/if}
      </Card.Content>
    </Card.Root>

    <Card.Root class="gap-2 py-4">
      <Card.Header class="px-4">
        <Card.Title class="text-xs font-medium">Endpoints</Card.Title>
      </Card.Header>
      <Card.Content class="flex flex-col gap-1.5 px-4">
        {#each endpoints as ep (ep.label)}
          <div class="flex items-center gap-2 rounded-md bg-code px-2 py-1">
            <span class="w-8 text-[10px] font-semibold tracking-wide text-faint uppercase">
              {ep.label}
            </span>
            <code class="flex-1 truncate font-mono text-[11.5px] select-text">{ep.url}</code>
            <CopyButton text={ep.url} label="Copy {ep.label} URL" />
          </div>
        {/each}
      </Card.Content>
    </Card.Root>

    <Card.Root class="gap-2 py-4">
      <Card.Header class="flex items-center justify-between px-4">
        <Card.Title class="text-xs font-medium">Recent log</Card.Title>
        <Button variant="link" size="xs" class="px-0" onclick={() => (appStore.tab = 'logs')}>
          View all
        </Button>
      </Card.Header>
      <Card.Content class="px-4">
        {#if recentLogs.length === 0}
          <p class="text-xs text-faint">No output yet.</p>
        {:else}
          <div class="flex flex-col gap-0.5 font-mono text-[10.5px] leading-snug">
            {#each recentLogs as log, i (i)}
              {@const level = logLevel(log.line)}
              <div
                class={cn(
                  'truncate',
                  level === 'error'
                    ? 'text-destructive'
                    : level === 'warn'
                      ? 'text-warning'
                      : 'text-muted-foreground',
                )}
                title={log.line}
              >
                {log.line}
              </div>
            {/each}
          </div>
        {/if}
      </Card.Content>
    </Card.Root>
  </div>
</div>
