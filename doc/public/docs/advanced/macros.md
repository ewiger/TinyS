# Macros

Rust macros are available in TinyS, but with two differences: they are **imported
explicitly**, and they are **called without** the trailing `!`.

## Importing macros

Macros come through the `macro` root:

```python
from macro import assert, debug, format
from macro.std import vec
```

Crate-specific macros use the crate's namespace under `macro`:

```python
from macro.serde_json import json
from macro.regex import regex
```

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

```python
from macro import debug as dbg
from macro import assert as require
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
