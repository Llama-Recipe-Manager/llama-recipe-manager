// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::path::Path;

fn main() {
    // NVIDIA proprietary drivers often fail to create a surfaceless EGL
    // display (EGL_BAD_ALLOC) in WebKitGTK's web process, especially on
    // Wayland where explicit sync can also cause flickering/crashes.
    // Detect NVIDIA by checking for /proc/driver/nvidia (created by
    // nvidia.ko) and apply Wayland-specific workarounds.
    let has_nvidia = Path::new("/proc/driver/nvidia").exists();
    if has_nvidia && env::var_os("__NV_DISABLE_EXPLICIT_SYNC").is_none() {
        unsafe { env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1") };
    }

    llama_recipe_manager_lib::run();
}
