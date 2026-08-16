//! Vendor/OS-specific GPU detection via each platform's own CLI tooling.
//!
//! llama.cpp's `--version` is platform-independent but does not reliably list
//! GPU devices (CUDA device lines commonly omit VRAM, and some builds print
//! nothing at all). These helpers instead ask each vendor's tool — `nvidia-smi`
//! on NVIDIA, `rocm-smi` on AMD, `system_profiler` on Apple — and expose the
//! results as devices ready to populate the recipe `gpu_info` field.

use serde::Serialize;

/// A single detected GPU device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GpuDevice {
    /// Human-readable device name (e.g. "NVIDIA GeForce RTX 4090").
    pub name: String,
    /// VRAM in MiB (0 when unknown).
    pub vram_mib: u64,
    /// CUDA compute capability (e.g. "8.9"); empty for non-CUDA backends.
    pub compute_capability: String,
}

/// Detect GPUs using the best CLI tool available on this platform, in
/// priority order. Returns an empty vec when nothing could be detected.
pub async fn detect_gpu_devices() -> Vec<GpuDevice> {
    let mut devices = Vec::new();

    // Apple: system_profiler is authoritative and covers both Apple Silicon
    // and discrete AMD/Intel GPUs.
    #[cfg(target_os = "macos")]
    if devices.is_empty() {
        if let Some(d) = system_profiler_devices().await {
            devices = d;
        }
    }

    // NVIDIA: nvidia-smi is the canonical source on Linux, Windows, and macOS.
    if devices.is_empty() {
        if let Some(d) = nvidia_smi_devices().await {
            devices = d;
        }
    }

    // AMD on Linux: detect via rocm-smi.
    #[cfg(target_os = "linux")]
    if devices.is_empty() {
        if let Some(d) = rocm_smi_devices().await {
            devices = d;
        }
    }

    devices
}

/// Run a command and capture trimmed combined stdout+stderr. Returns `None` if
/// the binary could not be spawned or exited unsuccessfully.
async fn run(binary: &str, args: &[&str]) -> Option<String> {
    let output = tokio::process::Command::new(binary)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    let trimmed = text.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

// ── NVIDIA ──────────────────────────────────────────────────────────────

/// Candidates for the `nvidia-smi` binary, including a well-known Windows path
/// that is not always on PATH.
fn nvidia_smi_binaries() -> Vec<&'static str> {
    #[cfg(target_os = "windows")]
    {
        return vec!["nvidia-smi", "C:\\Windows\\System32\\nvidia-smi.exe"];
    }
    #[cfg(not(target_os = "windows"))]
    {
        vec!["nvidia-smi"]
    }
}

async fn nvidia_smi_devices() -> Option<Vec<GpuDevice>> {
    let args = [
        "--query-gpu=name,memory.total,compute_cap",
        "--format=csv,noheader,nounits",
    ];
    for binary in nvidia_smi_binaries() {
        if let Some(out) = run(binary, &args).await {
            if let Some(devices) = parse_nvidia_smi(&out) {
                return Some(devices);
            }
        }
    }
    None
}

/// Parse `nvidia-smi --query-gpu=name,memory.total,compute_cap` output.
///
/// Each line is `NAME, <memory MiB>, <compute cap>`, e.g.:
/// `NVIDIA GeForce RTX 4090, 24564, 8.9`
pub fn parse_nvidia_smi(out: &str) -> Option<Vec<GpuDevice>> {
    // nvidia-smi emits "No devices were found"/"No supported devices" when
    // the machine has no NVIDIA GPU.
    let lower = out.to_ascii_lowercase();
    if lower.contains("no devices") || lower.contains("no supported") {
        return None;
    }
    let mut devices = Vec::new();
    for line in out.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split(',');
        let Some(name) = parts.next() else { continue };
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        let vram_mib: u64 = parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        let compute_capability = parts
            .next()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        devices.push(GpuDevice {
            name: name.to_string(),
            vram_mib,
            compute_capability,
        });
    }
    if devices.is_empty() {
        None
    } else {
        Some(devices)
    }
}

// ── AMD (Linux) ─────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
async fn rocm_smi_devices() -> Option<Vec<GpuDevice>> {
    let out = run(
        "rocm-smi",
        &[
            "--showproductname",
            "--showmeminfo",
            "vram",
            "--showdriverversion",
        ],
    )
    .await?;
    parse_rocm_smi(&out)
}

