# CLI commands

The `tinys` binary drives the whole pipeline: `.sn` source → generated Rust →
Cargo → native binary.

```text
tinys <command> <file.sn> [--release] [-- <program args>]
```

## Commands

| Command                     | Status      | Description                                       |
| --------------------------- | ----------- | ------------------------------------------------- |
| `tinys build <file.sn>`     | implemented | Generate Rust and compile a native binary         |
| `tinys run <file.sn>`       | implemented | Build and run an application                      |
| `tinys check <file.sn>`     | implemented | Parse and type-check via `cargo check` (no binary) |
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

Generates Rust into `target/tinys-generated/`, wraps it in a Cargo package whose
dependencies come from [`tinys.toml`](../guide/modules.md#dependencies), and runs
`cargo build`. Prints the resulting binary path on success. Pass `--release` to
build with Cargo's release profile.

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

Parses the `.sn` file and type-checks the generated Rust with `cargo check` — no
binary is produced. Reports `ok: … parses and type-checks` on success.

## `emit-rust`

```bash
tinys emit-rust examples/json_user.sn
```

Prints the generated Rust to standard output without invoking Cargo. This is the
only command that works without a Rust toolchain installed. See
[Reading the generated Rust](../advanced/generated-rust.md).

## Output layout

Each `.sn` file becomes its own Cargo package under `target/tinys-generated/`,
written next to `tinys.toml` if there is one, and otherwise next to the source
file:

```text
target/
└── tinys-generated/
    ├── hello/
    │   ├── Cargo.toml      # generated from tinys.toml
    │   └── src/main.rs     # generated Rust
    └── cargo-target/       # Cargo's build directory, shared by the package
        └── debug/hello     # compiled binary
```

Only the crates a program imports are copied into the generated `Cargo.toml`, so
a `tinys.toml` that declares `serde` costs nothing for programs that do not use
it.

## Requirements

`build`, `run`, and `check` require `cargo` on your `PATH`. If it is missing,
`tinys` prints a clear error pointing at <https://rustup.rs>. See
[Installation](../getting-started/installation.md).

## Driving through Cargo

Before installing the binary, you can run the compiler from a clone with Cargo —
`--` separates Cargo's arguments from `tinys`'s:

```bash
cargo run -- run examples/hello.sn
cargo run -- emit-rust examples/hello.sn
```
