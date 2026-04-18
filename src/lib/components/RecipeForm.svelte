<script lang="ts">
  import FilePickerInput from './FilePickerInput.svelte';
  import { createRecipe, updateRecipe } from '$lib/api';
  import { recipesStore } from '$lib/stores/recipes.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import type { Recipe } from '$lib/types';
  import { errorMessage } from '$lib/utils/format';
  import { previewCommand } from '$lib/utils/preview';
  import { validateCommand } from '$lib/utils/validate';
  import { ALL_GPU_VALUES, GPU_CUSTOM_VALUE, GPU_OPTIONS } from '$lib/utils/gpuOptions';

  let {
    mode,
    initial = null,
    onCancel,
    onSaved,
    onError,
  }: {
    mode: 'new' | 'edit';
    initial?: Recipe | null;
    onCancel: () => void;
    onSaved: (id: string) => void;
    onError: (msg: string) => void;
  } = $props();

  // The form deliberately holds its own editable copies of the initial recipe
  // so edits don't mutate the upstream store until save() succeeds.
  /* svelte-ignore state_referenced_locally */
  let name = $state(initial?.name ?? '');
  /* svelte-ignore state_referenced_locally */
  let description = $state(initial?.description ?? '');
  /* svelte-ignore state_referenced_locally */
  let command = $state(initial?.command ?? '');
  /* svelte-ignore state_referenced_locally */
  let modelPath = $state(initial?.model_path ?? '');
  /* svelte-ignore state_referenced_locally */
  let vision = $state(!!initial?.mmproj_path);
  /* svelte-ignore state_referenced_locally */
  let mmprojPath = $state(initial?.mmproj_path ?? '');
  /* svelte-ignore state_referenced_locally */
  let gpuInfo = $state(initial?.gpu_info ?? '');
  /* svelte-ignore state_referenced_locally */
  let tags = $state(initial?.tags ?? '');
  let formErrors = $state<string[]>([]);

  /* svelte-ignore state_referenced_locally */
  let gpuSelect = $state(
    !gpuInfo ? '' : ALL_GPU_VALUES.includes(gpuInfo) ? gpuInfo : GPU_CUSTOM_VALUE,
  );

  function onGpuSelectChange() {
    if (gpuSelect !== GPU_CUSTOM_VALUE) {
      gpuInfo = gpuSelect;
    }
  }

  async function save() {
    formErrors = validateCommand(command);
    if (formErrors.length > 0) return;

    if (!name.trim()) {
      formErrors = ['Name is required'];
      return;
    }
    if (!modelPath.trim()) {
      formErrors = ['Model path is required'];
      return;
    }
    if (vision && !mmprojPath.trim()) {
      formErrors = ['Vision is enabled but no mmproj file selected'];
      return;
    }

    try {
      const payload = {
        name,
        description,
        command,
        model_path: modelPath,
        mmproj_path: vision ? mmprojPath : '',
        gpu_info: gpuInfo,
        tags,
      };

      if (mode === 'new') {
        const r = await createRecipe(payload);
        await recipesStore.refresh();
        onSaved(r.id);
      } else if (initial) {
        await updateRecipe({ id: initial.id, ...payload });
        await recipesStore.refresh();
        onSaved(initial.id);
      }
    } catch (e) {
      onError(errorMessage(e));
    }
  }
</script>

