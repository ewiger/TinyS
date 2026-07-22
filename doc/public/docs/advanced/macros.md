# Macros

Rust macros are available in TinyS, but with two differences: they are **imported
explicitly**, and they are **called without** the trailing `!`.

## Importing macros

Macros come through the `macro` root:

```python
from macro import assert, debug, format
from macro.std import vec
```

`macro` and `macro.std` name the prelude and std macros, which are callable
unqualified. The `macro` root is routing only — it never appears in the
generated Rust, and no `use` line is emitted for it.

Crate-specific macros use the crate's namespace under `macro`:

```python
from macro.serde_json import json
from macro.regex import regex
```

Any root other than `std` is read as a crate namespace, so the generated call is
path-qualified with it:

```rust
serde_json::json!(...)
regex::regex!(...)
```

That keeps the invocation resolvable without an extra `use`, and lets two crates
export same-named macros in one file.

!!! note

    A small prelude — `print`, `format`, `debug`, `assert`, `assert_eq`,
    `panic`, `vec` — is in scope without an import, which is why
    [`hello.sn`](../examples/index.md#hello-world) can call `print` directly.
    Importing them explicitly is still supported, and is what makes aliases and
    crate macros possible.

## Calling macros

No exclamation mark at the call site:

```python
debug(user)
assert(user.id > 0)

values = vec(1, 2, 3)
message = format("Hello {}", user.name)
```

These generate the corresponding Rust macro invocations:

```rust
dbg!(user);
assert!(user.id > 0);
vec![1, 2, 3];
format!("Hello {}", user.name);
```

## Aliases

The alias becomes the call-site name; the Rust macro behind it is unchanged.

```python
from macro import debug as dbg
from macro import assert as require
```

```python
require(user.id > 0)     # assert!(user.id > 0)
```

## Common macros at a glance

| TinyS call            | Rust macro          |
| --------------------- | ------------------- |
| `print(...)`          | `println!(...)`     |
| `format(...)`         | `format!(...)`      |
| `debug(x)`            | `dbg!(x)`           |
| `assert(cond)`        | `assert!(cond)`     |
| `vec(a, b, c)`        | `vec![a, b, c]`     |

## User-defined macros

Custom, user-defined TinyS macros are outside the initial core language scope;
procedural and user-defined macro integration is part of the deeper
interoperability phase. See the [roadmap](../about/roadmap.md).

## Where to go next

- [Rust interoperability](interop.md)
- [Modules & imports](../guide/modules.md)
