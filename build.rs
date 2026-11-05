extern crate napi_build;

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    napi_build::setup();

    // Short git commit hash, or "unknown" when git is not available (e.g. a source tarball).
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    // "true" when the working tree has uncommitted or untracked changes, "false" when clean,
    // "unknown" when git is not available.
    let git_dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| (!o.stdout.is_empty()).to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Build time as UNIX seconds. Re-evaluated whenever this script reruns.
    let built_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    println!("cargo:rustc-env=VECT_GIT_HASH={git_hash}");
    println!("cargo:rustc-env=VECT_GIT_DIRTY={git_dirty}");
    println!("cargo:rustc-env=VECT_BUILD_UNIX={built_at_unix}");

    // Rerun when anything that is compiled into the binary changes, not only when git HEAD moves,
    // so the build time and dirty flag stay current.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-changed=.git/refs");
}
