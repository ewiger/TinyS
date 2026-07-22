//! The `tinys` command-line interface.
//!
//! ```text
//! tinys emit-rust <file.sn>   print the generated Rust
//! tinys build     <file.sn>   generate Rust and compile to a native binary
//! tinys run       <file.sn>   build and run
//! tinys check     <file.sn>   lex + parse (and type-check) only
//! ```
//!
//! `build`, `run` and `check` wrap the generated Rust in a scratch Cargo package
//! under `target/tinys-generated/` and drive `cargo`, so programs can depend on
//! crates declared in `tinys.toml`.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use tinys::manifest::{references_crate, Manifest};

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
                eprintln!(
                    "error: `{}` requires a .sn file argument\n\n{}",
                    command, USAGE
                );
                ExitCode::FAILURE
            }
        },
        other => {
            eprintln!("error: unknown command `{}`\n\n{}", other, USAGE);
            ExitCode::FAILURE
        }
    }
}

const USAGE: &str = concat!(
    "TinyS ",
    env!("CARGO_PKG_VERSION"),
    " — Python-shaped syntax, Rust semantics.

USAGE:
    tinys <command> <file.sn> [--release] [-- <program args>]

COMMANDS:
    emit-rust <file.sn>   Print the generated Rust source
    build     <file.sn>   Generate Rust and compile a native binary
    run       <file.sn>   Build and run the program
    check     <file.sn>   Parse and type-check without producing a binary
    version               Print the compiler version
    help                  Show this message

OPTIONS:
    --release             Build with Cargo's release profile

Cargo dependencies are read from the nearest `tinys.toml` above the source file."
);

fn run_command(command: &str, file: &str, rest: &[String]) -> ExitCode {
    let (release, prog_args) = match split_args(rest) {
        Ok(split) => split,
        Err(flag) => {
            eprintln!("error: unknown option `{}`\n\n{}", flag, USAGE);
            return ExitCode::FAILURE;
        }
    };

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

    if command == "emit-rust" {
        print!("{}", rust);
        return ExitCode::SUCCESS;
    }

    let project = match Project::prepare(&rust, file, release) {
        Ok(p) => p,
        Err(code) => return code,
    };

    match command {
        "check" => match project.cargo("check") {
            Ok(()) => {
                println!("ok: `{}` parses and type-checks", file);
                ExitCode::SUCCESS
            }
            Err(code) => code,
        },
        "build" => match project.cargo("build") {
            Ok(()) => {
                println!("compiled `{}` → {}", file, project.binary().display());
                ExitCode::SUCCESS
            }
            Err(code) => code,
        },
        "run" => match project.cargo("build") {
            Ok(()) => run_binary(&project.binary(), prog_args),
            Err(code) => code,
        },
        _ => unreachable!(),
    }
}

/// Split trailing CLI arguments into compiler flags and program arguments.
///
/// Flags come first; everything after them (or after a `--` separator) is passed
/// to the compiled program by `tinys run`.
fn split_args(rest: &[String]) -> Result<(bool, &[String]), String> {
    let mut release = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--release" => release = true,
            "--" => {
                i += 1;
                break;
            }
            other if other.starts_with("--") => return Err(other.to_string()),
            _ => break,
        }
        i += 1;
    }
    Ok((release, &rest[i..]))
}

/// The scratch Cargo package that backs one `.sn` file.
struct Project {
    /// `<root>/target/tinys-generated/<stem>/`, holding `Cargo.toml` + `src/main.rs`.
    dir: PathBuf,
    /// Cargo's build directory, shared by every program in the package so that
    /// dependencies are compiled once.
    target_dir: PathBuf,
    bin_name: String,
    release: bool,
    /// The `.sn` file as the user spelled it, for diagnostics.
    source: String,
}

