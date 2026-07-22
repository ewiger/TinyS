# Language status

TinyS is an **experimental** language design and compiler project. This page is
the single source of truth for what compiles and runs today versus what is
designed but still in progress.

!!! abstract "At a glance"

    A working **v0.1.0 compiler** exists — an indentation-aware lexer, a
    recursive-descent parser, and a Rust source generator driven by the `tinys`
    CLI. It covers the Phase 1 core plus a good slice of Phase 2. The syntax and
    semantics are still evolving, and TinyS is **not yet ready for production
    use**.

## Runs today

These features compile and run through `tinys run` / `tinys build`, which drive
`cargo` over a generated package. Each is backed by a runnable
[example](../examples/index.md):

- [x] Functions, explicit types, early `return` and tail-expression returns
- [x] Immutable bindings and `mut` variables
- [x] `if` / `elif` / `else`, including the expression form
- [x] `while`, `for`, ranges (`a..b`, `a..=b`)
- [x] `loop` as an expression, labeled loops (`as`), `break … with`
- [x] Structs, `impl` blocks, associated functions, methods with `ref`/`mut ref` receivers
- [x] Enums (algebraic data types) and exhaustive, expression-oriented `match` / `case`
- [x] Ownership and borrowing: `ref`, `mut ref`, `at`, `move`, `clone`
- [x] Generics and trait bounds (`[T]`, `[T: Bound]`)
- [x] Closures (`fn(...)`)
- [x] `print` / `format` / `debug` macros
- [x] `#[derive(...)]` and other attributes passing through to Rust
- [x] Cargo-backed builds driven by `tinys.toml` (dependency resolution)
- [x] Rust-crate interop through the `rust` / `macro` roots (e.g. `serde` / `serde_json`) — see [`json_user.sn`](../examples/index.md#rust-interop)

## Designed — in progress

Specified in the language reference and used throughout this manual, but not yet
fully wired into the compiler. Verify with `tinys emit-rust` before relying on a
specific form:

- [ ] Multi-file module discovery
- [ ] `dict` / `set` literals and `array` types (beyond `list` literals)
- [ ] Lifetime syntax coverage across all positions
- [ ] `async def` / `.await` (depends on runtime crates → Cargo builds)
- [ ] Traits with default methods, trait objects, associated types
- [ ] Source-mapped diagnostics (`.sn` locations for `rustc` errors)
- [ ] `tinys test` and `tinys fmt` subcommands
- [ ] `unsafe`, raw pointers, and FFI

See the [roadmap](roadmap.md) for how these are organized into phases.

## Open design questions

Several areas are intentionally undecided and expected to be resolved through
implementation experiments rather than syntax design alone:

- exact string-literal inference;
- closure syntax details;
- syntax for complex `where` clauses;
- named enum payload syntax;
- standard prelude size;
- comprehension or pipeline syntax;
- module discovery rules;
- raw-pointer representation;
- degree of generated-Rust stability;
- whether `own` remains documentation-only;
- whether Python-style inline conditional expressions are supported.

## Versioning

The compiler reports its version with `tinys version` (currently `tinys 0.1.0`).
