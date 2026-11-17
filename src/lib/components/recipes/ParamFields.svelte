<script lang="ts">
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import * as Select from '$lib/components/ui/select';
  import { Switch } from '$lib/components/ui/switch';
  import {
    PARAM_GROUPS,
    PARAMS,
    readParam,
    writeParam,
    type ParamSpec,
    type ParamValue,
  } from '$lib/utils/flags';

  let { command = $bindable('') }: { command: string } = $props();

  const DEFAULT = '__default__';

  const values = $derived(
    Object.fromEntries(PARAMS.map((p) => [p.key, readParam(command, p)])) as Record<
      string,
      ParamValue
    >,
  );

  let drafts = $state<Record<string, string>>({});

  function set(spec: ParamSpec, value: ParamValue) {
    if (typeof value === 'string' && value.startsWith('-') && !/^-\d/.test(value)) return;
    command = writeParam(command, spec, value);
  }

  const groups = PARAM_GROUPS.map((g) => ({
    name: g,
    params: PARAMS.filter((p) => p.group === g),
  }));
</script>

<div class="flex flex-col gap-5">
  {#each groups as group (group.name)}
    <section>
      <h4 class="mb-2 text-[11px] font-semibold tracking-wider text-faint uppercase">
        {group.name}
      </h4>
      <div class="grid grid-cols-2 gap-x-4 gap-y-3 lg:grid-cols-3">
        {#each group.params as spec (spec.key)}
          {@const v = values[spec.key]}
          {@const fid = `param-${spec.key}`}
          {#if spec.type === 'bool'}
            <div class="flex items-center justify-between gap-2 rounded-md border px-3 py-2">
              <Label for={fid} class="flex flex-col items-start gap-0.5">
                <span class="text-xs font-medium">{spec.label}</span>
                <code class="font-mono text-[10.5px] text-faint">{spec.flags[0]}</code>
              </Label>
              <Switch id={fid} checked={v === true} onCheckedChange={(c) => set(spec, c)} />
            </div>
          {:else}
            <div class="flex flex-col gap-1.5">
              <Label for={fid} class="flex items-baseline justify-between text-xs">
                <span class="font-medium text-muted-foreground">{spec.label}</span>
                <code class="font-mono text-[10.5px] text-faint">{spec.flags[0]}</code>
              </Label>
              {#if spec.type === 'select'}
                <Select.Root
                  type="single"
                  value={typeof v === 'string' ? v : DEFAULT}
                  onValueChange={(next) => set(spec, next === DEFAULT ? null : next)}
                >
                  <Select.Trigger id={fid} class="h-8 w-full font-mono text-xs">
                    {typeof v === 'string' ? v : 'default'}
                  </Select.Trigger>
                  <Select.Content>
                    <Select.Item value={DEFAULT} class="text-muted-foreground">default</Select.Item>
                    {#each spec.options ?? [] as opt (opt)}
                      <Select.Item value={opt} class="font-mono">{opt}</Select.Item>
                    {/each}
                  </Select.Content>
                </Select.Root>
              {:else}
                <Input
                  id={fid}
                  type="text"
                  inputmode={spec.type === 'number' ? 'decimal' : undefined}
                  class="h-8 font-mono text-xs"
                  placeholder={spec.placeholder ?? 'default'}
                  value={drafts[spec.key] ?? (typeof v === 'string' ? v : '')}
                  oninput={(e) => {
                    drafts[spec.key] = e.currentTarget.value;
                    set(spec, e.currentTarget.value.trim() || null);
                  }}
                  onblur={() => delete drafts[spec.key]}
                  spellcheck="false"
                  autocomplete="off"
                />
              {/if}
            </div>
          {/if}
        {/each}
      </div>
    </section>
  {/each}
</div>
