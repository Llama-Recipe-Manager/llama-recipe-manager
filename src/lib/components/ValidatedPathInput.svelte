<script lang="ts">
  import type { LlamaPathStatus } from '$lib/types';

  let {
    id,
    value = $bindable(''),
    placeholder = '',
    status,
    errorMessage = '',
    onInput,
  }: {
    id: string;
    value: string;
    placeholder?: string;
    status: LlamaPathStatus;
    errorMessage?: string;
    onInput?: () => void;
  } = $props();
</script>

<div class="input-with-status">
  <input
    {id}
    type="text"
    bind:value
    {placeholder}
    oninput={onInput}
    class:input-error={status === 'error'}
    class:input-ok={status === 'ok'}
  />
  {#if status === 'checking'}
    <span class="path-status checking">Checking...</span>
  {:else if status === 'ok'}
    <span class="path-status ok">Reachable</span>
  {:else if status === 'error'}
    <span class="path-status error">Not found</span>
  {/if}
</div>

{#if status === 'error' && errorMessage}
  <span class="form-hint" style="color: var(--danger)">{errorMessage}</span>
{/if}

<style>
  .input-with-status {
    position: relative;
    display: flex;
    align-items: center;
  }

  .input-with-status input {
    flex: 1;
  }

  .path-status {
    position: absolute;
    right: 10px;
    font-size: 11px;
    font-weight: 500;
    pointer-events: none;
  }

  .path-status.checking {
    color: var(--text-tertiary);
  }

  .path-status.ok {
    color: var(--success);
  }

  .path-status.error {
    color: var(--danger);
  }

  input.input-error {
    border-color: var(--danger);
  }

  input.input-ok {
    border-color: var(--success);
  }

  .form-hint {
    font-size: 11px;
    color: var(--text-tertiary);
  }
</style>
