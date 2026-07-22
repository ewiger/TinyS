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
│   ├── main.sn
│   ├── models.sn
│   └── services/
│       ├── mod.sn
│       └── database.sn
└── target/
    └── tinys-generated/
        └── ...
```

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

TinyS understands `[package]` (`name`, `version`, `edition`), `[dependencies]`
— in both the inline and `[dependencies.<name>]` forms — and passes `[profile.*]`
and `[patch.*]` through unchanged. Other sections are ignored with a warning.

!!! info "Multi-file modules are on the roadmap"

    The v0.1.0 compiler builds **one `.sn` file at a time**. Module discovery
    across multiple `.sn` files is planned — see the
    [roadmap](../about/roadmap.md).

## Where to go next

- [Rust interoperability](../advanced/interop.md)
- [Macros](../advanced/macros.md)
