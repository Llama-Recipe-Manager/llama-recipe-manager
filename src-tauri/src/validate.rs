//! Validation rules shared by the database and command layers.
//!
//! There are two flag deny-lists with different policies:
//!
//! * [`FORBIDDEN_FLAGS`] — **always** rejected. These either conflict with
//!   app-managed settings (host, port, model, mmproj, log file, TLS, HF token,
//!   timeouts, web UI / metrics / slots toggles, etc.) or would let a recipe
//!   silently override a value the user configured in Settings. Allowing them
//!   would produce confusing duplicate-flag errors at launch time.
//!
//! * [`UNSAFE_FLAGS`] — **not** blocked today. These are powerful capabilities
//!   (shell tools, MCP proxy, alternate model loaders, LoRA, arbitrary file
//!   read/write) that a local power-user may legitimately want. They must be
//!   blocked for *community-supplied* recipes once that feature ships, because
//!   they enable RCE, exfiltration, or arbitrary file access. Use
//!   [`validate_recipe_command_for_source`] with `RecipeSource::Community` to
//!   enforce that policy.
//!
//! Field-level hygiene ([`validate_recipe_fields`]) and path validation
//! (NUL / CR / LF / length caps) apply universally to every recipe regardless
//! of source.

/// Origin of a recipe. Determines which flag policies are enforced.
#[derive(Clone, Copy, Debug)]
pub enum RecipeSource {
    /// Authored by the local user. Only [`FORBIDDEN_FLAGS`] is enforced.
    Local,
    /// Imported from an untrusted source (community share, URL, etc.).
    /// Both [`FORBIDDEN_FLAGS`] and [`UNSAFE_FLAGS`] are enforced.
    ///
    /// Reserved for the upcoming community-recipes feature; not constructed
    /// yet by any caller.
    #[allow(dead_code)]
    Community,
}

/// Always-blocked flags. These conflict with app-managed settings; allowing
/// them would mean a recipe could either silently override a setting the user
/// configured, or produce a duplicate-flag error at launch.
pub const FORBIDDEN_FLAGS: &[&str] = &[
    // Network binding — managed by Settings
    "--port",
    "-p",
    "--host",
    // Model / mmproj — injected by us from the recipe's dedicated fields
    "-m",
    "--model",
    "--model-path",
    "--mmproj",
    "-mm",
    "--mmproj-url",
    "-mmu",
    // Logging — app captures stdout/stderr; --log-file would redirect away
    "--log-file",
    // Auth / TLS — managed by Settings
    "--api-key",
    "--api-key-file",
    "--ssl-cert-file",
    "--ssl-key-file",
    "--hf-token",
    "-hft",
    // Server endpoints / behaviour — managed by Settings
    "--api-prefix",
    "--timeout",
    "-to",
    "--log-verbosity",
    "-lv",
    "--webui",
    "--no-webui",
    "--metrics",
    "--slots",
    "--no-slots",
];

/// Capabilities that are powerful enough to be dangerous in an untrusted
/// recipe but useful for local power-users. Allowed today; enforced in the
/// future for community recipes via [`RecipeSource::Community`].
pub const UNSAFE_FLAGS: &[&str] = &[
    // Shell / MCP proxy — RCE-class
    "--tools",
    "--webui-mcp-proxy",
    "--no-webui-mcp-proxy",
    // Static directory served as files (info disclosure)
    "--path",
    // Alternate model loaders — bypass our model_path resolver and can fetch
    // arbitrary blobs from the network
    "-hf",
    "--hf-repo",
    "-hff",
    "--hf-file",
    "-hfd",
    "-hfrd",
    "--hf-repo-draft",
    "-hfv",
    "-hfrv",
    "--hf-repo-v",
    "-hffv",
    "--hf-file-v",
    "-mu",
    "--model-url",
    "-md",
    "--model-draft",
    // LoRA adapters — load arbitrary file into the model parser
    "--lora",
    "--lora-scaled",
    "--lora-base",
    // Filesystem read/write surface
    "--slot-save-path",
    "--media-path",
    "--models-dir",
    "--models-preset",
    "--models-max",
    "--models-autoload",
    "--no-models-autoload",
    // Web UI / template / grammar files — arbitrary file read
    "--webui-config",
    "--webui-config-file",
    "--grammar-file",
    "--json-schema-file",
    "-jf",
    "--chat-template-file",
];

