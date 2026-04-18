/**
 * Mirror of the Rust flag deny-lists. Kept in sync so users get immediate
 * inline feedback in the recipe form before the backend rejects.
 *
 * See `src-tauri/src/validate.rs` for the canonical lists and rationale.
 *
 * - `FORBIDDEN_FLAGS` — always blocked (conflict with app-managed settings).
 * - `UNSAFE_FLAGS`    — allowed for local recipes today; will be blocked for
 *                       community-supplied recipes once that feature ships.
 */
export const FORBIDDEN_FLAGS = [
  // Network binding
  '--port',
  '-p',
  '--host',
  // Model / mmproj
  '-m',
  '--model',
  '--model-path',
  '--mmproj',
  '-mm',
  '--mmproj-url',
  '-mmu',
  // Logging
  '--log-file',
  // Auth / TLS
  '--api-key',
  '--api-key-file',
  '--ssl-cert-file',
  '--ssl-key-file',
  '--hf-token',
  '-hft',
  // Server endpoints / behaviour
  '--api-prefix',
  '--timeout',
  '-to',
  '--log-verbosity',
  '-lv',
  '--webui',
  '--no-webui',
  '--metrics',
  '--slots',
  '--no-slots',
] as const;

export const UNSAFE_FLAGS = [
  // Shell / MCP proxy
  '--tools',
  '--webui-mcp-proxy',
  '--no-webui-mcp-proxy',
  // Static directory served as files
  '--path',
  // Alternate model loaders
  '-hf',
  '--hf-repo',
  '-hff',
  '--hf-file',
  '-hfd',
  '-hfrd',
  '--hf-repo-draft',
  '-hfv',
  '-hfrv',
  '--hf-repo-v',
  '-hffv',
  '--hf-file-v',
  '-mu',
  '--model-url',
  '-md',
  '--model-draft',
  // LoRA adapters
  '--lora',
  '--lora-scaled',
  '--lora-base',
  // Filesystem read/write surface
  '--slot-save-path',
  '--media-path',
  '--models-dir',
  '--models-preset',
  '--models-max',
  '--models-autoload',
  '--no-models-autoload',
  // Web UI / template / grammar files
  '--webui-config',
  '--webui-config-file',
  '--grammar-file',
  '--json-schema-file',
  '-jf',
  '--chat-template-file',
] as const;

export type RecipeSource = 'local' | 'community';

function checkAgainst(cmd: string, lists: readonly (readonly string[])[]): string[] {
  const errors: string[] = [];
  if (cmd.includes('\0')) {
    errors.push('Command contains a NUL byte.');
    return errors;
  }
  for (const token of cmd.split(/\s+/)) {
    if (!token) continue;
    const lower = token.toLowerCase();
    for (const list of lists) {
      for (const flag of list) {
        if (lower === flag || lower.startsWith(flag + '=')) {
          errors.push(
            `Command must not contain '${flag}' — this flag is either managed by the app or has been blocked for safety reasons.`,
          );
        }
      }
    }
  }
  return errors;
}

/**
 * Validate a recipe command. Defaults to the local-recipe policy
 * (`FORBIDDEN_FLAGS` only). Pass `'community'` to also block `UNSAFE_FLAGS`.
 */
export function validateCommand(cmd: string, source: RecipeSource = 'local'): string[] {
  const lists = source === 'community' ? [FORBIDDEN_FLAGS, UNSAFE_FLAGS] : [FORBIDDEN_FLAGS];
  return checkAgainst(cmd, lists);
}
