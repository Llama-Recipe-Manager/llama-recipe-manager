export interface Recipe {
  id: string;
  name: string;
  description: string;
  command: string;
  model_path: string;
  mmproj_path: string;
  gpu_info: string;
  tags: string;
  created_at: string;
  updated_at: string;
}

export interface CreateRecipe {
  name: string;
  description: string;
  command: string;
  model_path: string;
  mmproj_path: string;
  gpu_info: string;
  tags: string;
}

export interface UpdateRecipe extends CreateRecipe {
  id: string;
}

export interface Settings {
  host: string;
  port: number;
  model_dir: string;
  llama_server_path: string;
  // Security
  api_key: string;
  ssl_cert_file: string;
  ssl_key_file: string;
  hf_token: string;
  // Server behavior
  webui_enabled: boolean;
  metrics_enabled: boolean;
  slots_enabled: boolean;
  api_prefix: string;
  timeout_secs: number;
  // Diagnostics
  log_verbosity: number;
  // Lifecycle
  keep_server_on_exit: boolean;
}

export interface ServerStatus {
  recipe_id: string;
  running: boolean;
  pid: number | null;
}

export interface LogLine {
  recipe_id: string;
  line: string;
  is_stderr: boolean;
}

export interface ServerExit {
  recipe_id: string;
  /** Exit code, when the process exited normally. */
  code: number | null;
  /** Unix signal number (only set on Unix when the process was signalled). */
  signal: number | null;
  /** True if the user (or app shutdown) initiated the stop. */
  intentional: boolean;
}

export interface GpuDevice {
  name: string;
  vram_mib: number;
  compute_capability: string;
}

export interface LlamaServerInfo {
  version: string;
  compiler: string;
  gpu_devices: GpuDevice[];
  raw_output: string;
}

export type View = 'list' | 'edit' | 'new' | 'settings';

export type LlamaPathStatus = 'idle' | 'checking' | 'ok' | 'error';

/**
 * Top-level navigation section. Drives which sub-sidebar (if any) and which
 * main view is rendered. `community` is currently a placeholder; see
 * `docs/PLAN.md`.
 */
export type Section = 'recipes' | 'community' | 'settings';

/**
 * llama.cpp build / accelerator backend. Used to filter community recipes and
 * to label local recipes with "what build I tested this against".
 *
 * Currently only the type exists; persistence lands with phase 1 of the
 * community-recipes plan.
 */
export type Backend = 'auto' | 'cuda' | 'vulkan' | 'rocm' | 'metal' | 'cpu';

/**
 * Capability tag derived from a recipe's flags / mmproj presence. Used as a
 * filter facet on the community browse view.
 */
export type Capability = 'chat' | 'vision' | 'embedding';

/**
 * Filters for the community browse view (placeholder today).
 */
export interface CommunityFilters {
  search: string;
  backend: Backend | 'any';
  capability: Capability | 'any';
  vramMinGib: number | null;
  sort: 'recent' | 'forks' | 'validated';
}
