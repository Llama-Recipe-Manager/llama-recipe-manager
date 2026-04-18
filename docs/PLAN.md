# Plan — Community Recipes

Status: design / not yet implemented. UI scaffolding (nav rail, Community
placeholder view, filter chips) is shipped and wired but reads no data.

This document is the source of truth for the upcoming **Community Recipes**
feature: shared recipes that can be browsed, filtered by hardware, forked
into a user's local library, and validated against benchmark claims.

---

## 1. Goals

1. Let users discover working `llama-server` configurations published by
   other users for specific GPUs and backends.
2. Let publishers attach **performance claims** (prompt-processing tok/s,
   token-generation tok/s) to a recipe, scoped to the GPU + backend they
   measured on.
3. Let other users **validate** those claims by running the recipe on
   matching hardware and submitting their measured numbers, building a
   reputation signal per recipe.
4. Let users **fork/clone** a community recipe into their local library,
   while tracking real popularity (deduplicated per machine) so that a
   single user can't artificially inflate a recipe's fork count.
5. Keep the local-only experience completely OAuth-free: signing in is
   only required to _publish_, _fork_, or _validate_ — browsing is
   anonymous.

## 2. Non-goals (v1)

- Hosting model weights. Recipes reference models by their own paths
  (local) or HF identifiers (community); the platform never proxies
  weights.
- Comments / threaded discussion. Reactions and validations only.
- Cross-OS portability claims. A claim is always "this hardware + this OS
  family + this backend".

---

## 3. Data model additions

All schema additions go through new migration files in
`src-tauri/src/migrations/` (see `docs/database.md` migration process).

### 3.1 `recipes` table — new columns

| column          | type                            | notes                             |
| --------------- | ------------------------------- | --------------------------------- |
| `source`        | TEXT NOT NULL DEFAULT `'local'` | `'local'` \| `'community'`        |
| `community_id`  | TEXT NULL                       | server-side recipe id when forked |
| `origin_author` | TEXT NULL                       | display name from the platform    |
| `backend`       | TEXT NOT NULL DEFAULT `'auto'`  | see §3.4                          |
| `forked_at`     | TEXT NULL                       | ISO-8601 timestamp                |

Local-only recipes keep all the new columns at their defaults.

### 3.2 `community_claims` (new table, _future_ — not used by the desktop DB)

Lives on the server, not the SQLite DB. Listed here for completeness:

```text
community_claims(
  id, recipe_id, author_id,
  gpu_model, gpu_vram_mib, backend, os_family,
  prompt_processing_tps, token_generation_tps,
  measured_at, llama_server_version
)
```

### 3.3 `community_validations` (new table, _future_, server-side)

```text
community_validations(
  id, claim_id, validator_id, validator_machine_fp,
  prompt_processing_tps, token_generation_tps,
  measured_at, agreement /* 'matches' | 'lower' | 'higher' */
)
```

### 3.4 Backend enum

Stored as TEXT for forward compatibility:

- `auto` — let llama.cpp decide (default for legacy recipes)
- `cuda` — NVIDIA CUDA
- `vulkan` — cross-vendor Vulkan compute
- `rocm` — AMD ROCm
- `metal` — Apple Silicon
- `cpu` — pure CPU build

Used to filter community recipes and to label a local recipe so the user
remembers which build of `llama-server` it was tested against.

---

## 4. Authentication

### 4.1 Provider

GitHub OAuth (lowest friction for the LLM-tinkerer audience; everyone
already has an account; no email/password to babysit).

### 4.2 Flow

PKCE OAuth in an external browser (no embedded webview):

1. Tauri command `community::login_start` generates `code_verifier` + `state`,
   opens the system browser to the GitHub authorize URL with a
   `redirect_uri` of `http://127.0.0.1:<random-port>/oauth/callback`.
2. Tauri spawns a one-shot `axum`/`tiny_http` listener on that port that
   captures the authorization code, then shuts down.
3. Code + verifier exchanged for an access token by the **community
   backend** (not the desktop app), which stores its own session id.
4. Desktop app receives `{ session_id, expires_at, user: {...} }` and
   stores the session id in the OS keychain via `keyring` crate (NOT in
   the SQLite DB).

### 4.3 Boundaries

- The desktop app never sees a GitHub token. Only an opaque session id.
- Logging out clears the keychain entry; no server round-trip required
  for offline usage to keep working.
- All anonymous browsing endpoints (`GET /recipes`, `GET /recipes/:id`)
  must work without a session cookie.

---

## 5. Machine fingerprint

Used to deduplicate fork counts and tie validation submissions to a
distinct device (without identifying the user).

### 5.1 Inputs

A SHA-256 over a stable, JSON-serialised tuple:

- CPU brand string + physical core count
- Total system RAM (rounded to nearest GiB)
- Each GPU's PCI vendor:device id + VRAM size (rounded)
- OS family + arch (`linux/x86_64`, `darwin/aarch64`, ...)

We deliberately exclude MAC addresses, hostnames, disk serials, and
anything else that screams "tracker". The fingerprint is stable enough
to detect "same machine forking the same recipe twice" and weak enough
that a user can change one component and get a fresh id.

### 5.2 Storage

Computed once per launch, cached in memory. Submitted as
`X-Machine-Fingerprint` only on `POST /recipes/:id/fork` and
`POST /claims/:id/validate`. Never sent on browse requests.

