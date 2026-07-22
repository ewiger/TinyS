# Roadmap

TinyS development is organized into three phases. For the concrete
compiles-today status, see [Language status](status.md).

## Phase 1 — language core

- indentation-aware parser;
- functions and explicit types;
- immutable and mutable variables;
- structs and enums;
- `if`, `match`, `while`, `for`, and `loop`;
- ownership, moves, references, and dereferencing;
- basic generics;
- Rust source generation;
- Cargo-backed builds.

## Phase 2 — practical language support

- traits and implementations;
- lifetime syntax;
- collections and literals;
- closures;
- module discovery;
- public APIs and re-exports;

- async functions;
- source-mapped compiler diagnostics;
- formatter and test runner.

## Phase 3 — deeper Rust interoperability

- advanced trait bounds;
- associated types;
- trait objects;
- smart pointers;
- workspaces and multiple targets;
- foreign-function interfaces;
- raw pointers;
- unsafe implementations;
- procedural and user-defined macro integration.

## Design principle

Wherever possible, **Rust remains the source of truth** for ownership checking,
borrow checking, trait resolution, monomorphization, native code generation,
platform support, and dependency linking. The compiler avoids semantic
transformations that would silently differ from Rust.