/// Best-effort parse of `rocm-smi --showproductname --showmeminfo vram`.
///
/// rocm-smi emits tab-separated lines such as:
/// ```text
/// GPU[0]        : Name of GPU card        : AMD Radeon RX 7900 XTX
/// GPU[0]        : Vram total memory       : 24576 MB
/// ```
/// The parser is kept available (and unit-tested) on every platform so it can
/// be validated in CI even when not building for Linux.
#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
pub fn parse_rocm_smi(out: &str) -> Option<Vec<GpuDevice>> {
    let mut devices = Vec::new();
    let mut names: std::collections::HashMap<String, String> = std::collections::HashMap::default();
    let mut vrams: std::collections::HashMap<String, u64> = std::collections::HashMap::default();

    for line in out.lines() {
        let lower = line.to_ascii_lowercase();
        let is_name = lower.contains("name of gpu card") || lower.contains("product name");
        let is_vram = lower.contains("vram total memory")
            || lower.contains("vram memory")
            || lower.contains("total vram");
        if !is_name && !is_vram {
            continue;
        }
        // Key the entry by the "GPU[i]" prefix if present, else a global slot.
        let gpu_key = line
            .split_whitespace()
            .next()
            .filter(|t| t.starts_with("GPU["))
            .unwrap_or("_global")
            .to_string();
        let value = line.split(':').next_back().unwrap_or("").trim().to_string();
        if is_name && !value.is_empty() {
            names.insert(gpu_key.clone(), value);
        } else if is_vram {
            let mib = parse_mib(&value);
            vrams.insert(gpu_key, mib);
        }
    }

    let mut keys: Vec<String> = names.keys().cloned().collect();
    if keys.is_empty() {
        // Only a global VRAM with no name — fall back to a generic label in
        // the caller, so report nothing here.
        return None;
    }
    keys.sort();
    for key in keys {
        devices.push(GpuDevice {
            name: names.get(&key).cloned().unwrap_or_default(),
            vram_mib: vrams.get(&key).copied().unwrap_or(0),
            compute_capability: String::new(),
        });
    }

    if devices.iter().any(|d| !d.name.is_empty()) {
        Some(devices)
    } else {
        None
    }
}

// ── Apple (macOS) ───────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
async fn system_profiler_devices() -> Option<Vec<GpuDevice>> {
    for binary in ["system_profiler", "/usr/sbin/system_profiler"] {
        if let Some(out) = run(binary, &["SPDisplaysDataType"]).await {
            if let Some(devices) = parse_system_profiler(&out) {
                return Some(devices);
            }
        }
    }
    None
}

/// Parse `system_profiler SPDisplaysDataType`.
///
/// Each GPU appears as an indented block:
/// ```text
///     NVIDIA GeForce RTX 4090:
///
///       Chipset Model: NVIDIA GeForce RTX 4090
///       Type: GPU
///       VRAM (Total): 24 GB
///       Metal: Supported
/// ```
/// Apple Silicon entries have a `Chipset Model` but no `VRAM (Total)` line
/// (unified memory), so VRAM is left as 0 there.
///
/// The parser is kept available (and unit-tested) on every platform so it can
/// be validated in CI even when not building for macOS.
#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
pub fn parse_system_profiler(out: &str) -> Option<Vec<GpuDevice>> {
    let mut devices = Vec::new();
    let mut current: Option<GpuDevice> = None;
    let mut display_title: Option<String> = None;

    for line in out.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        // A block title is an indented line ending in ':' that is not a
        // "key: value" pair, e.g. "    NVIDIA GeForce RTX 4090:". The outer
        // "Graphics/Displays:" group header is not indented, so it is skipped.
        if line.starts_with(' ') && trimmed.ends_with(':') && !trimmed.contains(": ") {
            display_title = Some(trimmed.trim_end_matches(':').trim().to_string());
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "Chipset Model" => {
                if let Some(d) = current.take() {
                    if !d.name.is_empty() {
                        devices.push(d);
                    }
                }
                // Prefer the reliable "Chipset Model" value; fall back to the
                // block title only when it is missing.
                let mut name = value.to_string();
                if name.is_empty() {
                    name = display_title.take().unwrap_or_default();
                } else {
                    display_title.take(); // consume the stored title
                }
                current = Some(GpuDevice {
                    name,
                    vram_mib: 0,
                    compute_capability: String::new(),
                });
            }
            "VRAM (Total)" => {
                if let Some(dev) = current.as_mut() {
                    dev.vram_mib = parse_mib(value);
                } else if let Some(title) = display_title.take() {
                    // Some builds omit "Chipset Model" but still report VRAM.
                    current = Some(GpuDevice {
                        name: title,
                        vram_mib: parse_mib(value),
                        compute_capability: String::new(),
                    });
                }
            }
            _ => {}
        }
    }
    if let Some(d) = current.take() {
        if !d.name.is_empty() {
            devices.push(d);
        }
    }

    if devices.is_empty() {
        None
    } else {
        Some(devices)
    }
}

