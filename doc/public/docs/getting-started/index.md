# Getting started

Three short steps take you from an empty machine to a running TinyS program:

<div class="grid cards" markdown>

-   :material-download: **1. Install**

    ---

    Get the Rust toolchain and build the `tinys` compiler from source.

    [:octicons-arrow-right-24: Installation](installation.md)

-   :material-hand-wave: **2. Hello, world**

    ---

    Write, run, and inspect your first `.sn` program.

    [:octicons-arrow-right-24: Hello, world](hello-world.md)

-   :material-map: **3. Tutorial**

    ---

    A guided tour that builds a small program feature by feature.

    [:octicons-arrow-right-24: Tutorial](tutorial.md)

</div>

## The 60-second mental model

TinyS is **Python's look with Rust's guarantees**:

- **Indentation defines blocks** — no braces, no semicolons.
- **Static types, explicit signatures** — `def add(a: i64, b: i64) -> i64:`.
- **Immutable by default** — opt into mutation with `mut`.
- **Ownership is real** — borrow with `ref` / `mut ref`, dereference with `at`,
  transfer with `move`. There is no garbage collector.
- **Errors are values** — `Result` / `Option` and the `?` operator, never
  exceptions, never `null`.
- **Everything compiles to Rust** — and then to a native binary via `rustc`.

If you already know a little Rust, TinyS will feel like Rust wearing Python's
clothes. If you know Python, you get a readable on-ramp to Rust's ownership
model. Read the [TL;DR](../reference/cheatsheet.md) side-by-side table any time
you want the quick mapping.
