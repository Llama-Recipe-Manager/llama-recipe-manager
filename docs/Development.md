# Development

This document covers everything beyond a basic `bun run tauri dev`:
build prerequisites, the day-to-day commands, the repository layout,
how the app icon is regenerated, and how releases / auto-updates are
signed and published.

For project rationale and a quick install, see the [README](../README.md).
For contribution mechanics (branching, commits, PR review) see
[CONTRIBUTING.md](../CONTRIBUTING.md).

## Build from source

You'll need:

- [Rust](https://rustup.rs) stable, **1.78+**
- [Bun](https://bun.sh) **1.1+**
- On Linux, the [Tauri Linux prerequisites](https://tauri.app/start/prerequisites/#linux)

```bash
git clone https://github.com/coder3101/llama-recipe-manager.git
cd llama-recipe-manager
bun install
bun run tauri dev      # development with HMR
bun run tauri build    # produces an .app / .deb / .AppImage / .msi
```

The release artefact lands under
`src-tauri/target/release/bundle/<platform>/`.

## Day-to-day commands

```bash
bun run tauri dev          # native dev server with HMR
bun run check              # svelte-check
bun run lint               # eslint
bun run format             # prettier write
bun run test               # vitest (frontend)
bun run lint:rust          # cargo clippy -D warnings
bun run format:rust        # rustfmt
bun run test:rust          # cargo test (unit + integration)
bun run icons              # regenerate platform icons from src-tauri/icons/icon.png
```

CI runs every one of those on Linux, macOS, and Windows for every PR.

## Repository layout

```
src/                       SvelteKit frontend
  routes/                    pages and root layout
  lib/components/            Svelte components
  lib/stores/                global runes-based stores
  lib/api/                   typed wrappers around Tauri commands
src-tauri/                 Rust backend
  src/commands.rs            #[tauri::command] entry points
  src/db.rs                  rusqlite + migrations
  src/process.rs             llama-server lifecycle (spawn, signal, exit)
  src/validate.rs            recipe / settings validation + flag deny-list
  src/lib.rs                 plugin registration + ExitRequested wiring
  capabilities/              Tauri ACL (which APIs the webview can call)
  tauri.conf.json            bundle, updater, plugins, window
scripts/                   maintenance scripts (icons, …)
tests/                     frontend Vitest tests
docs/                      this folder — long-form documentation
```

The `src-tauri/src/migrations/` directory holds versioned `.sql` files;
the bootstrap in `db.rs` applies them in order using
`PRAGMA user_version`. To add one, see
[CONTRIBUTING.md → "Adding a database migration"](../CONTRIBUTING.md#adding-a-database-migration).

## App icon

The canonical source is `src-tauri/icons/icon.png` (square, ≥ 1024×1024).
Replace it and run `bun run icons` to regenerate every platform-native
asset. The script (`scripts/make-icons.py`) deliberately treats each
platform differently because their conventions disagree:

| Platform | Output | Treatment |
| --- | --- | --- |
| macOS | `icon.icns` | Squircle mask (superellipse, n=5) + 824/1024 safe area, per Apple HIG. |
| Windows | `icon.ico` | Multi-resolution 16/24/32/48/64/128/256, edge-to-edge, PNG-compressed entries. |
| Linux | `16x16…512x512.png` | Square PNGs at freedesktop hicolor sizes, edge-to-edge transparent. |

Why the split: macOS owns the icon silhouette (so we pre-mask), while
Windows and Linux expect raw square art and apply their own taskbar /
dock treatment — masking those would actively look wrong.

Requirements: Python 3 with `Pillow` and `numpy`, plus `iconutil`
(preinstalled on macOS) for the `.icns` step. Pass
`--platforms macos|windows|linux` to regenerate just one.

The user-supplied `icon.png` is never overwritten — re-runs are
idempotent and you won't lose source resolution.

## Releasing & auto-update

Tagged builds are published as GitHub Releases by
`.github/workflows/release.yml`. The in-app updater fetches signed
`latest.json` manifests from those releases.

To enable signed auto-updates from your own fork:

1. Generate a keypair locally — keep the private key offline:
   ```bash
   bun run tauri signer generate -- -w ~/.tauri/llama-recipe-manager.key
   ```
2. Copy the **public key** that's printed and paste it into
   `src-tauri/tauri.conf.json` under `plugins.updater.pubkey`.
3. Add two repository secrets in GitHub:
   - `TAURI_SIGNING_PRIVATE_KEY` — contents of the generated `.key` file.
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — the passphrase you chose.
4. Push a tag like `v0.1.1`. The release workflow will bundle, sign,
   and upload `latest.json` alongside the platform installers.

If the public key is empty or the secrets aren't set, the updater check
silently no-ops — the rest of the app keeps working.

## Where data lives

Llama Recipe Manager stores everything in your OS's standard app-data
directory:

| OS      | Path                                                                    |
| ------- | ----------------------------------------------------------------------- |
| macOS   | `~/Library/Application Support/com.llama-recipe-manager.app/recipes.db` |
| Linux   | `~/.local/share/com.llama-recipe-manager.app/recipes.db`                |
| Windows | `%APPDATA%\com.llama-recipe-manager.app\recipes.db`                     |

Schema migrations are version-tracked via SQLite's `PRAGMA user_version`
and applied automatically on launch.
