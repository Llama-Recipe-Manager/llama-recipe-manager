# Release Guide

This document describes the steps to cut a new release of Llama Recipe Manager.

## Versioning

We follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html):

- **MAJOR** — breaking schema, settings, or recipe-format change.
- **MINOR** — new user-visible functionality.
- **PATCH** — bug fixes, dependency bumps, doc-only changes, internal refactors.

The version lives in three files that must stay in lockstep:

| File | Field |
|------|-------|
| `package.json` | `"version"` |
| `src-tauri/Cargo.toml` | `version =` |
| `src-tauri/tauri.conf.json` | `"version"` |

## Cutting a Release

### 1. Bump the version

Run the bump script from the repository root:

```bash
python scripts/bump-version.py --minor   # or --major or --patch
```

This will:
- Update all three version files.
- Update `Cargo.lock`.
- Run the full quality gate (lint, typecheck, tests, format checks).

If any check fails, fix the issues and re-run the script.

### 2. Update the changelog

Move the `## [Unreleased]` section to `## [X.Y.Z] — YYYY-MM-DD` with today's date:

```bash
# Edit CHANGELOG.md manually
```

Ensure all notable changes are categorized under `Added`, `Changed`, `Deprecated`, `Fixed`, or `Removed`.

### 3. Commit and create a PR

```bash
git add -A
git commit -s -m "release: vX.Y.Z"
git push origin main
gh pr create --base main --title "release: vX.Y.Z"
```

Wait for CI to pass, then merge.

### 4. Create and push a tag

After the PR is merged, switch to main and pull:

```bash
git checkout main
git pull
```

Create and push the tag (the `v` prefix is required by the release workflow):

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

This triggers the [Release workflow](../.github/workflows/release.yml) which:
- Builds binaries for macOS (aarch64), Linux, and Windows.
- Generates `latest.json` for the auto-updater.
- Creates a draft GitHub Release with all artifacts attached.

### 5. Publish the release

Go to [GitHub Releases](https://github.com/Llama-Recipe-Manager/llama-recipe-manager/releases), find the draft release, edit the body if needed, and click **Publish release**.

The auto-updater will now discover the new version for users.

## Hotfix Flow

To patch an already-released version:

```bash
git checkout main
git pull
python scripts/bump-version.py --patch
# Update CHANGELOG — move Unreleased to new patch version
git add -A && git commit -s -m "release: vX.Y.Z+1"
git push origin main
gh pr create --base main --title "release: vX.Y.Z+1"
# After merge:
git tag -a vX.Y.Z+1 -m "Release vX.Y.Z+1"
git push origin vX.Y.Z+1
```

## Notes

- The `TAURI_SIGNING_PRIVATE_KEY` secret must be set on the repository for signed auto-updates to work.
- The signing key does not change between releases — it is tied to the pubkey in `tauri.conf.json`.
- The `latest.json` file is generated at build time and contains asset download URLs pointing to the current repository.
