# Rust interoperability

TinyS compiles *through* Rust, so the entire Rust ecosystem is reachable. A core
design principle is that interop stays **visible**: crate boundaries, generic
calls, and macros never hide behind TinyS-looking sugar.

## The `rust` import root

Anything from a Rust crate comes through the explicit `rust` root:

```python
from rust.regex import Regex
from rust.serde import Serialize, Deserialize
from rust.std.collections import HashMap

import rust.serde_json as json
```

```rust
use regex::Regex;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
```

## Generic calls emit the turbofish

A generic call into Rust uses square brackets and emits Rust's `::<...>`:

```python
user = json.from_str[User](source)?
```

```rust
let user = serde_json::from_str::<User>(source)?;
```

## Attributes and derives pass through

Because attributes are already concise, TinyS keeps them unchanged — including
derives that come from crates:

```python
from rust.serde import Serialize, Deserialize

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User:
    id: u64
    name: str
```

## A complete interop example

This mirrors the flagship example in the repository
([`examples/json_user.sn`](https://github.com/ewiger/TinyS/blob/main/examples/json_user.sn)):

```python
from macro import debug, format
import rust.serde_json as json

#[derive(Debug)]
struct User:
    id: u64
    name: str
    active: bool

def parse_user(source: ref str) -> Result[User, json.Error]:
    return json.from_str[User](source)

def describe(user: ref User) -> str:
    return if user.active:
        format("{} is active", user.name)
    else:
        format("{} is inactive", user.name)

def main() -> Result[void, json.Error]:
    source = r#"{"id":1,"name":"Ada","active":true}"#
    user = parse_user(source)?
    debug(ref user)
    print(describe(ref user))
    return Ok()
```

Inspect the generated Rust:

```bash
tinys emit-rust examples/json_user.sn
```

```rust
use serde_json;

#[derive(Debug)]
struct User {
    id: u64,
    name: String,
    active: bool,
}

fn parse_user(source: &str) -> Result<User, serde_json::Error> {
    serde_json::from_str::<User>(source)
}
// ...
```

!!! warning "Interop is emit-only in v0.1.0"

    Programs that depend on external crates generate correct Rust, but the
    single-file `tinys build` path shells out to `rustc` and does **not** yet
    resolve crate dependencies. Cargo-backed builds driven by `tinys.toml` are on
    the [roadmap](../about/roadmap.md); until then, use `tinys emit-rust` and
    build the output in a Cargo project. Pure-std programs
    ([the runnable examples](../examples/index.md)) build and run directly.

## Where the boundary sits

Wherever possible, Rust remains the source of truth for ownership checking, borrow
checking, trait resolution, monomorphization, code generation, and dependency
linking. TinyS is a readable surface; it does not try to reimplement Rust's
semantic core.

## Where to go next

- [Macros](macros.md)
- [Modules & imports](../guide/modules.md)
- [Reading the generated Rust](generated-rust.md)
