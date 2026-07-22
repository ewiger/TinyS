# Changelog

All notable changes to TinyS are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Cargo-backed builds and multi-file packages. `tinys` no longer shells out to
`rustc` for a single file; every `.sn` program is compiled as a real Cargo
package, so programs can depend on crates from [crates.io](https://crates.io)
and can be split across files.

### Added

- **Multi-file modules, with no `mod` keyword** (`src/modules.rs`). The compiler
  walks `src/`, derives the module tree from the file tree, and writes Rust's
  `mod` declarations itself — a file is a module because it exists. `src/` is the
  switch: a package without one is a directory of independent single-file
  programs. The file you build is the crate root and is never also a module, so
  `main.sn` is not required. A directory is a module whose own source is
  `mod.sn`; one without `mod.sn` still exists to hold its children. Imports stay
  absolute from the crate root.
- **Colocated tests through the `_test.sn` suffix.** Such a file is declared
  `#[cfg(test)]`, giving Rust's colocated `mod tests` without a keyword. `tinys
  check` now passes `--all-targets` so these are type-checked too.
- **`[package] exclude`** in `tinys.toml`, keeping files out of the module tree.
  `*` matches within one path segment; naming a directory excludes everything
  under it.
- **`examples/modules/`**, a multi-file package exercising directory modules,
  a `_test.sn` module, and `exclude` (with a deliberately broken file inside it).
- **Cargo-backed builds.** `build`, `run` and `check` generate a Cargo package
  per `.sn` file under `target/tinys-generated/` and drive `cargo`.
- **`tinys.toml` manifests** (`src/manifest.rs`), discovered by walking up from
  the source file the way Cargo finds `Cargo.toml`. Understands `[package]`
  (`name`, `version`, `edition`) and `[dependencies]` in both the inline and
  `[dependencies.<name>]` forms; `[profile.*]` and `[patch.*]` are passed through
  unchanged, relative `path` dependencies are resolved against the manifest, and
  unsupported sections are reported with a warning. No new compiler dependencies
  — TinyS still builds with an empty dependency tree.
- **Per-program dependency pruning.** Only crates whose identifier appears in the
  generated Rust reach the generated `Cargo.toml`, so a package can declare
  `serde` without slowing down — or requiring a network for — programs that never
  import it.
- **`--release`**, forwarding Cargo's release profile.
- `examples/tinys.toml`, declaring the `serde` and `serde_json` crates used by the
  interop example.
- `tests/cargo_build.rs`: manifest discovery, package root and binary naming, and
  the unsupported-section warning. All dependency-free, so they run offline.

### Changed

- `tinys check` now runs `cargo check` instead of `rustc --emit=metadata`, so it
  type-checks against real dependencies.
- `build`, `run` and `check` require `cargo` on `PATH` rather than `rustc`.
- Output layout: each program becomes
  `target/tinys-generated/<name>/{Cargo.toml,src/main.rs}`, with one
  `target/tinys-generated/cargo-target/` shared across the package so
  dependencies compile once. Generated output is rooted next to `tinys.toml` when
  the source belongs to a package.
- `examples/json_user.sn` builds and runs. It was emit-only, and as written would
  not have compiled even with crates wired up — `User` now derives `Deserialize`
  from `from rust.serde import Deserialize`.
- Generated files are rewritten only when their contents change, so an unchanged
  `tinys run` no longer triggers a Cargo rebuild.
- A failing build reports the absolute path of the generated `src/main.rs` that
  Cargo's errors refer to.
- `tinys emit-rust` labels each generated file when a program spans more than
  one; single-file output is unchanged.
- Generated `.rs` files left behind by a renamed or deleted module are pruned, so
  a stale `foo.rs` can never collide with a new `foo/mod.rs`.
- Documentation throughout the manual, the README, `doc/draft/syntax-general.md`
  and `doc/issues/` no longer describes crate interop as emit-only, Cargo builds
  as planned, or `mod` as an open design question — §22 of the draft now records
  the decision that TinyS has no `mod` keyword, not even as an optional form.

### Fixed

- `tinys help` printed a hardcoded `TinyS 0.1.0` while `--version` read the crate
  version; both now use `CARGO_PKG_VERSION`.

### Known limitations

- Dependency pruning is a textual identifier match against the generated Rust,
  not name resolution.
- `[dev-dependencies]` and other manifest sections are dropped with a warning.
- One entry point per build: each `.sn` file becomes its own Cargo package with
  a `main.rs`. Library targets (`lib.rs`), `export`/`pub use` re-exports and
  multiple declared binaries are not implemented.
- Imports are absolute from the crate root; there is no `from . import x`.
- Synthesized declarations are always `pub mod`, so a module's `pub` items are
  visible crate-wide. See `doc/issues/open/0001-early-limitations.md`.
- `rustc` type and borrow errors still point into the generated Rust rather than
  the `.sn` source.

## [0.1.0] — 2026-07-22

First published release: an indentation-aware lexer, a recursive-descent parser,
and a Rust source generator behind the `tinys` CLI (`build`, `run`, `check`,
`emit-rust`, `version`, `help`), compiling single-file programs with `rustc`.

Covers the Phase 1 language core — functions and explicit types, `mut` bindings,
structs, enums and `match`, `if`/`while`/`for`/`loop` including their expression
forms, references and ownership (`ref`, `mut ref`, `at`, `move`, `clone`),
generics with trait bounds, closures, the `print`/`format`/`debug` macros, and
attribute passthrough — plus the MkDocs language manual and eleven runnable
examples.

[0.1.0]: https://crates.io/crates/tinys/0.1.0
