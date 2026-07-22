# Structs & enums

Structs group named fields; enums are algebraic data types whose variants can
carry data. Methods and associated functions live in `impl` blocks.

## Structs

```python
struct Point:
    x: f64
    y: f64
```

Construct with **keyword-style fields**:

```python
point = Point(
    x=10.0,
    y=20.0,
)
```

Field access uses a dot:

```python
print(point.x)
```

Attributes pass straight through to Rust:

```python
#[derive(Debug, Clone)]
struct Point:
    x: f64
    y: f64
```

## Methods and associated functions

An `impl` block adds behavior. The **receiver's ownership** is written out in its
type, which is how TinyS distinguishes constructors, read-only methods, and
mutating methods:

```python
impl Point:

    def new(x: f64, y: f64) -> Point:          // associated function (no self)
        return Point(x=x, y=y)

    def length(self: ref Self) -> f64:         // borrows self
        squared = self.x * self.x + self.y * self.y
        return squared.sqrt()

    def scaled(self: ref Self, factor: f64) -> Point:
        return Point(x=self.x * factor, y=self.y * factor)
```

The receiver types map onto Rust directly:

| TinyS               | Rust        |
| ------------------- | ----------- |
| `self: Self`        | `self`      |
| `self: ref Self`    | `&self`     |
| `self: mut ref Self`| `&mut self` |

Call methods and associated functions with a dot; TinyS emits Rust's `::` where
required:

```python
origin = Point.new(3.0, 4.0)   // Point::new(3.0, 4.0)
print(origin.length())
```

## Enums

Enums are algebraic data types — each variant may carry payload data:

```python
enum Shape:
    Circle(f64)
    Rectangle(f64, f64)
    Empty
```

Construct a variant with dot syntax:

```python
c = Shape.Circle(2.0)
r = Shape.Rectangle(3.0, 4.0)
e = Shape.Empty
```

Enums can be generic and take lifetime parameters:

```python
enum Token[.source]:
    Identifier(ref[.source] str)
    Number(i64)
    Plus
    End
```

You consume an enum by [pattern matching](pattern-matching.md), which is
exhaustive:

```python
def area(shape: Shape) -> f64:
    return match shape:
        case Shape.Circle(r):
            3.14159 * r * r

        case Shape.Rectangle(width, height):
            width * height

        case Shape.Empty:
            0.0
```

## Generated Rust

```python
#[derive(Debug, Clone)]
struct Point:
    x: f64
    y: f64

impl Point:
    def new(x: f64, y: f64) -> Point:
        return Point(x=x, y=y)
```

```rust
#[derive(Debug, Clone)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Point {
        return Point { x: x, y: y };
    }
}
```

## Where to go next

- [Pattern matching](pattern-matching.md) — take enums apart.
- [Traits](traits.md) — shared behavior across types.
- [Generics](generics.md) — parameterize structs and enums by type.