### 5.3 Server-side use

- Fork endpoint: `INSERT ... ON CONFLICT (recipe_id, machine_fp) DO NOTHING`,
  the response says whether it counted.
- Validation endpoint: `UNIQUE (claim_id, machine_fp)`, last-write-wins.

---

## 6. Community backend (separate repo, future)

Stack proposal: Rust + `axum` + Postgres + `sqlx`. Endpoints:

| method | path                    | auth         | purpose                                                                  |
| ------ | ----------------------- | ------------ | ------------------------------------------------------------------------ |
| `GET`  | `/recipes`              | none         | list, paginated, with filters                                            |
| `GET`  | `/recipes/:id`          | none         | full recipe + claims + top validations                                   |
| `POST` | `/recipes`              | session      | publish (server runs `validate_recipe_command_for_source(_, Community)`) |
| `POST` | `/recipes/:id/fork`     | session + fp | bump fork counter                                                        |
| `POST` | `/recipes/:id/claims`   | session      | publish a benchmark claim                                                |
| `POST` | `/claims/:id/validate`  | session + fp | submit a validation result                                               |
| `GET`  | `/me`                   | session      | current user info                                                        |
| `POST` | `/auth/github/exchange` | none         | token exchange (called by desktop)                                       |

Filters on `GET /recipes`:

- `gpu` — substring match on GPU model
- `vram_min`, `vram_max` — MiB
- `backend` — one of the §3.4 values
- `capability` — `chat` \| `vision` \| `embedding` (derived from flags)
- `min_tg_tps`, `min_pp_tps` — claimed performance floor
- `sort` — `recent` \| `forks` \| `validated`

---

## 7. Security

The single most important guarantee: **a community recipe must not be
runnable until it has passed `validate_recipe_command_for_source(cmd,
RecipeSource::Community)`.** That gate already exists in
`src-tauri/src/validate.rs` and rejects every flag in `UNSAFE_FLAGS`
(shell tools, MCP proxy, alternate model loaders, LoRA adapters,
filesystem r/w surface, web-UI/template/grammar files).

Additional rules:

- `model_path` and `mmproj_path` on a community recipe must be HuggingFace
  identifiers of the form `org/repo[:filename]` — _never_ a local path. The
  fork action resolves them into a local path under `model_dir` after
  download.
- Server-side `POST /recipes` re-runs the same Rust validator (compiled
  for the backend) so a malicious client can't bypass it.
- Forks are stored as `source = 'community'` and re-validated on each
  load. If `UNSAFE_FLAGS` ever expands, previously-acceptable forks may
  be rejected; the UI shows them in a "needs review" state.
- Sessions: short-lived JWT-style id (24h sliding), refresh on activity.
- Rate limits: 30 req/min per session for writes.

## 8. UI

### 8.1 Shell (shipped now)

- **Nav rail** (56 px, far left): My Recipes / Community / Settings,
  plus a footer slot for "Account" (sign-in CTA today, avatar + menu
  later) and a "Running" indicator that flashes when a server is up.
- **Sub-sidebar** (280 px) — content depends on the section:
  - My Recipes → search + recipe list (today's behaviour).
  - Community → search + filter chips (GPU substring, VRAM range,
    backend, capability, sort). Today: chips render but the list is
    a placeholder.
  - Settings → no sub-sidebar.
- **Main pane** — current detail/form for My Recipes; placeholder
  "Sign in to browse community recipes" with a sample card for Community;
  full-width settings.

### 8.2 Community list (future)

- Card grid, 3 columns at desktop widths.
- Each card: name, author handle, GPU + backend badges, claimed PP/TG
  tok/s, fork count, validation count, "fork" button.
- Detail page: full command preview (read-only), claims table sorted by
  validations, "Validate on my hardware" CTA (only enabled when the
  user's fingerprint matches the claim's GPU class).

### 8.3 Recipe form additions (future)

- `backend` dropdown (§3.4) on every recipe.
- "Publish to community" button on local recipes (only after sign-in;
  re-runs the community validator before submitting).
- Claim editor (PP / TG numbers) tied to current machine fingerprint.

## 9. Phasing

| Phase   | Scope                                                                                     |
| ------- | ----------------------------------------------------------------------------------------- |
| 0 (now) | UI shell + nav rail + Community placeholder + this plan doc.                              |
| 1       | Schema migration: add `source`/`backend`/`community_id`. Backend dropdown in recipe form. |
| 2       | Stand up community backend (auth, GET endpoints, fingerprint).                            |
| 3       | Browse + filter + fork in the desktop app (read-only Community).                          |
| 4       | Publish + claim + validate flows.                                                         |
| 5       | Reputation surfaces (validator agreement, sort by validated).                             |

## 10. Open questions

- Do we want anonymous validation (no sign-in)? Strong fingerprint dedupe
  could allow it, but a hostile actor with many machines can still
  poison. Default: require sign-in.
- Do we let users **un-fork**? Yes — local delete just removes the row;
  the server keeps the count immutable to avoid griefing the original
  author.
- How do we deal with recipes that were valid when published but contain
  a flag that later gets added to `UNSAFE_FLAGS`? Show the recipe in a
  read-only "deprecated" state with an explanation; do not auto-delete.
