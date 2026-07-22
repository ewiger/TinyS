# 0001 — Early limitations of the v0.1.0 compiler

- **Status:** open
- **Opened:** 2026-07-22
- **Component:** compiler (`src/`), CLI (`src/main.rs`)
- **Applies to:** TinyS v0.1.0

This tracks the known limitations, shortcuts, and heuristics in the first working
compiler. None of these block the runnable [examples/](../../../examples/), but
each is a place where TinyS currently diverges from the full design in the
[README](../../../README.md) or leans on a heuristic that a later phase should
replace with real analysis.

Issues are grouped by area and given stable IDs (`L-NN`) so they can be split
into their own files as they are picked up.

---

## Toolchain & build

### L-01 — Cargo / `tinys.toml` integration — **resolved**
`tinys build`/`run`/`check` now generate a scratch Cargo package per `.sn` file
(`src/main.rs::Project`) from the nearest `tinys.toml` (`src/manifest.rs`) and
drive `cargo`. [examples/json_user.sn](../../../examples/json_user.sn) builds and
runs against `serde`/`serde_json`.

Remaining rough edges:

- only `[package]` (`name`, `version`, `edition`), `[dependencies]`,
  `[profile.*]` and `[patch.*]` are understood; other sections warn and are
  dropped (notably `[dev-dependencies]`, pending `tinys test`);
- a dependency reaches the generated `Cargo.toml` only if its crate identifier
  appears in the generated Rust, which is a textual test, not resolution;
- one Cargo package per file: there is no shared library target between the
  programs in a package (see L-02).

### L-02 — Multi-file modules / module discovery — **resolved**
`src/*.sn` now maps to the crate's module tree (`src/modules.rs`). The compiler
walks `src/`, derives modules from files, and writes the `mod` declarations
itself — TinyS has no `mod` keyword. `*_test.sn` files are declared
`#[cfg(test)]`, and `[package] exclude` keeps files out of the tree. See
[examples/modules/](../../../examples/modules/).

Remaining rough edges:

- one entry point per build: each `.sn` file generates its own Cargo package
  with a `main.rs`. Library targets (`lib.rs`), `export`/`pub use` re-exports and
  multiple declared binaries are not implemented;
- imports are absolute from the crate root only — no `from . import x`;
- a module's `pub` items are visible crate-wide because synthesized declarations
  are always `pub mod`; `pub[crate]` on items is the only narrowing available.

### L-03 — Missing CLI subcommands
`tinys fmt` and `tinys test` are documented but not implemented. `check` runs
`cargo check`; it does not yet do TinyS-level semantic checks.

---

## Diagnostics

### L-04 — Only lex/parse errors are mapped to `.sn` locations
`TinysError` reports `file:line:col` for lexer and parser failures, but **type
and borrow errors come straight from `rustc`**, pointing into the generated
`target/tinys-generated/*.rs`, not the original `.sn`. The design's diagnostic
remapping (README "Compiler diagnostics") is not implemented.

- **Impact:** a type error currently reads like `--> src/main.rs:29:5`, relative
  to the generated Cargo package; the CLI prints that package's absolute path
  underneath so the file can at least be found.
- **Toward a fix:** emit `#[line]`-style source maps or track spans through
  codegen and rewrite `rustc --error-format=json` output.

---

## Semantics resolved by heuristic (no real name/type resolution)

### L-05 — `let` vs reassignment is decided by flat per-function scope tracking
`name = expr` becomes `let name = …` on first sight of `name` and a plain
reassignment afterward (`src/codegen.rs`, `Stmt::Let`). The scope set is flat per
function, not per block.

- **Consequences:**
  - Rust-style **shadowing is unsupported** — a second `x = …` on an immutable
    `x` generates `x = …;`, which `rustc` rejects (use `mut` or a new name).
  - Block-local rebinding of an outer name is treated as assignment to the outer
    binding, not a new inner `let`.

### L-06 — Owned-string coercion (`str` → `String`) is context-limited
`.to_string()` is only inserted where an owned-string type is statically known:
typed `let`, struct-literal fields, `return`, and positional arguments to
**locally-defined** functions (`src/codegen.rs::gen_expr_coerced`, `is_owned_string`).

- **Consequences:** a string literal passed to a *method* or an imported function
  whose signature we don't track is emitted as `&str`; if `String` is required
  the generated Rust won't compile. String-literal inference is still an open
  design question in the README.

