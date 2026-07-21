# TinyS — TL;DR

**Python-shaped syntax, Rust semantics, native binaries.**

You write indentation-based `.sn` source; it transpiles to Rust and compiles to
a native binary. No GC, no runtime — ownership, borrowing, lifetimes, traits,
and `Result`-based errors all carry over from Rust unchanged.

```text
.sn source  →  generated .rs  →  rustc / Cargo  →  native binary
```

Python's *look*, not Python's *guarantees*: conditions must be `bool`, matches
are exhaustive, no implicit truthiness, no implicit conversion, no `null`.

## In one glance

| TinyS                        | Rust                     |
| ---------------------------- | ------------------------ |
| `def f(x: i32) -> i32:`      | `fn f(x: i32) -> i32 {}` |
| `list[i32]` / `dict[str,i32]`| `Vec<i32>` / `HashMap<…>`|
| `ref T` / `mut ref T`        | `&T` / `&mut T`          |
| `at value`                   | `*value`                 |
| `.a`, `.source`              | `'a`, `'source`          |
| `Result[User, Error]`        | `Result<User, Error>`    |
| `match` / `case`             | `match` arms             |
| `and` / `or` / `not`         | `&&` / `\|\|` / `!`      |
| `str` / `ref str`            | `String` / `&str`        |
| `from rust.regex import …`   | `use regex::…`           |

## A taste

```tinys
from rust.serde import Serialize, Deserialize

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User:
    id: u64
    name: str

impl User:

    def greeting(self: ref Self) -> str:
        return format("Hello, {}", self.name)


def main() -> void:
    user = User(id=1, name="Ada")
    print(user.greeting())
```

## The essentials

- **Immutable by default** — `count = 0`, opt into mutation with `mut total = 0`.
- **Borrowing is a keyword** — `ref` / `mut ref` instead of `&` / `&mut`;
  dereference with `at`; `move` for explicit ownership transfer.
- **Errors** — `Result`/`Option`, `?` propagation, `none` for absence. No exceptions.
- **Pattern matching** — exhaustive, an expression; `_` wildcard, `|` alternatives,
  `if` guards, `as` binds the whole value.
- **Control flow is expression-oriented** — `if`, `match`, and `loop` produce values.
- **Async** — `async def` with postfix `.await`.
- **Rust interop is always visible** — crates come through the `rust` root
  (`from rust.serde import …`); macros are imported and called without `!`.
- **Packaging** — `tinys.toml` maps onto Cargo; modules follow the file layout.

Full design drafts live in [doc/draft/](doc/draft/); see the [README](README.md)
for the complete tour.
