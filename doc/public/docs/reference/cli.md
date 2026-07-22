# CLI commands

The `tinys` binary drives the whole pipeline: `.sn` source → generated Rust →
`rustc` → native binary.

```text
tinys <command> <file.sn> [-- <program args>]
```

## Commands

| Command                     | Status      | Description                                       |
| --------------------------- | ----------- | ------------------------------------------------- |
| `tinys build <file.sn>`     | implemented | Generate Rust and compile a native binary         |
| `tinys run <file.sn>`       | implemented | Build and run an application                      |
| `tinys check <file.sn>`     | implemented | Parse and `rustc` type-check (no binary)          |
| `tinys emit-rust <file.sn>` | implemented | Print the generated Rust for inspection           |
| `tinys version`             | implemented | Print the compiler version                        |
| `tinys help`                | implemented | Show usage                                        |
| `tinys test`                | planned     | Run tests through Cargo                           |
| `tinys fmt`                 | planned     | Format `.sn` source files                         |

`--version` / `-V` and `--help` / `-h` are accepted as aliases for `version` and
`help`.

## `build`

```bash
tinys build examples/hello.sn
```

Generates Rust into `target/tinys-generated/`, compiles it with `rustc`
(`--edition 2021`), and writes the binary next to the generated source. Prints the
resulting binary path on success.

## `run`

```bash
tinys run examples/fizzbuzz.sn
```

Builds as above, then runs the binary. Forward arguments to your program after
`--`:

```bash
tinys run examples/hello.sn -- --name Ada
```

## `check`

```bash
tinys check examples/structs.sn
```

Parses the `.sn` file and type-checks the generated Rust with `rustc` (emitting
metadata only — no binary). Reports `ok: … parses and type-checks` on success.

## `emit-rust`

```bash
tinys emit-rust examples/json_user.sn
```

Prints the generated Rust to standard output without invoking `rustc`. This is the
only command that works without a Rust toolchain installed, and the way to inspect
*emit-only* interop programs. See
[Reading the generated Rust](../advanced/generated-rust.md).

## Output layout

Generated Rust and compiled binaries are written under a `target/tinys-generated/`
directory next to your source file:

```text
target/
└── tinys-generated/
    ├── hello.rs      # generated Rust
    └── hello         # compiled binary
```

## Requirements

`build`, `run`, and `check` require `rustc` on your `PATH`. If it is missing,
`tinys` prints a clear error pointing at <https://rustup.rs>. See
[Installation](../getting-started/installation.md).

## Driving through Cargo

Before installing the binary, you can run the compiler from a clone with Cargo —
`--` separates Cargo's arguments from `tinys`'s:

```bash
cargo run -- run examples/hello.sn
cargo run -- emit-rust examples/hello.sn
```
