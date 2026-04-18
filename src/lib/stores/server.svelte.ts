import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import { clearServerLogs, getServerLogs, getServerStatus, startServer, stopServer } from '$lib/api';
import { settingsStore } from '$lib/stores/settings.svelte';
import type { LogLine, ServerExit, ServerStatus } from '$lib/types';
import { fetchHealthOk } from '$lib/utils/llamaClient';

/**
 * Lifecycle stages a server can be in:
 *   - `idle`     no process running
 *   - `starting` process spawned, but `/health` has not yet responded OK
 *   - `ready`    `/health` is OK; the API is actually accepting traffic
 *
 * The Tauri backend can only observe the OS process, so it always reports
 * `running:true` the moment it spawns the child. We treat that signal as
 * `starting` and only promote to `ready` after the HTTP health check passes.
 */
export type ServerPhase = 'idle' | 'starting' | 'ready' | 'stopping';

/** How often to retry `/health` while the server is `starting`. */
const HEALTH_POLL_MS = 750;

class ServerStore {
  status = $state<ServerStatus | null>(null);
  phase = $state<ServerPhase>('idle');
  logs = $state<LogLine[]>([]);
  /**
   * Wall-clock time at which the server became `ready`. We don't start the
   * uptime clock until then — "starting" is not really uptime.
   */
  startedAt = $state<number | null>(null);
  /**
   * The most recent `server-exit` event, kept around so the UI can show a
   * crash banner after the process is gone. Cleared on the next `start()`
   * or by `dismissExit()`.
   */
  lastExit = $state<ServerExit | null>(null);

  private unlistenStatus: UnlistenFn | null = null;
  private unlistenLog: UnlistenFn | null = null;
  private unlistenExit: UnlistenFn | null = null;
  private healthAbort: AbortController | null = null;

  /** True once `/health` has acknowledged this recipe's server. */
  isReady(recipeId: string): boolean {
    return this.phase === 'ready' && this.status?.recipe_id === recipeId;
  }

  /** True if the process is up but not yet serving requests. */
  isStarting(recipeId: string): boolean {
    return this.phase === 'starting' && this.status?.recipe_id === recipeId;
  }

  /** True between the user clicking Stop and the process actually exiting. */
  isStopping(recipeId: string): boolean {
    return this.phase === 'stopping' && this.status?.recipe_id === recipeId;
  }

  /** True if the process exists for this recipe in any phase. */
  isActive(recipeId: string): boolean {
    return this.status?.recipe_id === recipeId && this.status?.running === true;
  }

  /**
   * @deprecated prefer `isReady` (UI semantics) or `isActive` (process-existence semantics).
   * Kept for callers that want "is the server up at all" — same as `isActive`.
   */
  isRunning(recipeId: string): boolean {
    return this.isActive(recipeId);
  }

  /** True if any recipe currently owns the single server slot. */
  anyActive(): boolean {
    return this.status?.running === true;
  }

  anyRunning(): boolean {
    return this.anyActive();
  }

  runningRecipeId(): string | null {
    return this.status?.running ? this.status.recipe_id : null;
  }

  async refresh(): Promise<void> {
    const s = await getServerStatus();
    this.status = s;
    if (s?.running) {
      // The process is up but we don't know if /health has answered yet.
      // Re-enter the `starting` -> `ready` handshake.
      if (this.phase === 'idle') this.phase = 'starting';
      this.kickHealthLoop();
    } else {
      this.phase = 'idle';
      this.startedAt = null;
      this.cancelHealthLoop();
    }
    this.logs = await getServerLogs();
  }

  async start(
    recipeId: string,
    command: string,
    modelPath: string,
    mmprojPath: string,
  ): Promise<void> {
    this.logs = [];
    this.lastExit = null;
    // Optimistic: switch to `starting` immediately so the UI can show a
    // spinner while the backend spawns and we wait for /health.
    this.phase = 'starting';
    this.startedAt = null;
    await startServer(recipeId, command, modelPath, mmprojPath);
  }

  dismissExit(): void {
    this.lastExit = null;
  }

  async stop(): Promise<void> {
    // Mark stopping immediately so the UI can show progress while the
    // backend SIGTERMs and waits for the graceful drain.
    if (this.status?.running) this.phase = 'stopping';
    this.cancelHealthLoop();
    await stopServer();
  }

  async clearLogs(): Promise<void> {
    await clearServerLogs();
    this.logs = [];
  }

  /** Reset client-side log buffer only (used when switching recipes). */
  resetLogView(): void {
    this.logs = [];
  }

  async subscribe(): Promise<void> {
    this.unlistenStatus = await listen<ServerStatus>('server-status', (event) => {
      const s = event.payload;
      if (s.running) {
        this.status = s;
        // Backend only knows that the process is alive. Stay in `starting`
        // until /health confirms the API is up.
        this.phase = 'starting';
        this.startedAt = null;
        this.kickHealthLoop();
      } else if (this.status?.recipe_id === s.recipe_id) {
        this.status = null;
        this.phase = 'idle';
        this.startedAt = null;
        this.cancelHealthLoop();
      }
    });

    this.unlistenLog = await listen<LogLine>('server-log', (event) => {
      this.logs = [...this.logs, event.payload];
      if (this.logs.length > 2000) {
        this.logs = this.logs.slice(-2000);
      }
    });

    this.unlistenExit = await listen<ServerExit>('server-exit', (event) => {
      this.lastExit = event.payload;
    });
  }

  unsubscribe(): void {
    this.unlistenStatus?.();
    this.unlistenLog?.();
    this.unlistenExit?.();
    this.unlistenStatus = null;
    this.unlistenLog = null;
    this.unlistenExit = null;
    this.cancelHealthLoop();
  }

  /**
   * Poll `/health` until the API responds OK or the process disappears.
   * Idempotent — multiple `kickHealthLoop` calls collapse onto one in-flight
   * loop because we abort any previous controller.
   */
  private kickHealthLoop(): void {
    this.cancelHealthLoop();
    const ac = new AbortController();
    this.healthAbort = ac;

    const loop = async () => {
      while (
        !ac.signal.aborted &&
        this.status?.running &&
        (this.phase as ServerPhase) !== 'stopping'
      ) {
        const ok = await fetchHealthOk(settingsStore.current, ac.signal);
        if (ac.signal.aborted || (this.phase as ServerPhase) === 'stopping') return;
        if (ok) {
          this.phase = 'ready';
          this.startedAt = Date.now();
          return;
        }
        await new Promise<void>((resolve) => {
          const t = setTimeout(resolve, HEALTH_POLL_MS);
          ac.signal.addEventListener('abort', () => {
            clearTimeout(t);
            resolve();
          });
        });
      }
    };
    void loop();
  }

  private cancelHealthLoop(): void {
    this.healthAbort?.abort();
    this.healthAbort = null;
  }
}

export const serverStore = new ServerStore();
