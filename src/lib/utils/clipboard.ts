/**
 * Copy helpers with a graceful fallback for older WebViews.
 *
 * The modern async `navigator.clipboard` API is only available in secure
 * contexts and newer WebKit/WebView implementations. When it is missing — or
 * rejects (e.g. the clipboard is in use by another document) — this falls back
 * to the classic hidden-textarea + `document.execCommand('copy')` technique
 * which works broadly across the WebViews that embed this app.
 */

type ClipboardGlobal = {
  navigator?: { clipboard?: { writeText: (text: string) => Promise<void> } };
  document?: Document;
  execCommand?: (command: string) => boolean;
};

function globals(): ClipboardGlobal {
  return globalThis as unknown as ClipboardGlobal;
}

/**
 * Copy `text` to the clipboard. Resolves `true` on success, `false` if every
 * strategy fails. Never throws — callers surface a "couldn't copy" message
 * based on the return value.
 */
export async function copyText(text: string): Promise<boolean> {
  const nav = globals().navigator;
  if (nav?.clipboard?.writeText) {
    try {
      await nav.clipboard.writeText(text);
      return true;
    } catch {
      // Fall through to the legacy path below.
    }
  }
  return legacyCopy(text);
}

function legacyCopy(text: string): boolean {
  const doc = globals().document;
  if (!doc) return false;

  const textarea = doc.createElement('textarea');
  textarea.value = text;
  // Keep it out of the layout and out of the visual viewport so pasting works
  // without causing a visible caret jump.
  textarea.setAttribute('readonly', '');
  textarea.style.position = 'fixed';
  textarea.style.top = '-9999px';
  textarea.style.opacity = '0';

  (doc.body ?? doc.documentElement).appendChild(textarea);
  textarea.select();
  textarea.setSelectionRange(0, textarea.value.length);

  let ok: boolean;
  try {
    ok = doc.execCommand('copy');
  } catch {
    ok = false;
  }

  textarea.remove();
  return ok;
}
