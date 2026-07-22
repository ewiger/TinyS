//! End-to-end tests: drive the `tinys` binary to compile the `.sn` examples with
//! `rustc` and check that the runnable ones produce the expected output.
//!
//! These are skipped automatically when `rustc` is unavailable.

use std::process::Command;

fn rustc_available() -> bool {
    Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn tinys() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tinys"))
}

/// Examples that are pure-std and therefore compile and run with `rustc` alone.
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
];

#[test]
fn all_runnable_examples_typecheck() {
    if !rustc_available() {
        eprintln!("skipping: rustc not available");
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
    if !rustc_available() {
        return;
    }
    assert_eq!(run_example("hello").trim(), "Hello from TinyS");
}

#[test]
fn functions_compute_expected_values() {
    if !rustc_available() {
        return;
    }
    let out = run_example("functions");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["7", "25", "20"]);
}

#[test]
fn control_flow_output() {
    if !rustc_available() {
        return;
    }
    let out = run_example("control_flow");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["15", "3", "2", "1", "negative", "zero", "positive"]);
}

#[test]
fn fizzbuzz_output() {
    if !rustc_available() {
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
    if !rustc_available() {
        return;
    }
    let out = run_example("references");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines, vec!["2", "10"]);
}

#[test]
fn emit_rust_showcase_smoke() {
    // The interop example is emit-only; just make sure it generates.
    let out = tinys()
        .args(["emit-rust", "examples/json_user.sn"])
        .output()
        .expect("failed to spawn tinys");
    assert!(out.status.success());
    let rust = String::from_utf8_lossy(&out.stdout);
    assert!(rust.contains("serde_json::from_str::<User>"));
    assert!(rust.contains("fn main() -> Result<(), serde_json::Error>"));
}
