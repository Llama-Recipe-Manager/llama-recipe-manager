<script lang="ts">
  import CircleAlert from '@lucide/svelte/icons/circle-alert';

  import ModelFileField from './ModelFileField.svelte';
  import ParamFields from './ParamFields.svelte';
  import CopyButton from '$lib/components/common/CopyButton.svelte';
  import Field from '$lib/components/common/Field.svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Kbd } from '$lib/components/ui/kbd';
  import * as Select from '$lib/components/ui/select';
  import { Switch } from '$lib/components/ui/switch';
  import { Label } from '$lib/components/ui/label';
  import { Textarea } from '$lib/components/ui/textarea';
  import { createRecipe, updateRecipe } from '$lib/api';
  import { appStore } from '$lib/stores/app.svelte';
  import { recipesStore } from '$lib/stores/recipes.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import type { Recipe } from '$lib/types';
  import { ALL_GPU_VALUES, GPU_CUSTOM_VALUE, GPU_OPTIONS } from '$lib/utils/gpuOptions';
  import { fileName } from '$lib/utils/model';
  import { notifyError } from '$lib/utils/notify';
  import { modKey } from '$lib/utils/platform';
  import { previewCommand } from '$lib/utils/preview';
  import { validateCommand } from '$lib/utils/validate';

  const GPU_NONE = '__none__';

  let {
    mode,
    initial = null,
    initialModel = null,
  }: { mode: 'new' | 'edit'; initial?: Recipe | null; initialModel?: string | null } = $props();

  // Local copies so edits don't touch the store until save() succeeds.
  /* svelte-ignore state_referenced_locally */
  let name = $state(
    initial?.name ?? (initialModel ? fileName(initialModel).replace(/\.gguf$/i, '') : ''),
  );
  /* svelte-ignore state_referenced_locally */
  let description = $state(initial?.description ?? '');
  /* svelte-ignore state_referenced_locally */
  let command = $state(initial?.command ?? '');
  /* svelte-ignore state_referenced_locally */
  let modelPath = $state(initial?.model_path ?? initialModel ?? '');
  /* svelte-ignore state_referenced_locally */
  let vision = $state(!!initial?.mmproj_path);
  /* svelte-ignore state_referenced_locally */
  let mmprojPath = $state(initial?.mmproj_path ?? '');
  /* svelte-ignore state_referenced_locally */
  let gpuInfo = $state(initial?.gpu_info ?? '');
  /* svelte-ignore state_referenced_locally */
  let tags = $state(initial?.tags ?? '');
  /* svelte-ignore state_referenced_locally */
  let gpuSelect = $state(
    !gpuInfo ? GPU_NONE : ALL_GPU_VALUES.includes(gpuInfo) ? gpuInfo : GPU_CUSTOM_VALUE,
  );

  let errors = $state<{ name?: string; model?: string; mmproj?: string; command?: string[] }>({});
  let saving = $state(false);

  const snapshot = () =>
    JSON.stringify([
      name,
      description,
      command,
      modelPath,
      vision ? mmprojPath : '',
      gpuInfo,
      tags,
    ]);
  const pristine = snapshot();

  $effect(() => {
    appStore.formDirty = snapshot() !== pristine;
  });

  const commandErrors = $derived(validateCommand(command));
  const preview = $derived(
    previewCommand(settingsStore.current, command, modelPath, vision ? mmprojPath : ''),
  );

  function onGpuChange(v: string) {
    gpuSelect = v;
    if (v === GPU_NONE) gpuInfo = '';
    else if (v !== GPU_CUSTOM_VALUE) gpuInfo = v;
  }

  async function cancel() {
    if (!(await appStore.confirmDiscard())) return;
    appStore.finishEdit();
  }

  async function save() {
    if (saving) return;
    errors = {
      name: name.trim() ? undefined : 'Name is required',
      model: modelPath.trim() ? undefined : 'Choose a model file',
      mmproj: vision && !mmprojPath.trim() ? 'Choose an mmproj file or turn vision off' : undefined,
      command: commandErrors.length ? commandErrors : undefined,
    };
    if (errors.name || errors.model || errors.mmproj || errors.command) return;

    saving = true;
    try {
      const payload = {
        name: name.trim(),
        description,
        command: command.trim(),
        model_path: modelPath,
        mmproj_path: vision ? mmprojPath : '',
        gpu_info: gpuInfo,
        tags,
      };
      const saved =
        mode === 'edit' && initial
          ? await updateRecipe({ id: initial.id, ...payload })
          : await createRecipe(payload);
      await recipesStore.refresh();
      appStore.finishEdit(saved.id);
    } catch (e) {
      notifyError(e);
    } finally {
      saving = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 's') {
      e.preventDefault();
      void save();
    } else if (
      e.key === 'Escape' &&
      !e.defaultPrevented &&
      !document.querySelector(
        '[role="dialog"], [role="alertdialog"], [role="listbox"], [role="menu"]',
      )
    ) {
      void cancel();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex min-h-0 flex-1 flex-col">
  <header class="flex items-center gap-3 border-b px-8 py-3">
    <div class="min-w-0 flex-1">
      <h1 class="truncate text-base font-semibold">
        {mode === 'new' ? 'New recipe' : `Edit ${initial?.name ?? 'recipe'}`}
      </h1>
    </div>
    <Button variant="ghost" onclick={cancel} aria-keyshortcuts="Escape">Cancel</Button>
    <Button onclick={save} disabled={saving} aria-keyshortcuts="Meta+S Control+S">
      {saving ? 'Saving…' : 'Save'}
      <Kbd class="bg-primary-foreground/15 text-primary-foreground">{modKey}S</Kbd>
    </Button>
  </header>

  <div class="min-h-0 flex-1 overflow-y-auto">
    <div class="mx-auto flex max-w-[960px] flex-col gap-4 px-8 py-6">
      <Card.Root class="py-5">
        <Card.Content class="grid grid-cols-2 gap-4 px-5">
          <Field label="Name" for="recipe-name" error={errors.name} class="col-span-2">
            <Input
              id="recipe-name"
              bind:value={name}
              placeholder="e.g. Qwen3 30B · coding"
              aria-invalid={!!errors.name || undefined}
            />
          </Field>
          <Field label="Description" for="recipe-desc" class="col-span-2">
            <Textarea
              id="recipe-desc"
              rows={2}
              bind:value={description}
              placeholder="What is this recipe for?"
            />
          </Field>
          <Field label="Hardware" for="recipe-gpu">
            <Select.Root type="single" value={gpuSelect} onValueChange={onGpuChange}>
              <Select.Trigger id="recipe-gpu" class="w-full">
                {gpuSelect === GPU_CUSTOM_VALUE
                  ? 'Other…'
                  : gpuSelect === GPU_NONE
                    ? 'Not specified'
                    : gpuSelect}
              </Select.Trigger>
              <Select.Content class="max-h-80">
                <Select.Item value={GPU_NONE}>Not specified</Select.Item>
                {#each GPU_OPTIONS as grp (grp.group)}
                  <Select.Group>
                    <Select.GroupHeading>{grp.group}</Select.GroupHeading>
                    {#each grp.items as item (item)}
                      <Select.Item value={item}>{item}</Select.Item>
                    {/each}
                  </Select.Group>
                {/each}
                <Select.Separator />
                <Select.Item value={GPU_CUSTOM_VALUE}>Other…</Select.Item>
              </Select.Content>
            </Select.Root>
            {#if gpuSelect === GPU_CUSTOM_VALUE}
              <Input bind:value={gpuInfo} placeholder="Describe your hardware" class="mt-2" />
            {/if}
          </Field>
          <Field label="Tags" for="recipe-tags" hint="Comma separated">
            <Input id="recipe-tags" bind:value={tags} placeholder="qwen, coding, 30b" />
          </Field>
        </Card.Content>
      </Card.Root>

      <Card.Root class="gap-4 py-5">
        <Card.Header class="px-5">
          <Card.Title class="text-sm">Model</Card.Title>
        </Card.Header>
        <Card.Content class="flex flex-col gap-4 px-5">
          <Field label="GGUF file" for="recipe-model" error={errors.model}>
            <ModelFileField
              id="recipe-model"
              bind:value={modelPath}
              kind="model"
              placeholder="/path/to/model.gguf"
              invalid={!!errors.model}
            />
          </Field>
          <div class="flex items-center gap-3">
            <Switch id="recipe-vision" bind:checked={vision} />
            <Label for="recipe-vision" class="flex flex-col items-start gap-0.5">
              <span>Vision</span>
              <span class="text-xs font-normal text-faint">
                Load a multimodal projector, passed as --mmproj
              </span>
            </Label>
          </div>
          {#if vision}
            <Field label="mmproj file" for="recipe-mmproj" error={errors.mmproj}>
              <ModelFileField
                id="recipe-mmproj"
                bind:value={mmprojPath}
                kind="mmproj"
                placeholder="/path/to/mmproj.gguf"
                invalid={!!errors.mmproj}
              />
            </Field>
          {/if}
        </Card.Content>
      </Card.Root>

      <Card.Root class="gap-4 py-5">
        <Card.Header class="px-5">
          <Card.Title class="text-sm">Parameters</Card.Title>
          <Card.Description class="text-xs">
            Common llama-server flags. Leave a field empty to use the server default.
          </Card.Description>
        </Card.Header>
        <Card.Content class="px-5">
          <ParamFields bind:command />
        </Card.Content>
      </Card.Root>

      <Card.Root class="gap-4 py-5">
        <Card.Header class="px-5">
          <Card.Title class="text-sm">Arguments</Card.Title>
          <Card.Description class="text-xs">
            The full flag list, kept in sync with the fields above. Add anything else llama-server
            supports here. --host, --port, -m and --mmproj are managed by the app.
          </Card.Description>
        </Card.Header>
        <Card.Content class="flex flex-col gap-3 px-5">
          <Textarea
            id="recipe-command"
            rows={4}
            bind:value={command}
            placeholder="-ngl 99 -c 8192 -fa on --temp 0.6"
            class="font-mono text-xs"
            spellcheck="false"
            aria-invalid={commandErrors.length > 0 || undefined}
          />
          {#if commandErrors.length}
            <ul class="flex flex-col gap-1 text-xs text-destructive">
              {#each commandErrors as err, i (i)}
                <li class="flex items-start gap-1.5">
                  <CircleAlert class="mt-px size-3.5 shrink-0" />{err}
                </li>
              {/each}
            </ul>
          {/if}
          <div>
            <div class="mb-1 flex items-center justify-between">
              <span class="text-xs font-medium text-muted-foreground">Launch preview</span>
              <CopyButton text={preview} />
            </div>
            <pre
              class="rounded-md bg-code p-3 font-mono text-[11px] leading-relaxed break-all whitespace-pre-wrap text-muted-foreground select-text">{preview}</pre>
          </div>
        </Card.Content>
      </Card.Root>
    </div>
  </div>
</div>
