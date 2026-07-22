# Modules & imports

TinyS keeps imports Python-shaped, while making Rust-crate boundaries explicit
through a dedicated `rust` import root. Interoperability stays **visible** by
design.

## Native TinyS modules

Native modules use Python-style imports:

```python
from models import User
import services.database as database
```

**There is no `mod` keyword.** A file is a module because it exists — the
compiler walks `src/`, derives the module tree from the file tree, and writes
Rust's `mod` declarations itself. Nothing in TinyS source ever declares a module,
and no declaration can drift out of sync with the filesystem.

```text
src/
├── app.sn              the file you build — the crate root
├── models.sn           →  crate::models
├── models_test.sn      →  crate::models_test, compiled only under `cargo test`
└── services/
    ├── mod.sn          →  crate::services
    └── store.sn        →  crate::services::store
```

Imports are absolute from the crate root, so `services/store.sn` names `models`
exactly the way `app.sn` does:

```python
from models import User
```

```rust
use crate::models::User;
```

Mutual imports between modules are fine. The whole tree is one compilation unit,
so `models` importing `services` importing `models` just works — TinyS inherits
Rust's behaviour here rather than Python's import cycles.

### Discovery rules

- **`src/` is the switch.** A package whose `tinys.toml` has no `src/` beside it
  is a directory of independent single-file programs.
- **The file you build is the crate root**, and is never also a module. `main.sn`
  is not required — any `.sn` file with a `def main()` can be the entry point.
- **A directory is a module.** Its own source is `mod.sn`; a directory without
  one still exists as a module holding its children.
- **`foo.sn` beside `foo/` is an error** — one module name with two sources.
- **Everything is declared `pub mod`.** The `pub` you write on *items* is what
  defines your public surface, so a private module would make it a lie.

### Colocated tests

A file whose name ends in `_test.sn` is declared `#[cfg(test)]`, so it is
compiled only when testing. This is how TinyS expresses Rust's colocated
`#[cfg(test)] mod tests` without a keyword:

```python
# src/models_test.sn
from models import User, is_visible

#[test]
def inactive_users_are_hidden() -> void:
    user = User.new(8, "Grace", false)
    assert(not is_visible(ref user))
```

`tinys check` type-checks these along with everything else.

### Keeping files out of the tree

Since a file joins the build simply by existing, `exclude` in `tinys.toml` is how
you keep work-in-progress out of it. `*` matches within one path segment, and
naming a directory excludes everything under it:

```toml
[package]
name = "example"
exclude = ["scratch", "*_wip.sn"]
```

### Conditional modules

Rust attaches `#[cfg(...)]` to a `mod` declaration. Since TinyS writes those
declarations for you, put the attribute at the top of the module's own file
instead:

```python
# src/platform.sn
#![cfg(unix)]

pub def open_tty() -> void:
    ...
```

## Importing from Rust crates

Rust crates and modules come through the explicit `rust` root, so it is always
obvious when you are reaching into the Rust ecosystem:

```python
from rust.regex import Regex
from rust.serde import Serialize, Deserialize
from rust.std.collections import HashMap
```

Module aliases:

```python
import rust.serde_json as json
```

Generic calls into Rust use square brackets and emit the turbofish:

```python
user = json.from_str[User](source)?
```

```rust
let user = serde_json::from_str::<User>(source)?;
```

## Importing macros

Macros are imported explicitly and called **without** the `!`:

```python
from macro import assert, debug, format
from macro.std import vec
```

```python
debug(user)
assert(user.id > 0)
values = vec(1, 2, 3)
message = format("Hello {}", user.name)
```

Aliases work too:

```python
from macro import debug as dbg
from macro import assert as require
```

Roots other than `std` are crate namespaces, and the generated call carries the
crate with it — `from macro.serde_json import json` makes `json(...)` emit
`serde_json::json!(...)`.

See [Macros](../advanced/macros.md) for the full mapping.

## Visibility

Declarations are private to their module by default. Export them with `pub`:

```python
pub struct User:
    pub id: u64
    name: str

pub def load_user(id: u64) -> Result[User, Error]:
    ...
```

More restricted visibility uses square brackets:

```python
pub[crate] def helper() -> void:
    ...
```

## Project layout

A TinyS package maps approximately to one Cargo package:

```text
example/
├── tinys.toml
├── src/
│   ├── app.sn
│   ├── models.sn
│   └── services/
│       ├── mod.sn
│       └── database.sn
└── target/
    └── tinys-generated/
        └── ...
```

See [`examples/modules/`](https://github.com/ewiger/TinyS/tree/main/examples/modules)
for a working package in exactly this shape.

Applications generate `main.rs`; libraries generate `lib.rs`.

## Dependencies

Cargo dependencies are declared in `tinys.toml`, which mirrors `Cargo.toml`:

```toml
[package]
name = "example"
version = "0.1.0"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
regex = "1"
```

`build`, `run` and `check` search upward from the source file for the nearest
`tinys.toml`, then generate the corresponding `Cargo.toml` under
`target/tinys-generated/` and drive `cargo`. Only the crates a program actually
imports are carried over, so a package can declare `regex` without slowing down
programs that never import it.

TinyS understands `[package]` (`name`, `version`, `edition`, `exclude`),
`[dependencies]` — in both the inline and `[dependencies.<name>]` forms — and
passes `[profile.*]` and `[patch.*]` through unchanged. Other sections are
ignored with a warning.

!!! info "One entry point per build"

    Each `.sn` entry point becomes its own Cargo package with a `main.rs`.
    Library targets (`lib.rs`), re-exports (`pub use`) and multiple declared
    binaries are still on the [roadmap](../about/roadmap.md).

## Where to go next

- [Rust interoperability](../advanced/interop.md)
- [Macros](../advanced/macros.md)
