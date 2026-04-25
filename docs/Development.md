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
git clone https://github.com/Llama-Recipe-Manager/llama-recipe-manager.git
cd llama-recipe-manager
bun install
bun run tauri dev      # development with HMR
bun run tauri build    # produces an .app / .deb / .AppImage / .msi
```

The release artefact lands under
`src-tauri/target/release/bundle/<platform>/`.

## Day-to-day commands

Install [`just`](https://github.com/casey/just) for a single command to run everything:

```bash
just fmt       # prettier + cargo fmt
just lint      # eslint + cargo clippy
just check     # svelte-check + clippy + cargo fmt --check
just test      # vitest + cargo test
```

Individual commands:

```bash
bun run tauri dev          # native dev server with HMR
bun run icons              # regenerate platform icons from src-tauri/icons/icon.png
```

CI runs every `just` recipe on Linux, macOS, and Windows for every PR.

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

| Platform | Output              | Treatment                                                                      |
| -------- | ------------------- | ------------------------------------------------------------------------------ |
| macOS    | `icon.icns`         | Squircle mask (superellipse, n=5) + 824/1024 safe area, per Apple HIG.         |
| Windows  | `icon.ico`          | Multi-resolution 16/24/32/48/64/128/256, edge-to-edge, PNG-compressed entries. |
| Linux    | `16x16…512x512.png` | Square PNGs at freedesktop hicolor sizes, edge-to-edge transparent.            |

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
`.github/workflows/release.yml`, which produces signed bundles for
macOS (Apple Silicon), Linux (Ubuntu), and Windows. The in-app updater
fetches signed `latest.json` manifests from those releases.

### Versioning policy

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** — breaking schema, settings, or recipe-format change that an
  older binary cannot read.
- **MINOR** — new user-visible functionality, added settings, new
  bundled assets.
- **PATCH** — bug fixes, dependency bumps, doc-only changes, internal
  refactors that don't change behaviour.

The version string lives in three files that **must stay in lockstep** —
the release workflow takes the tag as the source of truth, but the
in-app "About" dialog reads the local files, so a mismatch shows up as
the wrong version under the user's nose.

| File                        | Field                            |
| --------------------------- | -------------------------------- |
| `package.json`              | `"version"`                      |
| `src-tauri/Cargo.toml`      | `version =`                      |
| `src-tauri/tauri.conf.json` | `"version"`                      |
| `src-tauri/Cargo.lock`      | regenerated by `cargo update -p` |

### Release checklist

Pre-flight (on a release branch, or on `main` if you're confident):

1. **Bump versions** — run the bump script which updates all three files,
   refreshes `Cargo.lock`, and runs the full quality gate in one shot:
   ```bash
   python scripts/bump-version.py --patch   # --minor or --major
   ```
   If the quality gate fails, fix the issues and re-run the same command.
2. **Update the changelog.** Move everything from `## [Unreleased]` into
   a new `## [X.Y.Z] — YYYY-MM-DD` section, grouped by `Added` /
   `Changed` / `Fixed` / `Removed` / `Security` (Keep-a-Changelog
   conventions). Leave a fresh empty `## [Unreleased]` block on top.
3. **Land the bump on `main`** via PR.
   Title: `release: vX.Y.Z`. Merge once green.

Cut the release:

5. **Tag the merge commit** and push the tag:
   ```bash
   git checkout main && git pull
   git tag -a vX.Y.Z -m "vX.Y.Z"
   git push origin vX.Y.Z
   ```
6. The `Release` workflow runs automatically. It produces a **draft**
   GitHub Release containing:
   - `Llama Recipe Manager_X.Y.Z_aarch64.dmg` (macOS, Apple Silicon)
   - `llama-recipe-manager_X.Y.Z_amd64.{deb,AppImage}` (Linux)
   - `Llama Recipe Manager_X.Y.Z_x64-setup.{exe,msi}` (Windows)
   - `latest.json` + per-bundle `.sig` files (consumed by the updater)
7. **Review the draft** on GitHub. Edit the auto-generated body if
   needed (a link to the changelog section is usually enough). Click
   **Publish release**.

Post-release:

8. **Verify the updater chain.** Install the previous release on any
   platform, then **Settings → About → Check for updates** — it should
   discover the new version, download, and prompt to relaunch. If
   nothing appears, double-check that the new release is _published_
   (not still in draft) and that `latest.json` is attached.
9. **Close the milestone** (if you used one) and move any remaining
   issues to the next.

### Hotfix flow (patch on top of an already-released version)

When `main` has work-in-progress that shouldn't ship in the hotfix,
branch from the tag instead of `main`:

```bash
git checkout -b hotfix/vX.Y.Z+1 vX.Y.Z
# fix, test, commit
# bump versions to X.Y.Z+1, update CHANGELOG
git push origin hotfix/vX.Y.Z+1
# open PR targeting main, merge, then tag the merge commit as above
```

If the fix is critical and `main` has diverged in ways that would
complicate cherry-picking back, it's fine to tag directly off the
hotfix branch — just remember to merge it back into `main` afterwards
so the fix isn't lost.

### Setting up signed updates (one-time, for a fork)

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

If the public key is empty or the secrets aren't set, the updater check
silently no-ops — the rest of the app keeps working, you just lose the
"download and install" prompt.

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