// ── shared helpers ──────────────────────────────────────────────────────

/// Parse a byte-amount string that may carry a suffix, e.g. `"24576 MB"`,
/// `"24 GB"`, `"24576.0 MiB"`. Returns the value in MiB (0 when unknown).
#[allow(clippy::cast_precision_loss)]
fn parse_mib(value: &str) -> u64 {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return 0;
    }
    let number: f64 = trimmed
        .split_whitespace()
        .next()
        .and_then(|n| {
            n.trim_matches(|c: char| !c.is_ascii_digit() && c != '.')
                .parse()
                .ok()
        })
        .unwrap_or(0.0);
    if number <= 0.0 {
        return 0;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("gb") {
        (number * 1024.0).round() as u64
    } else if lower.contains("mb") {
        let mib = number * 1_000_000.0 / 1_048_576.0;
        mib.round() as u64
    } else if lower.contains("kib") {
        number as u64
    } else {
        // Assume MiB when no unit is given.
        number.round() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nvidia_single_device() {
        let out = "NVIDIA GeForce RTX 4090, 24564, 8.9";
        let d = parse_nvidia_smi(out).expect("devices");
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].name, "NVIDIA GeForce RTX 4090");
        assert_eq!(d[0].vram_mib, 24_564);
        assert_eq!(d[0].compute_capability, "8.9");
    }

    #[test]
    fn nvidia_multiple_devices() {
        let out = "\
NVIDIA GeForce RTX 4090, 24564, 8.9
NVIDIA GeForce RTX 4090, 24564, 8.9
NVIDIA A100-SXM4-40GB, 40960, 8.0";
        let d = parse_nvidia_smi(out).expect("devices");
        assert_eq!(d.len(), 3);
        assert_eq!(d[2].name, "NVIDIA A100-SXM4-40GB");
        assert_eq!(d[2].vram_mib, 40_960);
    }

    #[test]
    fn nvidia_no_devices_returns_none() {
        assert!(parse_nvidia_smi("No devices were found\n").is_none());
        assert!(parse_nvidia_smi("").is_none());
    }

    #[test]
    fn rocm_smi_parse() {
        let out = "\
GPU[0]\t\t: Name of GPU card\t\t: AMD Radeon RX 7900 XTX
GPU[0]\t\t: Vram total memory\t\t: 24576 MB";
        let d = parse_rocm_smi(out).expect("devices");
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].name, "AMD Radeon RX 7900 XTX");
        // 24576 MB -> 23437.5 MiB (rounded)
        assert_eq!(d[0].vram_mib, 23_438);
    }

    #[test]
    fn rocm_smi_empty() {
        assert!(parse_rocm_smi("No devices found\n").is_none());
    }

    #[test]
    fn system_profiler_nvidia() {
        let out = "\
Graphics/Displays:

    NVIDIA GeForce RTX 4090:

      Chipset Model: NVIDIA GeForce RTX 4090
      Type: GPU
      Bus: PCIe
      PCIe Lane Width: x16
      VRAM (Total): 24 GB
      Vendor: NVIDIA (0x10de)
      Metal: Supported";
        let d = parse_system_profiler(out).expect("devices");
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].name, "NVIDIA GeForce RTX 4090");
        assert_eq!(d[0].vram_mib, 24 * 1024);
    }

    #[test]
    fn system_profiler_apple_silicon() {
        let out = "\
Graphics/Displays:

    Apple M4 Max:

      Chipset Model: Apple M4 Max
      Type: GPU
      Bus: Built-In
      Total Number of Cores: 40
      Vendor: Apple (0x106b)
      Metal: Metal 3";
        let d = parse_system_profiler(out).expect("devices");
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].name, "Apple M4 Max");
        assert_eq!(d[0].vram_mib, 0); // no VRAM line on Apple Silicon
    }

    #[test]
    fn parse_mib_units() {
        assert_eq!(parse_mib("24576 MB"), 23_438);
        assert_eq!(parse_mib("24 GB"), 24 * 1024);
        assert_eq!(parse_mib("16384 MiB"), 16_384);
        assert_eq!(parse_mib("16384"), 16_384);
        assert_eq!(parse_mib(""), 0);
    }
}
