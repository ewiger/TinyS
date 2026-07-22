# Syntax overview

TinyS uses Python-shaped surface syntax over Rust-oriented semantics. This page
covers the mechanical basics: how blocks, comments, literals, and operators are
written.

## Indentation-based blocks

Blocks are defined by **indentation**, not braces. A block is introduced by a line
ending in `:`.

```python
def maximum(left: i32, right: i32) -> i32:
    if left >= right:
        return left

    return right
```

There are no semicolons and no braces. Use consistent indentation (spaces are
recommended) the way you would in Python.

## Comments and documentation

TinyS keeps **Rust-compatible** comment syntax, so `//` and `///` map straight
through to Rust:

```python
// Ordinary comment

/// Public API documentation (Rust doc comment)
pub def load_user(id: u64) -> Option[User]:
    ...
```

Module-level documentation uses `//!`:

```python
//! User-service module.
```

Using `//` (rather than `#`) avoids ambiguity with Rust-style attributes, which
begin with `#`.

!!! note "`#` in examples"

    The runnable `.sn` examples in this manual sometimes use `#` for short inline
    notes so the snippets highlight cleanly. In real TinyS source, prefer `//`,
    `///`, and `//!` — they are the comment forms that map to Rust.

## Attributes

Rust attributes are already concise and interoperable, so TinyS keeps them
**unchanged**:

```python
#[derive(Debug, Clone, PartialEq)]
struct User:
    id: u64
    name: str
```

Other common attributes work the same way:

```python
#[test]
#[inline]
#[repr(C)]
#[cfg(target_os = "linux")]
```

## Names and types

Type expressions read left to right, using words instead of punctuation and
square brackets for type parameters:

```python
value: i32
name: ref str
users: list[User]
lookup: dict[str, User]
maybe: Option[str]
outcome: Result[User, Error]
```

See [Generics](generics.md) and [Collections & tuples](collections.md) for the
full type-expression vocabulary.

## Literals

```python
count = 42            // integer
ratio = 3.14          // float
flag = true           // bool  (lowercase true / false)
letter = 'a'          // char
name = "Ada"          // string literal
raw = r#"{"id":1}"#   // raw string literal (Rust-style)
```

TinyS uses lowercase `true` and `false`, and there is **no `null`** — absence is
modeled with [`Option`](error-handling.md).

## Operators

Arithmetic, comparison, and assignment operators match Rust:

```python
a + b    a - b    a * b    a / b    a % b
a == b   a != b   a < b    a <= b   a > b   a >= b
x += 1   x -= 1   x *= 2   x /= 2   x %= 3
```

Logical operators use **words**:

| TinyS   | Rust  |
| ------- | ----- |
| `and`   | `&&`  |
| `or`    | `\|\|`|
| `not`   | `!`   |

```python
if not values.is_empty() and count > 0:
    process(values)
```

Conditions must be `bool`; there is no implicit truthiness and no implicit
numeric-to-bool conversion.

## Blocks are expressions

Many constructs that are statements in other languages are **expressions** in
TinyS — they produce a value. The final expression of a block is its value:

```python
status = if active:
    "active"
else:
    "inactive"
```

This applies to [`if`, `match`, and `loop`](control-flow.md), and to the trailing
expression of a [function](functions.md) or [closure](closures.md) body.

## Where to go next

- [Variables & mutability](variables.md)
- [Functions](functions.md)
- [Control flow](control-flow.md)
