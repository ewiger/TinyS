# Lifetimes

Lifetimes name how long a reference is valid. TinyS keeps Rust's lifetime model
but writes lifetime names with a **leading dot** instead of an apostrophe.

## Lifetime names

```python
.a
.source
.store
```

These generate Rust lifetimes such as `'a`, `'source`, `'store`.

## Lifetimes on functions

Declare lifetimes alongside generic parameters in square brackets, and use them in
reference types with `ref[.name]`:

```python
def longest[.a](
    left: ref[.a] str,
    right: ref[.a] str,
) -> ref[.a] str:
    ...
```

```rust
fn longest<'a>(left: &'a str, right: &'a str) -> &'a str {
    // ...
}
```

## Lifetimes on types

Lifetime-parameterized structs and enums use the same bracket syntax:

```python
struct ConfigView[.store]:
    primary: ref[.store] str
    fallback: ref[.store] str
```

```python
enum Token[.source]:
    Identifier(ref[.source] str)
    Number(i64)
    Plus
    End
```

## Why a dot?

The leading dot keeps lifetime names visually distinct from ordinary identifiers
and type parameters while avoiding the apostrophe, which reads awkwardly in an
indentation-based language.

| TinyS         | Rust        |
| ------------- | ----------- |
| `.a`          | `'a`        |
| `ref[.a] str` | `&'a str`   |
| `[.a]`        | `<'a>`      |

!!! info "Designed — verify against the compiler"

    Lifetime syntax is part of the language design and appears throughout the
    reference. Coverage in the v0.1.0 compiler is evolving; consult
    [Language status](../about/status.md) and try `tinys emit-rust` on a small
    sample before relying on a specific form.

## Where to go next

- [Generics](../guide/generics.md)
- [Ownership & borrowing](../guide/ownership.md)
