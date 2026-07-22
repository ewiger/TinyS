# Reading the generated Rust

TinyS never hides the Rust it produces. The generated source is meant to be
**readable, deterministic, and inspectable** — you can always see exactly what
your `.sn` program becomes.

## Inspect it with `emit-rust`

```bash
tinys emit-rust examples/structs.sn
```

The command prints the generated Rust to standard output. `tinys build` and
`tinys run` also write it to disk under `target/tinys-generated/` next to your
source, alongside the compiled binary.

## What the output looks like

```python
#[derive(Debug, Clone)]
struct Point:
    x: f64
    y: f64

impl Point:
    def length(self: ref Self) -> f64:
        squared = self.x * self.x + self.y * self.y
        return squared.sqrt()
```

becomes:

```rust
#[derive(Debug, Clone)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn length(&self) -> f64 {
        let squared = self.x * self.x + self.y * self.y;
        return squared.sqrt();
    }
}
```

## Design goals for the output

The generated Rust is intended to be:

- **deterministic** — the same input always produces the same output;
- **readable** — recognizably close to hand-written Rust;
- **inspectable** — you can review it before trusting it;
- **compatible** with standard Rust tooling;
- **mapped back** to the original `.sn` source for diagnostics.

## Diagnostics map back to `.sn`

A core requirement is that Rust diagnostics point at your TinyS source, not at
generated line numbers:

```text
error: value `data` was moved here
  --> src/main.sn:14:13
```

You should not have to debug generated `.rs` files by hand.

!!! info "Diagnostic remapping is maturing"

    Source-mapped diagnostics are a Phase 2 goal. In the current compiler, if
    `rustc` rejects the generated Rust, `tinys` points you at `emit-rust` so you
    can inspect the output directly. See [Language status](../about/status.md).

## The pipeline

```text
source.sn
    ↓  lexer (indentation-aware)
tokens
    ↓  recursive-descent parser
TinyS AST
    ↓  Rust source generator
generated .rs
    ↓  rustc / Cargo
native binary
```

## Where to go next

- [CLI commands](../reference/cli.md)
- [Rust interoperability](interop.md)