impl Project {
    /// Materialise the generated Rust as a Cargo package next to its source.
    fn prepare(rust: &str, file: &str, release: bool) -> Result<Project, ExitCode> {
        let source = Path::new(file);
        let manifest = match Manifest::discover(source) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("error: {}", e);
                return Err(ExitCode::FAILURE);
            }
        };
        if let Some(m) = &manifest {
            for section in &m.ignored {
                eprintln!(
                    "warning: `{}` ignores unsupported section `[{}]`",
                    m.path.display(),
                    section
                );
            }
        }

        // The package root is wherever `tinys.toml` lives, so generated output
        // lands in one `target/` for the whole package.
        let root = match &manifest {
            Some(m) => m.dir.clone(),
            None => source
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."))
                .to_path_buf(),
        };
        let out = root.join("target").join("tinys-generated");

        let stem = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("out")
            .to_string();
        // `src/main.sn` is the package's entry point, so it takes the package name.
        let bin_name = match (
            &stem[..],
            manifest.as_ref().and_then(|m| m.package.name.clone()),
        ) {
            ("main", Some(pkg)) => crate_name(&pkg),
            _ => crate_name(&stem),
        };

        let project = Project {
            dir: out.join(&stem),
            target_dir: out.join("cargo-target"),
            bin_name,
            release,
            source: file.to_string(),
        };

        let cargo_toml = project.cargo_toml(rust, manifest.as_ref());
        write_if_changed(&project.dir.join("Cargo.toml"), &cargo_toml)?;
        write_if_changed(&project.dir.join("src").join("main.rs"), rust)?;
        Ok(project)
    }

    fn cargo_toml(&self, rust: &str, manifest: Option<&Manifest>) -> String {
        let version = manifest
            .and_then(|m| m.package.version.clone())
            .unwrap_or_else(|| "0.0.0".to_string());
        let edition = manifest
            .and_then(|m| m.package.edition.clone())
            .unwrap_or_else(|| "2021".to_string());

        let mut toml = format!(
            "# Generated by tinys — this file is rewritten on every build.\n\
             [package]\n\
             name = \"{name}\"\n\
             version = \"{version}\"\n\
             edition = \"{edition}\"\n\
             \n\
             [[bin]]\n\
             name = \"{name}\"\n\
             path = \"src/main.rs\"\n\
             \n\
             # Stand alone rather than joining a surrounding Cargo workspace.\n\
             [workspace]\n",
            name = self.bin_name,
            version = version,
            edition = edition,
        );
        if let Some(m) = manifest {
            toml.push_str(&m.cargo_sections(|dep| references_crate(rust, dep)));
        }
        toml
    }

    fn profile_dir(&self) -> &'static str {
        if self.release {
            "release"
        } else {
            "debug"
        }
    }

    fn binary(&self) -> PathBuf {
        let name = if cfg!(windows) {
            format!("{}.exe", self.bin_name)
        } else {
            self.bin_name.clone()
        };
        self.target_dir.join(self.profile_dir()).join(name)
    }

    fn cargo(&self, subcommand: &str) -> Result<(), ExitCode> {
        ensure_cargo()?;
        let mut cmd = Command::new("cargo");
        cmd.arg(subcommand)
            .arg("--quiet")
            .arg("--manifest-path")
            .arg(self.dir.join("Cargo.toml"))
            .env("CARGO_TARGET_DIR", &self.target_dir)
            // Do not inherit the jobserver of a `cargo` that spawned us.
            .env_remove("CARGO_MAKEFLAGS");
        if self.release {
            cmd.arg("--release");
        }

        match cmd.status() {
            Ok(status) if status.success() => Ok(()),
            Ok(_) => {
                // Cargo reports errors against the scratch package, so spell out
                // where its `src/main.rs` actually lives.
                eprintln!(
                    "error: `cargo {}` failed for `{}`\n       \
                     (`src/main.rs` above is {})",
                    subcommand,
                    self.source,
                    self.dir.join("src").join("main.rs").display()
                );
                Err(ExitCode::FAILURE)
            }
            Err(e) => {
                eprintln!("error: failed to invoke cargo: {}", e);
                Err(ExitCode::FAILURE)
            }
        }
    }
}

/// Turn a file or package name into a valid Cargo crate name.
fn crate_name(raw: &str) -> String {
    let mut name: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if name.is_empty() {
        name.push_str("program");
    } else if name.starts_with(|c: char| c.is_ascii_digit()) {
        name.insert(0, '_');
    }
    name
}

/// Write only when the contents differ, so Cargo does not rebuild on every run.
fn write_if_changed(path: &Path, contents: &str) -> Result<(), ExitCode> {
    if let Ok(existing) = std::fs::read_to_string(path) {
        if existing == contents {
            return Ok(());
        }
    }
    if let Some(parent) = path.parent() {
        // v0.1.0 wrote the binary itself to `target/tinys-generated/<stem>`,
        // which is now the package directory; drop such leftovers.
        if parent.exists() && !parent.is_dir() {
            let _ = std::fs::remove_file(parent);
        }
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("error: cannot create `{}`: {}", parent.display(), e);
            return Err(ExitCode::FAILURE);
        }
    }
    if let Err(e) = std::fs::write(path, contents) {
        eprintln!("error: cannot write `{}`: {}", path.display(), e);
        return Err(ExitCode::FAILURE);
    }
    Ok(())
}

fn ensure_cargo() -> Result<(), ExitCode> {
    match Command::new("cargo").arg("--version").output() {
        Ok(o) if o.status.success() => Ok(()),
        _ => {
            eprintln!("error: `cargo` was not found on PATH; install Rust from https://rustup.rs");
            Err(ExitCode::FAILURE)
        }
    }
}

fn run_binary(bin: &Path, prog_args: &[String]) -> ExitCode {
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