### L-07 — `::` vs `.` member access is a naming heuristic
`gen_place` uses `::` when the receiver is uppercase-initial, a known import
alias, or the member is uppercase; otherwise `.` (`src/codegen.rs`).

- **Consequences:** a lowercase *type* alias or an uppercase *value* binding will
  be routed the wrong way. Works for idiomatic code (Types PascalCase, values
  snake_case) but is not real resolution.

### L-08 — `debug(x)` passes by value, unlike the README
`debug(x)` → `dbg!(x)` (move); to borrow you must write `debug(ref x)` →
`dbg!(&x)`. The README's flagship example shows `debug(user)` generating
`dbg!(&user)`. The examples were written to the `debug(ref x)` convention.

### L-09 — `for x in ref coll` always adds one `&`
Shared/mut iteration emits `&coll` / `&mut coll` (`Stmt::For`, `Borrow`). If
`coll` is *already* a reference this double-borrows; iterate it without `ref`
instead. See the note in [examples/references.sn](../../../examples/references.sn).

### L-10 — Lifetime lexing is positional, not contextual
`.name` is lexed as a lifetime only after `[ , ( : -> < ref`
(`src/lexer.rs::lifetime_position`); everywhere else `.` is field access. This
separates `user.name` from `ref[.a] str`, but an unusual layout could confuse it.
No lifetime elision/consistency checking is done (left to `rustc`).

### L-11 — No TinyS type checker
There is no name resolution, exhaustiveness, or type analysis in TinyS itself;
`rustc` is the sole source of truth. Match exhaustiveness, "conditions must be
`bool`", and "no implicit truthiness" are therefore enforced only as far as the
generated Rust enforces them.

---

## Codegen edge cases

### L-12 — `print`/`format` format-string synthesis is heuristic
`gen_print_like` treats a **leading string literal** as the format string;
otherwise it synthesizes `"{} {} …"` across the args. A non-literal-first call
with a `{}`-containing string, or a literal not intended as a format string, can
produce surprising output. No escaping of `{`/`}` is performed.

### L-13 — Comments (including `///` and `//!`) are discarded
The lexer skips all `//` comments, so doc comments are **not preserved** in the
generated Rust.

### L-14 — `pub[crate]` restriction is parsed then dropped
Restricted visibility (`pub[crate]`, etc.) is accepted by the parser but emitted
as plain `pub` (`src/parser.rs::parse_item`).

### L-15 — Zero-arg `Ok()` / `Some()` are special-cased to `(())`
`Ok()`/`Some()` with no arguments become `Ok(())`/`Some(())` (`gen_call`). A
genuinely intended zero-field call would be mis-emitted, but that has no Rust
meaning anyway.

### L-16 — `dict` literal coercion needs an annotation
`{ … }` → `HashMap::from([ … ])`; keys/values are only coerced to owned types
when the binding carries a `dict[K, V]` annotation. `set` literals have no
surface syntax (only the `set[T]` type).

### L-17 — `array[T, N]` length is literal/ident only
Const-generic array lengths only render an integer literal or a bare identifier
(`gen_expr_const`); const expressions are not supported.

---

## Unimplemented language surface (design exists, codegen does not)

### L-18 — Enum payloads are positional-only
Only tuple-style variants (`Number(i64)`) are supported. Named/struct-like enum
payloads (an open README question) are not parsed.

### L-19 — Closures cannot be named as parameter types
Closure params default to `_` when a type is omitted, but there is no surface
type for `Fn`/`FnMut`/`FnOnce`, so higher-order functions can't declare a
closure parameter's type.

### L-20 — No `where` clauses, associated types, trait objects, or smart pointers
Trait bounds are inline only (`[T: Bound + Bound2]`). The Phase 3 interop
features (associated types, `dyn` trait objects, `Box`/`Rc`/`Arc` sugar, FFI,
raw pointers, `unsafe impl`) are out of scope for v0.1.0. `unsafe:` blocks and
`unsafe def` are supported minimally.

---

## Not bugs (intentional v0.1.0 decisions)

- Generated Rust starts with a broad `#![allow(...)]` so incidental warnings
  never mask real errors — see `src/codegen.rs::emit_program`.
- `move x` emits just `x` (ownership transfer is implicit in Rust); `move` on a
  closure becomes `move |…|`.
- Implicit tail returns are dropped only for value-returning functions; `void`
  functions keep every trailing expression a statement (fixed during bring-up).
