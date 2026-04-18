# Contributing to Llama Recipe Manager

Thanks for taking the time to contribute! This document is the short version
of how this project is developed and what to expect when you open a PR.

## Quick start

```bash
git clone https://github.com/<your-fork>/llama-recipe-manager.git
cd llama-recipe-manager
bun install              # frontend deps
bun run tauri dev        # native dev build (pulls Rust deps on first run)
```

You'll need:

- **Rust** stable (1.78+ recommended). Install via [rustup](https://rustup.rs).
- **Bun** 1.1+. Install via [bun.sh](https://bun.sh).
- Platform toolchain for Tauri:
  [macOS](https://tauri.app/start/prerequisites/#macos) /
  [Linux](https://tauri.app/start/prerequisites/#linux) /
  [Windows](https://tauri.app/start/prerequisites/#windows).

## Repository layout

```
src/                  SvelteKit frontend (TypeScript, Svelte 5 runes)
  lib/api/            Tauri command wrappers, grouped by domain
  lib/components/     UI components
  lib/stores/         Reactive state stores
  lib/utils/          Pure utilities (covered by Vitest)
  lib/featureFlags.ts Feature gating for in-progress work
src-tauri/            Rust backend
  src/commands/       Tauri command handlers
  src/db.rs           SQLite layer
  src/migrations/     Embedded SQL migration files
  src/process.rs      llama-server lifecycle
  src/validate.rs     Recipe / flag deny-lists
tests/unit/           Vitest unit tests (frontend)
src-tauri/tests/      Cargo integration tests (backend)
docs/                 Design docs and plans
```

## Quality gates

Before opening a PR, run:

```bash
bun run check           # svelte-check (types)
bun run lint            # eslint
bun run format:check    # prettier
bun run test            # vitest
bun run lint:rust       # cargo clippy -D warnings
bun run format:rust     # rustfmt
bun run test:rust       # cargo test
```

CI runs all of the above on Linux, macOS, and Windows. PRs are expected to
pass without warnings.

## Coding style

- **Rust** — `rustfmt` + `clippy` pedantic with the allow-list in
  `src-tauri/Cargo.toml`. Keep modules small and prefer returning `Result<_,
String>` from Tauri commands so the frontend can render the error verbatim.
- **TypeScript / Svelte** — Prettier + ESLint flat config. Svelte 5 runes
  (`$state`, `$derived`, `$effect`, `$props`) only — no legacy `$:` reactive
  statements.
- **Comments** explain _why_, not _what_. Avoid restating code in prose.

## Adding a database migration

Migrations are forward-only and version-tracked via SQLite's
`PRAGMA user_version`.

1. Create a new file in `src-tauri/src/migrations/` named
   `NNNN_description.sql`, where `NNNN` is the next integer version
   (zero-padded to 4 digits).
2. Add a `Migration` entry to the `MIGRATIONS` array in
   `src-tauri/src/db/migrations.rs` with the same `version` and an
   `include_str!` of the new file.
3. Update the integration tests in `src-tauri/tests/db_integration.rs` if the
   schema change affects existing fields.

Migrations apply on app launch and are idempotent.

## Security-sensitive changes

Changes to `src-tauri/src/validate.rs` (FORBIDDEN_FLAGS / UNSAFE_FLAGS) and
`src-tauri/src/process.rs` (path resolution) require:

- A test added to `src-tauri/src/validate.rs#tests` or
  `src-tauri/tests/db_integration.rs`.
- The mirror update in `src/lib/utils/validate.ts` (with a matching test in
  `tests/unit/validate.test.ts`).

If you discover a vulnerability, please follow [SECURITY.md](./SECURITY.md)
instead of opening a public issue.

## Commit style

Conventional Commits are encouraged but not enforced:

```
feat(settings): expose log verbosity dropdown
fix(process): canonicalise mmproj path before spawn
docs(readme): add windows build steps
```

Keep PRs focused. If you want to discuss a sweeping change, open an issue
first so we can scope it together.

## Code of conduct

This project follows the [Contributor Covenant](./CODE_OF_CONDUCT.md). By
participating you agree to abide by its terms.
