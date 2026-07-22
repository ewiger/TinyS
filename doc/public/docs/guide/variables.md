# Variables & mutability

TinyS follows Rust-style ownership and mutability rather than Python-style
unrestricted reassignment.

## Immutable by default

A plain assignment binds an **immutable** value. The type is inferred, or you can
annotate it:

```python
name = "Ada"
count: i32 = 0
```

Once bound, an immutable variable cannot be reassigned.

## Mutable variables

Opt into mutation explicitly with `mut`:

```python
mut total: i64 = 0
total += 1
```

This maps to Rust's `let mut total: i64 = 0;`.

```python
def main() -> void:
    mut count = 3
    while count > 0:
        print(count)
        count -= 1
```

## Type annotations

Annotations use `name: Type`. They are optional when the type is clear from the
initializer and required when it is not (for example, an empty collection):

```python
count: i32 = 0
numbers: list[i32] = [1, 2, 3]
```

## Declaration vs. reassignment

A bare `name = value` **declares** a binding. Reassignment only applies to a
binding you previously declared `mut`. This is a deliberate difference from
Python, where any assignment rebinds a name.

```python
mut score = 0
score = 10        // OK: score is mutable

total = 0
total = 5         // error: total is immutable
```

## Constants and statics

!!! info "Designed — see the spec"

    Module-level constants and statics follow Rust's model. Their exact TinyS
    surface syntax is being finalized as part of the language spec; check
    [Language status](../about/status.md) for the current state.

## Generated Rust

```python
name = "Ada"
count: i32 = 0
mut total: i64 = 0
```

```rust
let name = "Ada";
let count: i32 = 0;
let mut total: i64 = 0;
```

## Where to go next

- [Ownership & borrowing](ownership.md) — what happens when you pass a value on.
- [Functions](functions.md) — parameters and returns.
