import type { Settings } from '$lib/types';

/** Render the final llama-server command for display. */
export function previewCommand(
  settings: Settings,
  recipeArgs: string,
  modelPath?: string,
  mmprojPath?: string,
): string {
  const prog = settings.llama_server_path || 'llama-server';
  let args = recipeArgs;
  if (args.startsWith('llama-server ')) args = args.slice('llama-server '.length);
  let preview = `${prog} ${args} --host ${settings.host} --port ${settings.port}`;
  if (modelPath) preview += ` -m ${modelPath}`;
  if (mmprojPath) preview += ` --mmproj ${mmprojPath}`;

  if (settings.api_key) preview += ` --api-key ${redact(settings.api_key)}`;
  if (settings.ssl_cert_file && settings.ssl_key_file) {
    preview += ` --ssl-cert-file ${settings.ssl_cert_file} --ssl-key-file ${settings.ssl_key_file}`;
  }
  if (!settings.webui_enabled) preview += ' --no-webui';
  if (settings.metrics_enabled) preview += ' --metrics';
  if (!settings.slots_enabled) preview += ' --no-slots';
  if (settings.api_prefix) preview += ` --api-prefix ${settings.api_prefix}`;
  if (settings.timeout_secs && settings.timeout_secs !== 600) {
    preview += ` --timeout ${settings.timeout_secs}`;
  }
  if (settings.log_verbosity !== 3) preview += ` --log-verbosity ${settings.log_verbosity}`;

  return preview;
}

function redact(secret: string): string {
  if (secret.length <= 4) return '***';
  return `${secret.slice(0, 2)}${'*'.repeat(Math.max(3, secret.length - 4))}${secret.slice(-2)}`;
}
