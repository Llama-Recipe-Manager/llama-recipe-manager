# Security Policy

## Reporting a vulnerability

If you believe you have found a security vulnerability in Llama Recipe
Manager, **please do not file a public GitHub issue**. Instead, report it
privately so we can fix it before it is widely known.

Use one of:

- GitHub's [private vulnerability reporting](https://github.com/Llama-Recipe-Manager/llama-recipe-manager/security/advisories/new)
  on this repository (preferred).
- Email the maintainer at <ashar786khan@gmail.com>.

Please include:

- A description of the issue and the impact.
- Steps to reproduce, or a proof-of-concept recipe / payload.
- The version / commit you tested against.
- Whether you'd like to be credited in the advisory.

We aim to acknowledge reports within **3 business days** and to ship a fix or
mitigation within **30 days** for high-severity issues.

## Threat model

Llama Recipe Manager runs `llama-server` locally on the user's machine using
recipes that the user authored. Because a recipe is, in essence, a
command-line plus a model path, the surface area is non-trivial. The current
mitigations are:

- **Flag deny-list**: recipe commands are tokenised and rejected if they
  contain any flag that conflicts with app-managed settings (host, port,
  model, mmproj, TLS, API key, HF token, log file, web UI / metrics / slots
  toggles, timeouts, log verbosity, API prefix). This prevents a recipe
  from silently overriding security-relevant settings the user configured
  in the Settings page.
- **Path containment**: relative model / mmproj paths are joined against
  the configured `model_dir`, then canonicalised, and the result must
  remain inside that directory. This blocks `../../etc/passwd`-style
  escapes.
- **Field hygiene**: every recipe text field has a length cap and rejects
  NUL / CR / LF where appropriate, to defend against log-injection and
  pathological inputs.

The canonical sources are `src-tauri/src/validate.rs` and
`src-tauri/src/process.rs`.

## Out of scope

- Bugs in upstream `llama-server` itself. Please report those to
  [llama.cpp](https://github.com/ggml-org/llama.cpp).
- Vulnerabilities that require already-compromised local user privileges
  (e.g. an attacker who can already write to your home directory).
- Defects in third-party model files. We do not download, sandbox, or scan
  model weights.

## Disclosure

Once a fix is shipped we will publish a GitHub Security Advisory with credit
to the reporter (unless they prefer to remain anonymous).
