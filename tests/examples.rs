//! End-to-end tests: drive the `tinys` binary to compile the `.sn` examples with
//! Cargo and check that the runnable ones produce the expected output.
//!
//! These are skipped automatically when `cargo` is unavailable.

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

/// Examples that are pure-std and therefore build without touching the network.
const RUNNABLE: &[&str] = &[
    "hello",
    "functions",
    "control_flow",
    "structs",
    "enums",
    "references",
    "closures",
    "fizzbuzz",
    "generics",
    "loops",
    "macros",
];

#[test]
fn all_runnable_examples_typecheck() {
    if !cargo_available() {
        eprintln!("skipping: cargo not available");
        return;
    }
    for name in RUNNABLE {
        let path = format!("examples/{name}.sn");
        let out = tinys()
            .args(["check", &path])
            .output()
            .expect("failed to spawn tinys");
        assert!(
            out.status.success(),
            "`tinys check {path}` failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

fn run_example(name: &str) -> String {
    let path = format!("examples/{name}.sn");
    let out = tinys()
        .args(["run", &path])
        .output()
        .expect("failed to spawn tinys");
    assert!(
        out.status.success(),
        "`tinys run {path}` failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn hello_prints_greeting() {
    if !cargo_available() {
        return;
    }
    assert_eq!(run_example("hello").trim(), "Hello from TinyS");
}

#[test]
fn functions_compute_expected_values() {
    if !cargo_available() {
        return;
    }
    let out = run_example("functions");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["7", "25", "20"]);
}

#[test]
fn control_flow_output() {
    if !cargo_available() {
        return;
    }
    let out = run_example("control_flow");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines,
        vec!["15", "3", "2", "1", "negative", "zero", "positive"]
    );
}

#[test]
fn fizzbuzz_output() {
    if !cargo_available() {
        return;
    }
    let out = run_example("fizzbuzz");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.first(), Some(&"1"));
    assert_eq!(lines.get(2), Some(&"Fizz"));
    assert_eq!(lines.get(4), Some(&"Buzz"));
    assert_eq!(lines.get(14), Some(&"FizzBuzz"));
}

#[test]
fn references_output() {
    if !cargo_available() {
        return;
    }
    let out = run_example("references");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["2", "10"]);
}

#[test]
fn macros_output() {
    if !cargo_available() {
        return;
    }
    // `debug` writes to stderr, so only the two `print` calls reach stdout.
    let out = run_example("macros");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["3", "#7 ada"]);
}

#[test]
fn emit_rust_showcase_smoke() {
    let out = tinys()
        .args(["emit-rust", "examples/json_user.sn"])
        .output()
        .expect("failed to spawn tinys");
    assert!(out.status.success());
    let rust = String::from_utf8_lossy(&out.stdout);
    assert!(rust.contains("serde_json::from_str::<User>"));
    assert!(rust.contains("fn main() -> Result<(), serde_json::Error>"));
}

/// Only the crates a program imports reach the generated `Cargo.toml`, so the
/// std-only examples never pull in the `serde` dependencies of `tinys.toml`.
#[test]
fn generated_manifest_only_lists_imported_crates() {
    if !cargo_available() {
        return;
    }
    run_example("hello");
    let manifest = std::fs::read_to_string("examples/target/tinys-generated/hello/Cargo.toml")
        .expect("generated Cargo.toml");
    assert!(manifest.contains("name = \"hello\""));
    assert!(!manifest.contains("serde"));
}

/// The interop example needs `serde`/`serde_json` from `examples/tinys.toml`,
/// so it is skipped when the crates cannot be fetched (offline sandboxes).
#[test]
fn json_user_builds_and_runs_with_cargo_dependencies() {
    if !cargo_available() {
        return;
    }
    let out = tinys()
        .args(["run", "examples/json_user.sn"])
        .output()
        .expect("failed to spawn tinys");
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !out.status.success() {
        let offline = [
            "failed to fetch",
            "network",
            "offline",
            "no matching package",
        ]
        .iter()
        .any(|needle| stderr.contains(needle));
        if offline {
            eprintln!("skipping: crates.io unreachable\n{stderr}");
            return;
        }
        panic!("`tinys run examples/json_user.sn` failed:\n{stderr}");
    }
    let manifest = std::fs::read_to_string("examples/target/tinys-generated/json_user/Cargo.toml")
        .expect("generated Cargo.toml");
    assert!(manifest.contains("serde_json"));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "Ada is active",
        "stderr:\n{stderr}"
    );
}
