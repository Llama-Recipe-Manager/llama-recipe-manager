<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/plugin-dialog';
  import {
    listRecipes, createRecipe, updateRecipe, deleteRecipe, duplicateRecipe,
    startServer, stopServer, getServerStatus, getServerLogs, getSettings, updateSettings,
    getLlamaServerInfo,
    type Recipe, type ServerStatus, type LogLine, type Settings
  } from '$lib/api';

  // State
  let recipes = $state<Recipe[]>([]);
  let runningStatus = $state<ServerStatus | null>(null);
  let selectedId = $state<string | null>(null);
  let view = $state<'list' | 'edit' | 'new' | 'settings'>('list');
  let logs = $state<LogLine[]>([]);
  let showLogs = $state(false);
  let searchQuery = $state('');
  let error = $state('');

  // Settings state
  let settings = $state<Settings>({ host: '127.0.0.1', port: 8080, model_dir: '~/Models', llama_server_path: 'llama-server' });
  let settingsHost = $state('127.0.0.1');
  let settingsPort = $state(8080);
  let settingsModelDir = $state('~/Models');
  let settingsLlamaPath = $state('llama-server');
  let settingsSaved = $state(false);
  let llamaPathStatus = $state<'idle' | 'checking' | 'ok' | 'error'>('idle');
  let llamaPathError = $state('');

  // Recipe form state
  let formName = $state('');
  let formDescription = $state('');
  let formCommand = $state('');
  let formModelPath = $state('');
  let formVision = $state(false);
  let formMmprojPath = $state('');
  let formGpuInfo = $state('');
  let formTags = $state('');
  let formErrors = $state<string[]>([]);

  let unlistenStatus: UnlistenFn;
  let unlistenLog: UnlistenFn;
  let logContainer: HTMLDivElement;

  const selected = $derived(recipes.find(r => r.id === selectedId) ?? null);
  const filteredRecipes = $derived(
    searchQuery
      ? recipes.filter(r =>
          r.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          r.tags.toLowerCase().includes(searchQuery.toLowerCase()) ||
          r.gpu_info.toLowerCase().includes(searchQuery.toLowerCase())
        )
      : recipes
  );

  function isRunning(id: string): boolean {
    return runningStatus?.recipe_id === id && runningStatus?.running === true;
  }

  function anyServerRunning(): boolean {
    return runningStatus?.running === true;
  }

  function runningRecipeId(): string | null {
    return runningStatus?.running ? runningStatus.recipe_id : null;
  }

  // Forbidden flags that cannot be in a recipe command
  const FORBIDDEN_FLAGS = ['--port', '-p', '--host', '-m', '--model', '--model-path', '--mmproj', '--log-file'];

  function validateCommand(cmd: string): string[] {
    const errs: string[] = [];
    // Simple token-level check
    const tokens = cmd.split(/\s+/);
    for (const token of tokens) {
      const lower = token.toLowerCase();
      for (const flag of FORBIDDEN_FLAGS) {
        if (lower === flag || lower.startsWith(flag + '=')) {
          errs.push(`Command must not contain '${flag}' -- this is controlled by app settings.`);
        }
      }
    }
    return errs;
  }

  onMount(async () => {
    await refresh();

    unlistenStatus = await listen<ServerStatus>('server-status', (event) => {
      const s = event.payload;
      if (s.running) {
        runningStatus = s;
      } else {
        // If this is the server we knew about, clear it
        if (runningStatus?.recipe_id === s.recipe_id) {
          runningStatus = null;
        }
      }
    });

    unlistenLog = await listen<LogLine>('server-log', (event) => {
      const log = event.payload;
      if (showLogs) {
        logs = [...logs, log];
        requestAnimationFrame(() => {
          if (logContainer) {
            logContainer.scrollTop = logContainer.scrollHeight;
          }
        });
      }
    });
  });

  onDestroy(() => {
    unlistenStatus?.();
    unlistenLog?.();
  });

  async function refresh() {
    try {
      recipes = await listRecipes();
      runningStatus = await getServerStatus();
      settings = await getSettings();
    } catch (e: any) {
      error = e;
    }
  }

  function selectRecipe(id: string) {
    selectedId = id;
    view = 'list';
    showLogs = false;
    logs = [];
  }

  function openSettings() {
    settingsHost = settings.host;
    settingsPort = settings.port;
    settingsModelDir = settings.model_dir;
    settingsLlamaPath = settings.llama_server_path;
    settingsSaved = false;
    view = 'settings';
    selectedId = null;
    checkLlamaPath();
  }

  async function saveSettings() {
    error = '';
    try {
      settings = await updateSettings({
        host: settingsHost,
        port: settingsPort,
        model_dir: settingsModelDir,
        llama_server_path: settingsLlamaPath,
      });
      settingsSaved = true;
      setTimeout(() => settingsSaved = false, 2000);
      checkLlamaPath();
    } catch (e: any) {
      error = e;
    }
  }

  function startNew() {
    formName = '';
    formDescription = '';
    formCommand = '-ngl 99 -c 8192';
    formModelPath = '';
    formVision = false;
    formMmprojPath = '';
    formGpuInfo = '';
    formTags = '';
    formErrors = [];
    view = 'new';
    selectedId = null;
  }

  function startEdit() {
    if (!selected) return;
    formName = selected.name;
    formDescription = selected.description;
    formCommand = selected.command;
    formModelPath = selected.model_path;
    formVision = !!selected.mmproj_path;
    formMmprojPath = selected.mmproj_path;
    formGpuInfo = selected.gpu_info;
    formTags = selected.tags;
    formErrors = [];
    view = 'edit';
  }

  async function saveRecipe() {
    error = '';
    formErrors = validateCommand(formCommand);
    if (formErrors.length > 0) return;

    if (!formName.trim()) {
      formErrors = ['Name is required'];
      return;
    }

    if (!formModelPath.trim()) {
      formErrors = ['Model path is required'];
      return;
    }

    if (formVision && !formMmprojPath.trim()) {
      formErrors = ['Vision is enabled but no mmproj file selected'];
      return;
    }

    try {
      if (view === 'new') {
        const r = await createRecipe({
          name: formName,
          description: formDescription,
          command: formCommand,
          model_path: formModelPath,
          mmproj_path: formVision ? formMmprojPath : '',
          gpu_info: formGpuInfo,
          tags: formTags,
        });
        await refresh();
        selectedId = r.id;
        view = 'list';
      } else if (view === 'edit' && selected) {
        await updateRecipe({
          id: selected.id,
          name: formName,
          description: formDescription,
          command: formCommand,
          model_path: formModelPath,
          mmproj_path: formVision ? formMmprojPath : '',
          gpu_info: formGpuInfo,
          tags: formTags,
        });
        await refresh();
        view = 'list';
      }
    } catch (e: any) {
      error = typeof e === 'string' ? e : String(e);
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Delete this recipe?')) return;
    try {
      await deleteRecipe(id);
      if (selectedId === id) selectedId = null;
      await refresh();
    } catch (e: any) {
      error = e;
    }
  }

  async function handleDuplicate(id: string) {
    try {
      const r = await duplicateRecipe(id);
      await refresh();
      selectedId = r.id;
    } catch (e: any) {
      error = e;
    }
  }

  async function handleStart() {
    if (!selected) return;
    error = '';
    try {
      await startServer(selected.id, selected.command, selected.model_path, selected.mmproj_path);
    } catch (e: any) {
      error = typeof e === 'string' ? e : String(e);
    }
  }

  async function handleStop() {
    error = '';
    try {
      await stopServer();
    } catch (e: any) {
      error = typeof e === 'string' ? e : String(e);
    }
  }

  async function toggleLogs() {
    showLogs = !showLogs;
    if (showLogs) {
      try {
        logs = await getServerLogs();
        requestAnimationFrame(() => {
          if (logContainer) logContainer.scrollTop = logContainer.scrollHeight;
        });
      } catch (e: any) {
        error = e;
      }
    }
  }

  function formatDate(iso: string): string {
    const d = new Date(iso);
    return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
  }

  function cancelEdit() {
    view = 'list';
  }

  /** Show what the final command will look like at runtime */
  function previewCommand(cmd: string, modelPath?: string, mmprojPath?: string): string {
    const prog = settings.llama_server_path || 'llama-server';
    // Strip leading "llama-server" if present
    let args = cmd;
    if (args.startsWith('llama-server ')) args = args.slice('llama-server '.length);
    let preview = `${prog} ${args} --host ${settings.host} --port ${settings.port}`;
    if (modelPath) {
      preview += ` -m ${modelPath}`;
    }
    if (mmprojPath) {
      preview += ` --mmproj ${mmprojPath}`;
    }
    return preview;
  }

  async function checkLlamaPath() {
    if (!settings.llama_server_path.trim()) {
      llamaPathStatus = 'idle';
      llamaPathError = '';
      return;
    }
    llamaPathStatus = 'checking';
    llamaPathError = '';
    try {
      await getLlamaServerInfo();
      llamaPathStatus = 'ok';
    } catch (e: any) {
      llamaPathStatus = 'error';
      llamaPathError = typeof e === 'string' ? e : String(e);
    }
  }

  function onLlamaPathInput() {
    // Reset status while user is typing — actual check happens after Save
    llamaPathStatus = 'idle';
    llamaPathError = '';
  }

  async function pickModelFile() {
    const result = await open({
      multiple: false,
      filters: [{ name: 'GGUF Models', extensions: ['gguf'] }],
      defaultPath: settings.model_dir || undefined,
    });
    if (result) {
      formModelPath = result as string;
    }
  }

  async function pickMmprojFile() {
    const result = await open({
      multiple: false,
      filters: [{ name: 'GGUF Models', extensions: ['gguf'] }],
      defaultPath: settings.model_dir || undefined,
    });
    if (result) {
      formMmprojPath = result as string;
    }
  }
</script>

<div class="app">
  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <h1 class="app-title">Recipes</h1>
      <div class="sidebar-actions">
        <button class="btn-icon" onclick={openSettings} title="Settings">
          <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/></svg>
        </button>
        <button class="btn-icon" onclick={startNew} title="New Recipe">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        </button>
      </div>
    </div>

    <div class="search-box">
      <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
      <input type="text" placeholder="Search recipes..." bind:value={searchQuery} class="search-input" />
    </div>

    <!-- Global server status bar -->
    {#if anyServerRunning()}
      <div class="server-bar">
        <span class="status-dot running"></span>
        <span class="server-bar-text">
          Server running
          {#if runningRecipeId()}
            {#each recipes.filter(r => r.id === runningRecipeId()) as r}
              &mdash; {r.name}
            {/each}
          {/if}
        </span>
        <button class="server-bar-stop" onclick={handleStop}>Stop</button>
      </div>
    {/if}

    <div class="recipe-list">
      {#each filteredRecipes as recipe (recipe.id)}
        <button
          class="recipe-item"
          class:active={selectedId === recipe.id && view === 'list'}
          onclick={() => selectRecipe(recipe.id)}
        >
          <div class="recipe-item-header">
            <span class="recipe-item-name">{recipe.name}</span>
            {#if isRunning(recipe.id)}
              <span class="status-dot running" title="Running"></span>
            {/if}
          </div>
          {#if recipe.gpu_info}
            <span class="recipe-item-meta">{recipe.gpu_info}</span>
          {:else if recipe.tags}
            <span class="recipe-item-meta">{recipe.tags}</span>
          {/if}
        </button>
      {/each}

      {#if filteredRecipes.length === 0}
        <div class="empty-state-small">
          {searchQuery ? 'No matching recipes' : 'No recipes yet'}
        </div>
      {/if}
    </div>
  </aside>

  <!-- Main Content -->
  <main class="content">
    {#if error}
      <div class="error-bar">
        <span>{error}</span>
        <button onclick={() => error = ''}>Dismiss</button>
      </div>
    {/if}

    {#if view === 'settings'}
      <!-- Settings View -->
      <div class="form-view">
        <div class="form-header">
          <h2>Settings</h2>
          <div class="form-actions">
            <button class="btn secondary" onclick={() => { view = 'list'; }}>Back</button>
            <button class="btn primary" onclick={saveSettings}>
              {settingsSaved ? 'Saved' : 'Save'}
            </button>
          </div>
        </div>

        <div class="form-body">
          <p class="settings-desc">These settings control how llama-server is launched. Host and port are always injected into the command at runtime.</p>

          <div class="form-group">
            <label for="s-llama-path">llama-server Path</label>
            <div class="input-with-status">
              <input id="s-llama-path" type="text" bind:value={settingsLlamaPath} placeholder="llama-server" oninput={onLlamaPathInput} class:input-error={llamaPathStatus === 'error'} class:input-ok={llamaPathStatus === 'ok'} />
              {#if llamaPathStatus === 'checking'}
                <span class="path-status checking">Checking...</span>
              {:else if llamaPathStatus === 'ok'}
                <span class="path-status ok">Reachable</span>
              {:else if llamaPathStatus === 'error'}
                <span class="path-status error">Not found</span>
              {/if}
            </div>
            {#if llamaPathStatus === 'error' && llamaPathError}
              <span class="form-hint" style="color: var(--danger)">{llamaPathError}</span>
            {:else}
              <span class="form-hint">Absolute path or just "llama-server" if it's in your PATH</span>
            {/if}
          </div>

          <div class="form-row">
            <div class="form-group flex-1">
              <label for="s-host">Host</label>
              <input id="s-host" type="text" bind:value={settingsHost} placeholder="127.0.0.1" />
            </div>
            <div class="form-group" style="width: 140px">
              <label for="s-port">Port</label>
              <input id="s-port" type="number" bind:value={settingsPort} min="1" max="65535" />
            </div>
          </div>

          <div class="form-group">
            <label for="s-model-dir">Default Model Directory</label>
            <input id="s-model-dir" type="text" bind:value={settingsModelDir} placeholder="~/Models" />
            <span class="form-hint">Relative model paths in recipes will be resolved against this directory</span>
          </div>

          {#if anyServerRunning()}
            <div class="settings-warning">
              Changes will apply to the next server launch. A server is currently running.
            </div>
          {/if}
        </div>
      </div>
    {:else if view === 'new' || view === 'edit'}
      <!-- Recipe Form -->
      <div class="form-view">
        <div class="form-header">
          <h2>{view === 'new' ? 'New Recipe' : 'Edit Recipe'}</h2>
          <div class="form-actions">
            <button class="btn secondary" onclick={cancelEdit}>Cancel</button>
            <button class="btn primary" onclick={saveRecipe}>Save</button>
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
            <input id="name" type="text" bind:value={formName} placeholder="e.g. Qwen3 35B Q4" />
          </div>

          <div class="form-group">
            <label for="description">Description</label>
            <textarea id="description" rows="2" bind:value={formDescription} placeholder="What is this recipe for?"></textarea>
          </div>

          <div class="form-group">
            <label for="model_path">Model File</label>
            <div class="input-with-button">
              <input id="model_path" type="text" bind:value={formModelPath} placeholder="No model selected" class="mono" readonly />
              <button class="btn secondary" onclick={pickModelFile}>Browse...</button>
            </div>
            <span class="form-hint">Select a .gguf model file. Injected as -m at runtime.</span>
          </div>

          <div class="form-group">
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={formVision} />
              <span>Vision (mmproj)</span>
            </label>
            {#if formVision}
              <div class="input-with-button" style="margin-top: 6px">
                <input type="text" bind:value={formMmprojPath} placeholder="No mmproj file selected" class="mono" readonly />
                <button class="btn secondary" onclick={pickMmprojFile}>Browse...</button>
              </div>
              <span class="form-hint">Select the mmproj .gguf file for vision. Injected as --mmproj at runtime.</span>
            {/if}
          </div>

          <div class="form-group">
            <label for="command">Recipe Arguments</label>
            <textarea id="command" rows="4" bind:value={formCommand} placeholder="-ngl 99 -c 8192 -fa on --temp 0.6" class="mono"></textarea>
            <span class="form-hint">
              llama-server flags only. Do not include --host, --port, -m, or --mmproj &mdash; those are managed by the app.
              You can include "llama-server" at the start but it will be replaced by the configured path.
            </span>
          </div>

          {#if formCommand.trim()}
            <div class="command-preview">
              <h4>Command Preview</h4>
              <pre class="command-block">{previewCommand(formCommand, formModelPath, formVision ? formMmprojPath : '')}</pre>
            </div>
          {/if}

          <div class="form-row">
            <div class="form-group flex-1">
              <label for="gpu_info">GPU / Hardware</label>
              <input id="gpu_info" type="text" bind:value={formGpuInfo} placeholder="e.g. RTX 4090 24GB, M4 Max 128GB" />
            </div>
            <div class="form-group flex-1">
              <label for="tags">Tags</label>
              <input id="tags" type="text" bind:value={formTags} placeholder="e.g. qwen, 35b, q4" />
              <span class="form-hint">Comma separated</span>
            </div>
          </div>
        </div>
      </div>
    {:else if selected}
      <!-- Recipe Detail -->
      <div class="detail-view">
        <div class="detail-header">
          <div>
            <h2 class="detail-title">{selected.name}</h2>
            {#if selected.description}
              <p class="detail-desc">{selected.description}</p>
            {/if}
          </div>
          <div class="detail-actions">
            {#if isRunning(selected.id)}
              <button class="btn danger" onclick={handleStop}>Stop Server</button>
            {:else}
              <button class="btn primary" onclick={handleStart} disabled={anyServerRunning()}>
                {anyServerRunning() ? 'Another server running' : 'Start Server'}
              </button>
            {/if}
            <button class="btn secondary" onclick={startEdit}>Edit</button>
            <button class="btn-icon" onclick={() => handleDuplicate(selected.id)} title="Duplicate">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/></svg>
            </button>
            <button class="btn-icon danger-icon" onclick={() => handleDelete(selected.id)} title="Delete">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="3,6 5,6 21,6"/><path d="M19,6v14a2,2,0,0,1-2,2H7a2,2,0,0,1-2-2V6m3,0V4a2,2,0,0,1,2-2h4a2,2,0,0,1,2,2v2"/></svg>
            </button>
          </div>
        </div>

        <div class="detail-meta">
          {#if isRunning(selected.id)}
            <span class="badge running">Running{runningStatus?.pid ? ` (PID ${runningStatus.pid})` : ''}</span>
          {:else}
            <span class="badge stopped">Stopped</span>
          {/if}
          <span class="badge neutral">{settings.host}:{settings.port}</span>
          {#if selected.gpu_info}
            <span class="badge neutral">{selected.gpu_info}</span>
          {/if}
          {#each selected.tags.split(',').filter((t: string) => t.trim()) as tag}
            <span class="badge tag">{tag.trim()}</span>
          {/each}
        </div>

        <div class="detail-section">
          <h3>Model</h3>
          <pre class="command-block">{selected.model_path || '(not set)'}</pre>
        </div>

        {#if selected.mmproj_path}
          <div class="detail-section">
            <h3>Vision (mmproj)</h3>
            <pre class="command-block">{selected.mmproj_path}</pre>
          </div>
        {/if}

        <div class="detail-section">
          <h3>Recipe Arguments</h3>
          <pre class="command-block">{selected.command}</pre>
        </div>

        <div class="detail-section">
          <h3>Full Command (at runtime)</h3>
          <pre class="command-block preview">{previewCommand(selected.command, selected.model_path, selected.mmproj_path)}</pre>
        </div>

        <div class="detail-section">
          <span class="detail-date">Created {formatDate(selected.created_at)} &middot; Updated {formatDate(selected.updated_at)}</span>
        </div>

        <!-- Logs Section -->
        <div class="logs-section">
          <button class="btn secondary" onclick={toggleLogs}>
            {showLogs ? 'Hide Logs' : 'Show Logs'}
          </button>

          {#if showLogs}
            <div class="log-viewer" bind:this={logContainer}>
              {#if logs.length === 0}
                <div class="log-empty">No logs yet. Start the server to see output.</div>
              {:else}
                {#each logs as log}
                  <div class="log-line" class:stderr={log.is_stderr}>{log.line}</div>
                {/each}
              {/if}
            </div>
          {/if}
        </div>
      </div>
    {:else}
      <!-- Empty State -->
      <div class="empty-state">
        <div class="empty-icon">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14,2 14,8 20,8"/><line x1="12" y1="18" x2="12" y2="12"/><line x1="9" y1="15" x2="15" y2="15"/>
          </svg>
        </div>
        <h2>No recipe selected</h2>
        <p>Select a recipe from the sidebar or create a new one to get started.</p>
        <button class="btn primary" onclick={startNew}>Create Recipe</button>
      </div>
    {/if}
  </main>
</div>

<style>
  .app {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  /* Sidebar */
  .sidebar {
    width: 280px;
    min-width: 280px;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 16px 8px;
    -webkit-app-region: drag;
  }

  .sidebar-header .sidebar-actions {
    display: flex;
    gap: 2px;
    -webkit-app-region: no-drag;
  }

  .app-title {
    font-size: 18px;
    font-weight: 700;
    letter-spacing: -0.3px;
  }

  .search-box {
    position: relative;
    padding: 4px 16px 12px;
  }

  .search-icon {
    position: absolute;
    left: 26px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-tertiary);
    margin-top: -4px;
  }

  .search-input {
    width: 100%;
    padding: 6px 10px 6px 30px;
    background: var(--bg-tertiary);
    border: none;
    border-radius: var(--radius-sm);
    font-size: 13px;
  }

  /* Server status bar in sidebar */
  .server-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: rgba(52, 199, 89, 0.08);
    border-top: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
  }

  .server-bar-text {
    flex: 1;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .server-bar-stop {
    font-size: 12px;
    font-weight: 600;
    color: var(--danger);
    flex-shrink: 0;
  }

  .server-bar-stop:hover {
    text-decoration: underline;
  }

  .recipe-list {
    flex: 1;
    overflow-y: auto;
    padding: 0 8px 8px;
  }

  .recipe-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    margin-bottom: 2px;
    transition: background 0.1s;
  }

  .recipe-item:hover {
    background: var(--bg-tertiary);
  }

  .recipe-item.active {
    background: var(--accent);
    color: white;
  }

  .recipe-item.active .recipe-item-meta {
    color: rgba(255,255,255,0.7);
  }

  .recipe-item-header {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .recipe-item-name {
    font-weight: 500;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .recipe-item-meta {
    font-size: 11px;
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .status-dot.running {
    background: var(--success);
    box-shadow: 0 0 4px var(--success);
  }

  .empty-state-small {
    text-align: center;
    padding: 24px 16px;
    color: var(--text-tertiary);
    font-size: 13px;
  }

  /* Content */
  .content {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  /* Error */
  .error-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 20px;
    background: var(--danger);
    color: white;
    font-size: 13px;
  }

  .error-bar button {
    color: white;
    text-decoration: underline;
    font-size: 13px;
  }

  /* Empty State */
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--text-secondary);
  }

  .empty-icon {
    color: var(--text-tertiary);
    opacity: 0.5;
    margin-bottom: 8px;
  }

  .empty-state h2 {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .empty-state p {
    font-size: 14px;
    max-width: 300px;
    text-align: center;
  }

  /* Detail View */
  .detail-view {
    padding: 24px 32px;
    max-width: 800px;
  }

  .detail-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 16px;
  }

  .detail-title {
    font-size: 22px;
    font-weight: 700;
    letter-spacing: -0.3px;
  }

  .detail-desc {
    color: var(--text-secondary);
    font-size: 14px;
    margin-top: 4px;
  }

  .detail-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .detail-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 24px;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    font-size: 12px;
    font-weight: 500;
    padding: 3px 10px;
    border-radius: 100px;
  }

  .badge.running {
    background: rgba(52, 199, 89, 0.15);
    color: var(--success);
  }

  .badge.stopped {
    background: var(--bg-tertiary);
    color: var(--text-tertiary);
  }

  .badge.neutral {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
  }

  .badge.tag {
    background: rgba(0, 113, 227, 0.1);
    color: var(--accent);
  }

  .detail-section {
    margin-bottom: 20px;
  }

  .detail-section h3 {
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

  .command-block.preview {
    border-style: dashed;
    opacity: 0.85;
  }

  .detail-date {
    font-size: 12px;
    color: var(--text-tertiary);
  }

  /* Logs */
  .logs-section {
    margin-top: 8px;
  }

  .log-viewer {
    margin-top: 12px;
    background: #1a1a1a;
    border-radius: var(--radius-md);
    padding: 12px;
    height: 320px;
    overflow-y: auto;
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.5;
  }

  .log-line {
    color: #d4d4d4;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .log-line.stderr {
    color: #f87171;
  }

  .log-empty {
    color: #666;
    text-align: center;
    padding: 48px 0;
  }

  /* Form View (shared for recipe form + settings) */
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

  .form-group input, .form-group textarea {
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

  .command-preview {
    padding: 0;
  }

  .command-preview h4 {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-tertiary);
    margin-bottom: 8px;
  }

  .settings-desc {
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .settings-warning {
    font-size: 13px;
    color: var(--warning);
    padding: 10px 14px;
    background: rgba(255, 159, 10, 0.08);
    border: 1px solid rgba(255, 159, 10, 0.3);
    border-radius: var(--radius-sm);
  }

  /* Buttons */
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

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn.primary {
    background: var(--accent);
    color: white;
  }

  .btn.primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .btn.secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .btn.secondary:hover {
    background: var(--border);
  }

  .btn.danger {
    background: var(--danger);
    color: white;
  }

  .btn.danger:hover {
    background: var(--danger-hover);
  }

  .btn-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    transition: all 0.15s;
  }

  .btn-icon:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .btn-icon.danger-icon:hover {
    background: rgba(255, 59, 48, 0.1);
    color: var(--danger);
  }

  /* Path validation status */
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

  .input-with-button {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .input-with-button input {
    flex: 1;
  }

  .checkbox-label {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
  }

  .checkbox-label input[type="checkbox"] {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
  }
</style>
