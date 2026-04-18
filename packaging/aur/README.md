# Arch Linux (AUR) packaging

This directory holds the source-of-truth PKGBUILDs for both AUR packages.
The workflow at `.github/workflows/aur.yml` keeps AUR in sync — you should
**never edit the PKGBUILDs on AUR directly**.

## Packages

| AUR package                                                                               | Source                                                 | Updated by                                                  |
| ----------------------------------------------------------------------------------------- | ------------------------------------------------------ | ----------------------------------------------------------- |
| [`llama-recipe-manager`](https://aur.archlinux.org/packages/llama-recipe-manager)         | `archive/refs/tags/v$pkgver.tar.gz` (a tagged release) | the `stable` job, on every GitHub Release                   |
| [`llama-recipe-manager-git`](https://aur.archlinux.org/packages/llama-recipe-manager-git) | `git+…/llama-recipe-manager.git` (`main`)              | the `git` job, on changes to `packaging/aur/**` or manually |

`llama-recipe-manager-git` provides + conflicts with `llama-recipe-manager`,
so users pick exactly one.

## One-time maintainer setup

1. **Create the packages on AUR.** From a local machine:

   ```bash
   # Clone the (initially empty) AUR repos.
   git clone ssh://aur@aur.archlinux.org/llama-recipe-manager.git
   git clone ssh://aur@aur.archlinux.org/llama-recipe-manager-git.git

   # Drop the rendered PKGBUILD + .SRCINFO inside, commit, push.
   # (After this first push the GitHub workflow takes over.)
   ```

2. **Add three GitHub repository secrets** (Settings → Secrets and variables → Actions):

   | Secret                | Value                                                                             |
   | --------------------- | --------------------------------------------------------------------------------- |
   | `AUR_USERNAME`        | your AUR account username                                                         |
   | `AUR_EMAIL`           | the email registered with that account                                            |
   | `AUR_SSH_PRIVATE_KEY` | private half of the SSH key you registered at <https://aur.archlinux.org/account> |

3. Push. Subsequent releases publish automatically.

## Editing the PKGBUILDs

- **Stable**: edit `llama-recipe-manager/PKGBUILD.tmpl`. The `__PKGVER__`
  placeholder is replaced at publish time with the release tag (minus
  the leading `v`); `sha256sums` is computed on the fly by `updpkgsums`
  inside an Arch container.
- **Git**: edit `llama-recipe-manager-git/PKGBUILD` directly. `pkgver` is
  computed at build-time on each user's machine via the `pkgver()` helper.
- Pushing changes under `packaging/aur/**` to `main` re-publishes the git
  package automatically. The stable package only re-publishes on a new
  GitHub Release — if you need to push a `pkgrel` bump without a new
  upstream version, run the workflow manually with `package: stable`.

## Local sanity check

To validate a PKGBUILD before pushing, on an Arch box:

```bash
cd packaging/aur/llama-recipe-manager-git
makepkg --printsrcinfo > /tmp/.SRCINFO   # validates syntax
makepkg -si                              # full build, install
```

The CI runs `test: true` which performs the equivalent inside an Arch
container, so failures surface in the workflow log before anything
reaches AUR.