<div class="form-view">
  <div class="form-header">
    <h2>{mode === 'new' ? 'New Recipe' : 'Edit Recipe'}</h2>
    <div class="form-actions">
      <button class="btn secondary" onclick={onCancel}>Cancel</button>
      <button class="btn primary" onclick={save}>Save</button>
    </div>
  </div>

  {#if formErrors.length > 0}
    <div class="form-errors">
      {#each formErrors as err}
        <div class="form-error-line">{err}</div>
      {/each}
    </div>
  {/if}

  <div class="form-body">
    <div class="form-group">
      <label for="name">Name</label>
      <input id="name" type="text" bind:value={name} placeholder="e.g. Qwen3 35B Q4" />
    </div>

    <div class="form-group">
      <label for="description">Description</label>
      <textarea
        id="description"
        rows="2"
        bind:value={description}
        placeholder="What is this recipe for?"
      ></textarea>
    </div>

    <div class="form-group">
      <label for="model_path">Model File</label>
      <FilePickerInput
        bind:value={modelPath}
        placeholder="No model selected"
        defaultPath={settingsStore.current.model_dir || undefined}
      />
      <span class="form-hint">Select a .gguf model file. Injected as -m at runtime.</span>
    </div>

    <div class="form-group">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={vision} />
        <span>Vision (mmproj)</span>
      </label>
      {#if vision}
        <div style="margin-top: 6px">
          <FilePickerInput
            bind:value={mmprojPath}
            placeholder="No mmproj file selected"
            defaultPath={settingsStore.current.model_dir || undefined}
          />
        </div>
        <span class="form-hint"
          >Select the mmproj .gguf file for vision. Injected as --mmproj at runtime.</span
        >
      {/if}
    </div>

    <div class="form-group">
      <label for="command">Recipe Arguments</label>
      <textarea
        id="command"
        rows="4"
        bind:value={command}
        placeholder="-ngl 99 -c 8192 -fa on --temp 0.6"
        class="mono"
      ></textarea>
      <span class="form-hint">
        llama-server flags only. Do not include --host, --port, -m, or --mmproj &mdash; those are
        managed by the app. You can include "llama-server" at the start but it will be replaced by
        the configured path.
      </span>
    </div>

    {#if command.trim()}
      <div class="command-preview">
        <h4>Command Preview</h4>
        <pre class="command-block">{previewCommand(
            settingsStore.current,
            command,
            modelPath,
            vision ? mmprojPath : '',
          )}</pre>
      </div>
    {/if}

    <div class="form-row">
      <div class="form-group flex-1">
        <label for="gpu_info">GPU / Hardware</label>
        <select id="gpu_info" bind:value={gpuSelect} onchange={onGpuSelectChange}>
          <option value="">(none)</option>
          {#each GPU_OPTIONS as grp}
            <optgroup label={grp.group}>
              {#each grp.items as item}
                <option value={item}>{item}</option>
              {/each}
            </optgroup>
          {/each}
          <option value={GPU_CUSTOM_VALUE}>Other (custom)...</option>
        </select>
        {#if gpuSelect === GPU_CUSTOM_VALUE}
          <input
            type="text"
            bind:value={gpuInfo}
            placeholder="Describe your hardware"
            style="margin-top: 6px"
          />
        {/if}
      </div>
      <div class="form-group flex-1">
        <label for="tags">Tags</label>
        <input id="tags" type="text" bind:value={tags} placeholder="e.g. qwen, 35b, q4" />
        <span class="form-hint">Comma separated</span>
      </div>
    </div>
  </div>
</div>

<style>
  .form-view {
    padding: 24px 32px;
    max-width: 700px;
  }

  .form-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 24px;
  }

  .form-header h2 {
    font-size: 20px;
    font-weight: 700;
  }

  .form-actions {
    display: flex;
    gap: 8px;
  }

  .form-body {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .form-group label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .form-group input,
  .form-group textarea {
    width: 100%;
  }

  .form-group textarea.mono {
    font-family: var(--font-mono);
    font-size: 13px;
  }

  .form-hint {
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .form-row {
    display: flex;
    gap: 12px;
  }

  .flex-1 {
    flex: 1;
  }

  .form-errors {
    background: rgba(255, 59, 48, 0.08);
    border: 1px solid var(--danger);
    border-radius: var(--radius-sm);
    padding: 10px 14px;
    margin-bottom: 16px;
  }

  .form-error-line {
    font-size: 13px;
    color: var(--danger);
  }

  .command-preview h4 {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-tertiary);
    margin-bottom: 8px;
  }

  .command-block {
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 1.6;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 14px 16px;
    white-space: pre-wrap;
    word-break: break-all;
    user-select: text;
  }

  .checkbox-label {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
  }

  .checkbox-label input[type='checkbox'] {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
  }

  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 7px 16px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 500;
    transition: all 0.15s;
    white-space: nowrap;
  }

  .btn.primary {
    background: var(--accent);
    color: white;
  }

  .btn.secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .btn.secondary:hover {
    background: var(--border);
  }
</style>
