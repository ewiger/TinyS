# Installation

TinyS transpiles to Rust and then shells out to `rustc`, so you need the Rust
toolchain installed, plus the TinyS compiler itself (built from source).

## Prerequisites

TinyS generates Rust and compiles it with `rustc`, so a working Rust toolchain
must be on your `PATH`.

=== "macOS / Linux"

    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

=== "Windows"

    Download and run [`rustup-init.exe`](https://rustup.rs) and follow the
    prompts.

Verify it:

```bash
rustc --version
cargo --version
```

!!! warning "`rustc` is required at compile time"

    If `rustc` is not found on your `PATH`, `tinys build`, `tinys run`, and
    `tinys check` will fail with a clear error. Only `tinys emit-rust` (which
    just prints generated Rust) works without it.

## Build the `tinys` compiler

Clone the repository and build the compiler with Cargo:

```bash
git clone https://github.com/ewiger/TinyS.git
cd TinyS
cargo build --release
```

The compiler binary lands at `target/release/tinys`. You can run it directly, or
put it on your `PATH`:

```bash
./target/release/tinys --version        # tinys 0.1.0

# Optional: make `tinys` available everywhere
cargo install --path .                   # installs into ~/.cargo/bin
tinys --version
```

During development you can also skip the install and drive the compiler through
Cargo:

```bash
cargo run -- run examples/hello.sn
```

The `--` separates Cargo's own arguments from the arguments passed to `tinys`.

## Verify your setup

Run the bundled examples' test suite — it drives the `tinys` binary end to end:

```bash
cargo test
```

The end-to-end tests are skipped automatically when `rustc` is unavailable, so a
green run confirms both halves of the toolchain are wired up.

## What's next

- Build and run [Hello, world](hello-world.md).
- Take the [guided tutorial](tutorial.md).
- Skim the [CLI reference](../reference/cli.md) for every command.
