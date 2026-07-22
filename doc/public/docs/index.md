---
title: TinyS — Python-shaped syntax, Rust semantics
---

# TinyS

<p class="tinys-tagline">A small statically typed language with <strong>Python-shaped syntax</strong> and <strong>Rust-oriented semantics</strong>. You write indentation-based <code>.sn</code> source; it transpiles to Rust and compiles to a native binary.</p>

```text
.sn source  →  generated .rs  →  rustc / Cargo  →  native binary
```

TinyS aims to make systems programming more **readable** without weakening the
ownership, borrowing, type-safety, and interoperability guarantees that make
Rust valuable.

```python
def maximum(left: i32, right: i32) -> i32:
    if left >= right:
        return left

    return right
```

You keep Python's *look* — indentation blocks, `def`, `match`/`case`, `for … in …` —
but not Python's *runtime*: no garbage collector, no interpreter, conditions must
be `bool`, matches are exhaustive, there is no implicit truthiness and no `null`.

<div class="grid cards" markdown>

-   :material-rocket-launch: **Get started**

    ---

    Install the compiler, build the classic *Hello, world*, and follow a guided
    tour of the language.

    [:octicons-arrow-right-24: Getting started](getting-started/index.md)

-   :material-book-open-variant: **Language guide**

    ---

    A topic-by-topic reference: variables, ownership, structs, enums, pattern
    matching, traits, generics, and error handling.

    [:octicons-arrow-right-24: Language guide](guide/index.md)

-   :material-flask: **Advanced**

    ---

    Lifetimes, async, Rust interoperability, macros, and how to read the
    generated Rust.

    [:octicons-arrow-right-24: Advanced](advanced/index.md)

-   :material-table: **Reference**

    ---

    CLI commands, the keyword list, and a TinyS ↔ Rust cheat sheet.

    [:octicons-arrow-right-24: Reference](reference/index.md)

</div>

## Why TinyS?

Rust has a powerful semantic model, but some of its syntax reflects historical
C-family conventions. Python has a highly readable layout, but its runtime does
not provide Rust's ownership, native performance, deterministic destruction, or
compile-time safety.

TinyS explores a narrow question:

> What would a Rust-oriented systems language look like if its surface syntax had
> been designed around indentation, readable type expressions, and explicit words
> rather than pointer punctuation?

The goal is not to make Rust disappear — it is to make Rust's strongest ideas
easier to read and write. TinyS is **not intended to hide Rust semantics**: it
should stay clear when values are owned, borrowed, moved, cloned, mutated, or
passed into external Rust code.

## In one glance

| TinyS                          | Rust                       |
| ------------------------------ | -------------------------- |
| `def f(x: i32) -> i32:`        | `fn f(x: i32) -> i32 {}`   |
| `list[i32]` / `dict[str, i32]` | `Vec<i32>` / `HashMap<…>`  |
| `ref T` / `mut ref T`          | `&T` / `&mut T`            |
| `at value`                     | `*value`                   |
| `.a`, `.source`                | `'a`, `'source`            |
| `Result[User, Error]`          | `Result<User, Error>`      |
| `match` / `case`               | `match` arms               |
| `and` / `or` / `not`           | `&&` / `\|\|` / `!`        |
| `str` / `ref str`              | `String` / `&str`          |
| `from rust.regex import …`     | `use regex::…`             |

## A taste

```python
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

!!! note "Project status"

    TinyS is an **experimental** language design and compiler project. A working
    **v0.1.0 compiler** exists — an indentation-aware lexer, a recursive-descent
    parser, and a Rust source generator driven by the `tinys` CLI. It is not yet
    ready for production use. See [Language status](about/status.md) for exactly
    what compiles and runs today versus what is still on the
    [roadmap](about/roadmap.md).
