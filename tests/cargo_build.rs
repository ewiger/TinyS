//! Tests for the Cargo-backed build driver: manifest discovery, the generated
//! scratch package, and binary naming.
//!
//! Everything here builds dependency-free programs, so it works offline.

use std::path::{Path, PathBuf};
use std::process::Command;

fn cargo_available() -> bool {
    Command::new("cargo")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn tinys() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tinys"))
}

/// A fresh directory under the system temp dir, with no `tinys.toml` above it.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tinys-test-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dir");
    }
    std::fs::write(path, contents).expect("write file");
}

const HELLO: &str = "def main() -> void:\n    print(\"hi\")\n";

#[test]
fn builds_without_a_manifest() {
    if !cargo_available() {
        return;
    }
    let dir = scratch("no-manifest");
    let source = dir.join("solo.sn");
    write(&source, HELLO);

    let out = tinys()
        .args(["run", source.to_str().unwrap()])
        .output()
        .expect("failed to spawn tinys");
    assert!(
        out.status.success(),
        "build failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "hi");

    let generated = dir.join("target/tinys-generated/solo");
    assert!(generated.join("Cargo.toml").is_file());
    assert!(generated.join("src/main.rs").is_file());
    assert!(dir
        .join("target/tinys-generated/cargo-target/debug/solo")
        .exists());
}

/// Output goes next to `tinys.toml`, not next to the source file, and
/// `src/main.sn` is named after the package like Cargo's own entry point.
#[test]
fn manifest_sets_the_package_root_and_binary_name() {
    if !cargo_available() {
        return;
    }
    let dir = scratch("manifest-root");
    write(
        &dir.join("tinys.toml"),
        "[package]\nname = \"greeter\"\nversion = \"2.3.4\"\n",
    );
    write(&dir.join("src/main.sn"), HELLO);

    let out = tinys()
        .args(["build", dir.join("src/main.sn").to_str().unwrap()])
        .output()
        .expect("failed to spawn tinys");
    assert!(
        out.status.success(),
        "build failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let manifest = std::fs::read_to_string(dir.join("target/tinys-generated/main/Cargo.toml"))
        .expect("generated Cargo.toml");
    assert!(manifest.contains("name = \"greeter\""), "{manifest}");
    assert!(manifest.contains("version = \"2.3.4\""), "{manifest}");
    assert!(dir
        .join("target/tinys-generated/cargo-target/debug/greeter")
        .exists());
    // Nothing was written beside the source file.
    assert!(!dir.join("src/target").exists());
}

#[test]
fn unsupported_manifest_sections_are_reported() {
    if !cargo_available() {
        return;
    }
    let dir = scratch("unsupported-section");
    write(
        &dir.join("tinys.toml"),
        "[package]\nname = \"warned\"\n\n[dev-dependencies]\ntempfile = \"3\"\n",
    );
    write(&dir.join("app.sn"), HELLO);

    let out = tinys()
        .args(["check", dir.join("app.sn").to_str().unwrap()])
        .output()
        .expect("failed to spawn tinys");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "check failed:\n{stderr}");
    assert!(
        stderr.contains("unsupported section `[dev-dependencies]`"),
        "expected a warning, got:\n{stderr}"
    );
}
