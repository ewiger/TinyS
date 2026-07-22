# Error handling

TinyS follows Rust's error model exactly: errors are **values**, not exceptions,
and absence is **`Option`**, not `null`.

## `Result` for recoverable errors

A function that can fail returns `Result[T, E]`:

```python
def parse(source: ref str) -> Result[User, Error]:
    return json.from_str[User](source)
```

`Result[T, E]` maps to Rust's `Result<T, E>`, with variants `Ok(value)` and
`Err(error)`.

## Propagating errors with `?`

The `?` operator returns early on `Err`, unwrapping `Ok` otherwise — the same as
Rust:

```python
user = parse(source)?
```

If `parse` returns `Err`, the enclosing function returns that error immediately;
otherwise `user` is bound to the inner value.

## `Option` for absence

Absence is represented with `Option[T]`, never `null`:

```python
user = Some(value)
user = none
```

Match on it like any enum:

```python
match user:
    case Some(value):
        print(value)

    case none:
        print("not found")
```

Or use a pattern condition:

```python
if case Some(user) = find_user(user_id):
    print(user.name)
else:
    print("not found")
```

## No exceptions

TinyS does **not** use exceptions for ordinary recoverable errors. Fallible
operations return `Result`; the caller decides whether to handle the error,
propagate it with `?`, or convert it.

## A complete example

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

def main() -> Result[void, json.Error]:
    source = r#"{"id":1,"name":"Ada","active":true}"#
    user = parse_user(source)?
    debug(ref user)
    return Ok()
```

!!! note "Crates come from `tinys.toml`"

    This `serde_json` example builds and runs like any other, provided `serde_json`
    is declared in the package's `tinys.toml`. See
    [Rust interoperability](../advanced/interop.md) and
    [Language status](../about/status.md).

## Generated Rust

```python
user = parse(source)?
```

```rust
let user = parse(source)?;
```

## Where to go next

- [Pattern matching](pattern-matching.md) — destructure `Result` / `Option`.
- [Rust interoperability](../advanced/interop.md) — error types from crates.
