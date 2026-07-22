//! Multi-file module discovery: the `src/` tree becomes the module tree, with
//! no `mod` keyword anywhere in TinyS source.

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

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tinys-mod-{}-{}", name, std::process::id()));
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

fn emit(entry: &Path) -> String {
    let out = tinys()
        .args(["emit-rust", entry.to_str().unwrap()])
        .output()
        .expect("failed to spawn tinys");
    assert!(
        out.status.success(),
        "emit-rust failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

const MAIN: &str = "def main() -> void:\n    print(\"hi\")\n";

/// A package with `tinys.toml` but no `src/` is a directory of single-file
/// programs — which is what keeps the flat `examples/` layout working.
#[test]
fn no_src_directory_means_no_modules() {
    let dir = scratch("no-src");
    write(&dir.join("tinys.toml"), "[package]\nname = \"flat\"\n");
    write(&dir.join("app.sn"), MAIN);
    write(
        &dir.join("sibling.sn"),
        "pub def helper() -> void:\n    print(\"x\")\n",
    );

    let rust = emit(&dir.join("app.sn"));
    assert!(!rust.contains("pub mod"), "{rust}");
    assert!(!rust.contains("sibling"), "{rust}");
}

#[test]
fn src_tree_becomes_the_module_tree() {
    let dir = scratch("src-tree");
    write(&dir.join("tinys.toml"), "[package]\nname = \"tree\"\n");
    write(&dir.join("src/app.sn"), MAIN);
    write(
        &dir.join("src/models.sn"),
        "pub def one() -> i32:\n    return 1\n",
    );
    write(
        &dir.join("src/services/mod.sn"),
        "pub def two() -> i32:\n    return 2\n",
    );
    write(
        &dir.join("src/services/store.sn"),
        "pub def three() -> i32:\n    return 3\n",
    );

    let rust = emit(&dir.join("src/app.sn"));
    assert!(rust.contains("pub mod models;"), "{rust}");
    assert!(rust.contains("pub mod services;"), "{rust}");
    // `store` is declared by `services`, not by the crate root.
    assert!(rust.contains("// ---- services/mod.rs"), "{rust}");
    assert!(rust.contains("pub mod store;"), "{rust}");
    assert!(rust.contains("// ---- services/store.rs"), "{rust}");
    // The entry file is the crate root, never also a module.
    assert!(!rust.contains("pub mod app;"), "{rust}");
}

/// `main.sn` is not required: the file you build is the crate root.
#[test]
fn any_file_can_be_the_crate_root() {
    let dir = scratch("any-root");
    write(&dir.join("tinys.toml"), "[package]\nname = \"roots\"\n");
    write(&dir.join("src/first.sn"), MAIN);
    write(&dir.join("src/second.sn"), MAIN);

    let rust = emit(&dir.join("src/first.sn"));
    assert!(rust.contains("pub mod second;"), "{rust}");
    assert!(!rust.contains("pub mod first;"), "{rust}");

    // Building the other one flips which is the root.
    let rust = emit(&dir.join("src/second.sn"));
    assert!(rust.contains("pub mod first;"), "{rust}");
    assert!(!rust.contains("pub mod second;"), "{rust}");
}

#[test]
fn test_files_are_declared_cfg_test() {
    let dir = scratch("cfg-test");
    write(&dir.join("tinys.toml"), "[package]\nname = \"tested\"\n");
    write(&dir.join("src/app.sn"), MAIN);
    write(
        &dir.join("src/models.sn"),
        "pub def one() -> i32:\n    return 1\n",
    );
    write(
        &dir.join("src/models_test.sn"),
        "#[test]\ndef works() -> void:\n    print(\"ok\")\n",
    );

    let rust = emit(&dir.join("src/app.sn"));
    assert!(
        rust.contains("#[cfg(test)]\npub mod models_test;"),
        "{rust}"
    );
    // A plain module is not gated.
    assert!(rust.contains("pub mod models;"), "{rust}");
    assert!(!rust.contains("#[cfg(test)]\npub mod models;"), "{rust}");
}

/// A directory with no `mod.sn` still exists as a module holding its children.
#[test]
fn directories_without_mod_sn_are_synthesized() {
    let dir = scratch("synth-dir");
    write(&dir.join("tinys.toml"), "[package]\nname = \"synth\"\n");
    write(&dir.join("src/app.sn"), MAIN);
    write(
        &dir.join("src/util/text.sn"),
        "pub def one() -> i32:\n    return 1\n",
    );

    let rust = emit(&dir.join("src/app.sn"));
    assert!(rust.contains("pub mod util;"), "{rust}");
    assert!(rust.contains("// ---- util/mod.rs"), "{rust}");
    assert!(rust.contains("pub mod text;"), "{rust}");
}

#[test]
fn excluded_files_stay_out_of_the_tree() {
    let dir = scratch("exclude");
    write(
        &dir.join("tinys.toml"),
        "[package]\nname = \"skipping\"\nexclude = [\"scratch\", \"*_wip.sn\"]\n",
    );
    write(&dir.join("src/app.sn"), MAIN);
    write(
        &dir.join("src/models.sn"),
        "pub def one() -> i32:\n    return 1\n",
    );
    write(
        &dir.join("src/draft_wip.sn"),
        "this is not valid tinys at all\n",
    );
    write(&dir.join("src/scratch/broken.sn"), "neither is this\n");

    let rust = emit(&dir.join("src/app.sn"));
    assert!(rust.contains("pub mod models;"), "{rust}");
    assert!(!rust.contains("draft_wip"), "{rust}");
    assert!(!rust.contains("scratch"), "{rust}");
}

/// `foo.sn` beside `foo/` would declare one module name twice.
#[test]
fn conflicting_module_names_are_rejected() {
    let dir = scratch("conflict");
    write(&dir.join("tinys.toml"), "[package]\nname = \"clash\"\n");
    write(&dir.join("src/app.sn"), MAIN);
    write(
        &dir.join("src/models.sn"),
        "pub def one() -> i32:\n    return 1\n",
    );
    write(
        &dir.join("src/models/deep.sn"),
        "pub def two() -> i32:\n    return 2\n",
    );

    let out = tinys()
        .args(["emit-rust", dir.join("src/app.sn").to_str().unwrap()])
        .output()
        .expect("failed to spawn tinys");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("declares the module `models` twice"),
        "{stderr}"
    );
}

/// The bundled multi-file example builds, runs, and its `_test.sn` module is
/// excluded from the binary but picked up by `cargo test`.
#[test]
fn modules_example_builds_and_runs() {
    if !cargo_available() {
        return;
    }
    let out = tinys()
        .args(["run", "examples/modules/src/app.sn"])
        .output()
        .expect("failed to spawn tinys");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "run failed:\n{stderr}");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines,
        vec![
            "#1 Ada (active)",
            "#2 Grace (inactive)",
            "#3 Alan (active)",
            "2 of 3 are active",
        ]
    );

    // `exclude` kept the deliberately broken scratch file out of the build.
    let generated = Path::new("examples/modules/target/tinys-generated/app/src");
    assert!(generated.join("services/store.rs").is_file());
    assert!(!generated.join("scratch").exists());
}
