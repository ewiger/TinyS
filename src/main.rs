//! The `tinys` command-line interface.
//!
//! ```text
//! tinys emit-rust <file.sn>   print the generated Rust
//! tinys build     <file.sn>   generate Rust and compile to a native binary
//! tinys run       <file.sn>   build and run
//! tinys check     <file.sn>   lex + parse (and rustc type-check) only
//! ```

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("{}", USAGE);
        return ExitCode::FAILURE;
    }

    let command = args[1].as_str();
    match command {
        "--version" | "-V" | "version" => {
            println!("tinys {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        "--help" | "-h" | "help" => {
            println!("{}", USAGE);
            ExitCode::SUCCESS
        }
        "emit-rust" | "build" | "run" | "check" => match args.get(2) {
            Some(file) => run_command(command, file, &args[3..]),
            None => {
                eprintln!("error: `{}` requires a .sn file argument\n\n{}", command, USAGE);
                ExitCode::FAILURE
            }
        },
        other => {
            eprintln!("error: unknown command `{}`\n\n{}", other, USAGE);
            ExitCode::FAILURE
        }
    }
}

const USAGE: &str = "\
TinyS 0.1.0 — Python-shaped syntax, Rust semantics.

USAGE:
    tinys <command> <file.sn> [-- <program args>]

COMMANDS:
    emit-rust <file.sn>   Print the generated Rust source
    build     <file.sn>   Generate Rust and compile a native binary
    run       <file.sn>   Build and run the program
    check     <file.sn>   Parse and type-check without producing a binary
    version               Print the compiler version
    help                  Show this message";

fn run_command(command: &str, file: &str, rest: &[String]) -> ExitCode {
    let source = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read `{}`: {}", file, e);
            return ExitCode::FAILURE;
        }
    };

    let rust = match tinys::compile_to_rust(&source, file) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::FAILURE;
        }
    };

    match command {
        "emit-rust" => {
            print!("{}", rust);
            ExitCode::SUCCESS
        }
        "check" => check(&rust, file),
        "build" => match build(&rust, file, false) {
            Ok(bin) => {
                println!("compiled `{}` → {}", file, bin.display());
                ExitCode::SUCCESS
            }
            Err(code) => code,
        },
        "run" => match build(&rust, file, false) {
            Ok(bin) => run_binary(&bin, rest),
            Err(code) => code,
        },
        _ => unreachable!(),
    }
}

/// Directory that holds generated Rust and binaries, mirroring the design's
/// `target/tinys-generated/` layout.
fn out_dir(file: &Path) -> PathBuf {
    let parent = file.parent().unwrap_or_else(|| Path::new("."));
    parent.join("target").join("tinys-generated")
}

fn stem(file: &str) -> String {
    Path::new(file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("out")
        .to_string()
}

fn write_generated(rust: &str, file: &str) -> Result<PathBuf, ExitCode> {
    let dir = out_dir(Path::new(file));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("error: cannot create `{}`: {}", dir.display(), e);
        return Err(ExitCode::FAILURE);
    }
    let rs_path = dir.join(format!("{}.rs", stem(file)));
    if let Err(e) = std::fs::write(&rs_path, rust) {
        eprintln!("error: cannot write `{}`: {}", rs_path.display(), e);
        return Err(ExitCode::FAILURE);
    }
    Ok(rs_path)
}

fn ensure_rustc() -> Result<(), ExitCode> {
    match Command::new("rustc").arg("--version").output() {
        Ok(o) if o.status.success() => Ok(()),
        _ => {
            eprintln!("error: `rustc` was not found on PATH; install Rust from https://rustup.rs");
            Err(ExitCode::FAILURE)
        }
    }
}

fn build(rust: &str, file: &str, check_only: bool) -> Result<PathBuf, ExitCode> {
    ensure_rustc()?;
    let rs_path = write_generated(rust, file)?;
    let dir = out_dir(Path::new(file));
    let bin_path = dir.join(stem(file));

    let mut cmd = Command::new("rustc");
    cmd.arg(&rs_path).arg("--edition").arg("2021");
    if check_only {
        // Keep the emitted `.rmeta` inside the generated dir, not the CWD.
        cmd.arg("--emit").arg("metadata").arg("--out-dir").arg(&dir);
    } else {
        cmd.arg("-o").arg(&bin_path);
    }

    match cmd.status() {
        Ok(status) if status.success() => Ok(bin_path),
        Ok(_) => {
            eprintln!(
                "error: rustc failed to compile the generated Rust for `{}`\n       (inspect it with `tinys emit-rust {}`)",
                file, file
            );
            Err(ExitCode::FAILURE)
        }
        Err(e) => {
            eprintln!("error: failed to invoke rustc: {}", e);
            Err(ExitCode::FAILURE)
        }
    }
}

fn check(rust: &str, file: &str) -> ExitCode {
    // Parsing already succeeded (we have `rust`); now let rustc type-check it.
    match build(rust, file, true) {
        Ok(_) => {
            println!("ok: `{}` parses and type-checks", file);
            ExitCode::SUCCESS
        }
        Err(code) => code,
    }
}

fn run_binary(bin: &Path, rest: &[String]) -> ExitCode {
    // Allow `tinys run file.sn -- arg1 arg2`.
    let prog_args: &[String] = if rest.first().map(|s| s == "--").unwrap_or(false) {
        &rest[1..]
    } else {
        rest
    };
    match Command::new(bin).args(prog_args).status() {
        Ok(status) => {
            if status.success() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(status.code().unwrap_or(1) as u8)
            }
        }
        Err(e) => {
            eprintln!("error: failed to run `{}`: {}", bin.display(), e);
            ExitCode::FAILURE
        }
    }
}
