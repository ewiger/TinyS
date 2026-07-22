# TinyS ↔ Rust cheat sheet

A one-page mapping between TinyS surface syntax and the Rust it becomes.

## Core mappings

| TinyS                          | Rust                       |
| ------------------------------ | -------------------------- |
| `def f(x: i32) -> i32:`        | `fn f(x: i32) -> i32 {}`   |
| `list[i32]` / `dict[str, i32]` | `Vec<i32>` / `HashMap<…>`  |
| `ref T` / `mut ref T`          | `&T` / `&mut T`            |
| `ref x` / `mut ref x`          | `&x` / `&mut x`            |
| `at value`                     | `*value`                   |
| `move x`                       | `move` / moved value       |
| `.a`, `.source`                | `'a`, `'source`            |
| `ref[.a] str`                  | `&'a str`                  |
| `Result[User, Error]`          | `Result<User, Error>`      |
| `Option[str]`                  | `Option<String>`           |
| `match` / `case`               | `match` arms               |
| `if case` / `while case`       | `if let` / `while let`     |
| `and` / `or` / `not`           | `&&` / `\|\|` / `!`        |
| `str` / `ref str`              | `String` / `&str`          |
| `none`                         | `None`                     |
| `void`                         | `()`                       |
| `from rust.regex import …`     | `use regex::…`             |
| `from macro import format`     | (import) `format!`         |
| `f[T](x)`                      | `f::<T>(x)`                |
| `Point.new(…)`                 | `Point::new(…)`            |

## Functions

```python
def add(left: i64, right: i64) -> i64:
    return left + right
```

```rust
fn add(left: i64, right: i64) -> i64 {
    return left + right;
}
```

## Structs & impl

```python
#[derive(Debug, Clone)]
struct Point:
    x: f64
    y: f64

impl Point:
    def length(self: ref Self) -> f64:
        return (self.x * self.x + self.y * self.y).sqrt()
```

```rust
#[derive(Debug, Clone)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn length(&self) -> f64 {
        return (self.x * self.x + self.y * self.y).sqrt();
    }
}
```

## Enums & match

```python
enum Shape:
    Circle(f64)
    Empty

description = match shape:
    case Shape.Circle(_):
        "circle"
    case Shape.Empty:
        "nothing"
```

```rust
enum Shape {
    Circle(f64),
    Empty,
}

let description = match shape {
    Shape::Circle(_) => "circle",
    Shape::Empty => "nothing",
};
```

## Receiver types

| TinyS                | Rust        |
| -------------------- | ----------- |
| `self: Self`         | `self`      |
| `self: ref Self`     | `&self`     |
| `self: mut ref Self` | `&mut self` |

## Errors & options

```python
user = parse(source)?
```

```rust
let user = parse(source)?;
```

| TinyS               | Rust                |
| ------------------- | ------------------- |
| `Some(value)`       | `Some(value)`       |
| `none`              | `None`              |
| `Ok()` / `Err(e)`   | `Ok(())` / `Err(e)` |
| `value?`            | `value?`            |

See the [keyword list](keywords.md) for every keyword, and the
[TL;DR](../index.md#in-one-glance) on the home page for the short version.