/// Tokenise the command and ensure none of the deny-listed flags appear.
fn check_flags(command: &str, deny: &[&[&str]]) -> Result<(), String> {
    if command.contains('\0') {
        return Err("Recipe command contains a NUL byte".to_string());
    }

    let tokens =
        shell_words::split(command).map_err(|e| format!("Invalid command syntax: {}", e))?;

    for token in &tokens {
        let lower = token.to_lowercase();
        for list in deny {
            for flag in *list {
                if lower == *flag || lower.starts_with(&format!("{}=", flag)) {
                    return Err(format!(
                        "Recipe command must not contain '{}'. This flag is either managed by \
                         the app or has been blocked for safety reasons.",
                        flag
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Validate a recipe command using only [`FORBIDDEN_FLAGS`].
///
/// This is the policy applied to locally-authored recipes today and is kept
/// as a thin wrapper over [`validate_recipe_command_for_source`] with
/// [`RecipeSource::Local`] for callers that don't need the source distinction.
pub fn validate_recipe_command(command: &str) -> Result<(), String> {
    validate_recipe_command_for_source(command, RecipeSource::Local)
}

/// Validate a recipe command according to its source policy.
///
/// `Local` enforces [`FORBIDDEN_FLAGS`] only; `Community` enforces both
/// [`FORBIDDEN_FLAGS`] and [`UNSAFE_FLAGS`].
pub fn validate_recipe_command_for_source(
    command: &str,
    source: RecipeSource,
) -> Result<(), String> {
    match source {
        RecipeSource::Local => check_flags(command, &[FORBIDDEN_FLAGS]),
        RecipeSource::Community => check_flags(command, &[FORBIDDEN_FLAGS, UNSAFE_FLAGS]),
    }
}

// ── Field-level hygiene ──────────────────────────────────────────────────────

/// Maximum lengths for free-form recipe text fields. Generous; intended only
/// to cap pathological inputs (e.g. a 50 MB "name" from a malicious import),
/// not to constrain normal use.
pub const MAX_NAME_LEN: usize = 200;
pub const MAX_DESCRIPTION_LEN: usize = 4096;
pub const MAX_COMMAND_LEN: usize = 8192;
pub const MAX_PATH_LEN: usize = 4096;
pub const MAX_GPU_INFO_LEN: usize = 200;
pub const MAX_TAGS_LEN: usize = 500;

#[derive(Clone, Copy)]
enum FieldKind {
    /// Single-line text: rejects NUL, CR, LF.
    SingleLine,
    /// Multi-line text: rejects NUL only.
    MultiLine,
    /// Filesystem path: rejects NUL, CR, LF.
    Path,
}

fn check_field(name: &str, value: &str, max_len: usize, kind: FieldKind) -> Result<(), String> {
    if value.len() > max_len {
        return Err(format!(
            "{} is too long ({} bytes, max {})",
            name,
            value.len(),
            max_len
        ));
    }
    if value.contains('\0') {
        return Err(format!("{} contains a NUL byte", name));
    }
    match kind {
        FieldKind::SingleLine | FieldKind::Path => {
            if value.contains('\n') || value.contains('\r') {
                return Err(format!("{} must not contain newline characters", name));
            }
        }
        FieldKind::MultiLine => {}
    }
    Ok(())
}

/// Apply length + charset checks to every recipe field. Universal — applies
/// regardless of recipe source.
pub fn validate_recipe_fields(
    name: &str,
    description: &str,
    command: &str,
    model_path: &str,
    mmproj_path: &str,
    gpu_info: &str,
    tags: &str,
) -> Result<(), String> {
    check_field("Name", name, MAX_NAME_LEN, FieldKind::SingleLine)?;
    check_field(
        "Description",
        description,
        MAX_DESCRIPTION_LEN,
        FieldKind::MultiLine,
    )?;
    check_field("Command", command, MAX_COMMAND_LEN, FieldKind::MultiLine)?;
    check_field("Model path", model_path, MAX_PATH_LEN, FieldKind::Path)?;
    check_field("Mmproj path", mmproj_path, MAX_PATH_LEN, FieldKind::Path)?;
    check_field(
        "GPU info",
        gpu_info,
        MAX_GPU_INFO_LEN,
        FieldKind::SingleLine,
    )?;
    check_field("Tags", tags, MAX_TAGS_LEN, FieldKind::SingleLine)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(field_check: Result<(), String>) {
        assert!(field_check.is_ok(), "expected Ok, got {:?}", field_check);
    }

    fn err_contains(result: Result<(), String>, needle: &str) {
        match result {
            Ok(()) => panic!("expected error containing {:?}, got Ok", needle),
            Err(e) => assert!(
                e.contains(needle),
                "expected error containing {:?}, got {:?}",
                needle,
                e
            ),
        }
    }

    // ── command-flag deny-lists ──────────────────────────────────────────────

    #[test]
    fn local_recipes_allow_unsafe_flags() {
        for flag in UNSAFE_FLAGS {
            let cmd = format!("llama-server {}", flag);
            assert!(
                validate_recipe_command_for_source(&cmd, RecipeSource::Local).is_ok(),
                "{} should be allowed locally",
                flag
            );
        }
    }

    #[test]
    fn community_recipes_block_unsafe_flags() {
        for flag in UNSAFE_FLAGS {
            let cmd = format!("llama-server {}", flag);
            err_contains(
                validate_recipe_command_for_source(&cmd, RecipeSource::Community),
                flag,
            );
        }
    }

    #[test]
    fn forbidden_flags_blocked_for_every_source() {
        for flag in FORBIDDEN_FLAGS {
            let cmd = format!("llama-server {} foo", flag);
            err_contains(
                validate_recipe_command_for_source(&cmd, RecipeSource::Local),
                flag,
            );
            err_contains(
                validate_recipe_command_for_source(&cmd, RecipeSource::Community),
                flag,
            );
        }
    }

    #[test]
    fn flag_match_is_case_insensitive() {
        err_contains(
            validate_recipe_command("llama-server --HOST 0.0.0.0"),
            "--host",
        );
    }

    #[test]
    fn flag_equals_form_is_caught() {
        err_contains(
            validate_recipe_command("llama-server --host=0.0.0.0"),
            "--host",
        );
    }

    #[test]
    fn legitimate_recipe_passes() {
        ok(validate_recipe_command(
            "--ctx-size 8192 --n-gpu-layers 999 --threads 8",
        ));
    }

    #[test]
    fn invalid_shell_syntax_reports_clearly() {
        err_contains(
            validate_recipe_command("--quote \"unterminated"),
            "Invalid command syntax",
        );
    }

    #[test]
    fn nul_byte_in_command_rejected() {
        err_contains(validate_recipe_command("foo\0bar"), "NUL byte");
    }

    #[test]
    fn forbidden_and_unsafe_lists_are_disjoint() {
        for f in FORBIDDEN_FLAGS {
            assert!(
                !UNSAFE_FLAGS.contains(f),
                "{} appears in both FORBIDDEN_FLAGS and UNSAFE_FLAGS",
                f
            );
        }
    }

    // ── field-level hygiene ──────────────────────────────────────────────────

    fn good_fields() -> [&'static str; 7] {
        [
            "My recipe",
            "A description.",
            "--ctx 8192",
            "model.gguf",
            "",
            "RTX 4090",
            "vision,chat",
        ]
    }

    #[test]
    fn good_fields_pass() {
        let f = good_fields();
        ok(validate_recipe_fields(
            f[0], f[1], f[2], f[3], f[4], f[5], f[6],
        ));
    }

    #[test]
    fn name_length_capped() {
        let big = "x".repeat(MAX_NAME_LEN + 1);
        let f = good_fields();
        err_contains(
            validate_recipe_fields(&big, f[1], f[2], f[3], f[4], f[5], f[6]),
            "Name is too long",
        );
    }

    #[test]
    fn name_rejects_newlines() {
        let f = good_fields();
        err_contains(
            validate_recipe_fields("multi\nline", f[1], f[2], f[3], f[4], f[5], f[6]),
            "newline",
        );
    }

    #[test]
    fn description_allows_newlines() {
        let f = good_fields();
        ok(validate_recipe_fields(
            f[0],
            "line1\nline2",
            f[2],
            f[3],
            f[4],
            f[5],
            f[6],
        ));
    }

    #[test]
    fn paths_reject_newlines() {
        let f = good_fields();
        err_contains(
            validate_recipe_fields(f[0], f[1], f[2], "model\n.gguf", f[4], f[5], f[6]),
            "Model path",
        );
        err_contains(
            validate_recipe_fields(f[0], f[1], f[2], f[3], "mm\r.gguf", f[5], f[6]),
            "Mmproj path",
        );
    }

    #[test]
    fn nul_in_any_field_rejected() {
        let f = good_fields();
        err_contains(
            validate_recipe_fields(f[0], "with\0nul", f[2], f[3], f[4], f[5], f[6]),
            "NUL byte",
        );
    }

    #[test]
    fn tags_capped() {
        let big = "a,".repeat(MAX_TAGS_LEN);
        let f = good_fields();
        err_contains(
            validate_recipe_fields(f[0], f[1], f[2], f[3], f[4], f[5], &big),
            "Tags is too long",
        );
    }
}
