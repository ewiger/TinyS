# Hello, world

The canonical first program. Create a file called `hello.sn`:

```python
# hello.sn
def main() -> void:
    print("Hello from TinyS")
```

`main` is the program entry point. Its return type is `void` (the equivalent of
Rust's unit type `()`), and `print` writes a line to standard output.

## Run it

```bash
tinys run hello.sn
```

```text
Hello from TinyS
```

`tinys run` transpiles `hello.sn` to Rust, compiles it with Cargo, and runs the
resulting binary in one step.

!!! tip "Running from a fresh clone"

    If you have not installed the `tinys` binary yet, drive it through Cargo from
    the repository root:

    ```bash
    cargo run -- run examples/hello.sn
    ```

## Look under the hood

TinyS never hides the Rust it produces. Ask for the generated source with
`emit-rust`:

```bash
tinys emit-rust hello.sn
```

```rust
fn main() {
    println!("Hello from TinyS");
}
```

That is the whole idea of TinyS: a readable, Python-shaped surface over
predictable, inspectable Rust. Read more in
[Reading the generated Rust](../advanced/generated-rust.md).

## The commands you'll use most

| Command                     | What it does                                            |
| --------------------------- | ------------------------------------------------------- |
| `tinys run hello.sn`        | Transpile → compile → run                               |
| `tinys build hello.sn`      | Transpile → compile to a native binary (no run)         |
| `tinys check hello.sn`      | Parse and type-check only (via `cargo check`), no binary |
| `tinys emit-rust hello.sn`  | Print the generated Rust                                |

Generated Rust and binaries are written under `target/tinys-generated/` next to
your source — or next to `tinys.toml` when the file belongs to a package. See the
full [CLI reference](../reference/cli.md).

## Pass arguments to your program

Anything after `--` is forwarded to the compiled program, not to `tinys`:

```bash
tinys run hello.sn -- --name Ada
```

## Where to go next

You have a working toolchain and a running program. The
[guided tutorial](tutorial.md) builds up a small but real program — functions,
structs, enums, pattern matching, and error handling — one concept at a time.
