# Tutorial — a guided tour

This tutorial builds understanding one concept at a time. Every snippet here uses
features that **compile and run today** with `tinys run`, so you can follow along
in your own `.sn` files. When you want the full detail on any topic, follow the
links into the [Language guide](../guide/index.md).

By the end you will have written functions, mutable state, structs with methods,
an enum, exhaustive pattern matching, borrowing, and a value-producing loop.

## 1. Values, types, and functions

Values are **immutable by default**. A function is introduced with `def`, takes
explicitly typed parameters, and declares its return type after `->`.

```python
def add(left: i64, right: i64) -> i64:
    return left + right

def square(value: i32) -> i32:
    value * value          # the final expression is the return value

def main() -> void:
    print(add(3, 4))       # 7
    print(square(5))       # 25
```

Two things to notice:

- `add` uses an explicit `return`; `square` relies on the **final expression**
  becoming the return value (like Rust's trailing expression).
- Number literals take the type required by their context (`i64` vs `i32`).

→ [Functions](../guide/functions.md)

## 2. Mutability and control flow

Opt into mutation with `mut`. Conditions must be `bool` — there is no Python-style
truthiness — and `if`/`elif`/`else` work the way they look.

```python
def classify(n: i32) -> str:
    if n > 0:
        return "positive"
    elif n < 0:
        return "negative"
    else:
        return "zero"

def main() -> void:
    mut total = 0
    for i in 0..=5:            # inclusive range 0,1,2,3,4,5
        total += i
    print(total)              # 15

    print(classify(-4))       # negative
    print(classify(7))        # positive
```

Ranges follow Rust semantics: `0..5` is exclusive, `0..=5` is inclusive.

→ [Variables & mutability](../guide/variables.md) · [Control flow](../guide/control-flow.md)

## 3. Structs and methods

A `struct` groups named fields. An `impl` block adds associated functions (like
constructors) and methods. The receiver's ownership is spelled out in its type:
`self: ref Self` borrows, so the method can read `self` without consuming it.

```python
#[derive(Debug, Clone)]
struct Point:
    x: f64
    y: f64

impl Point:

    def new(x: f64, y: f64) -> Point:
        return Point(x=x, y=y)

    def length(self: ref Self) -> f64:
        squared = self.x * self.x + self.y * self.y
        return squared.sqrt()

    def scaled(self: ref Self, factor: f64) -> Point:
        return Point(x=self.x * factor, y=self.y * factor)

def main() -> void:
    origin = Point.new(3.0, 4.0)
    print(origin.length())    # 5

    bigger = origin.scaled(2.0)
    print(bigger.length())    # 10
```

- Construction uses **keyword fields**: `Point(x=..., y=...)`.
- Associated functions are called with a dot: `Point.new(...)` (TinyS emits
  Rust's `Point::new(...)`).
- `#[derive(...)]` attributes pass straight through to Rust unchanged.

→ [Structs & enums](../guide/structs-and-enums.md)

## 4. Enums and pattern matching

Enums are **algebraic data types** — each variant can carry data. You take them
apart with `match`/`case`, which is **exhaustive** (you must cover every variant)
and is an **expression** (it produces a value).

```python
enum Shape:
    Circle(f64)
    Rectangle(f64, f64)
    Empty

def area(shape: Shape) -> f64:
    return match shape:
        case Shape.Circle(r):
            3.14159 * r * r

        case Shape.Rectangle(width, height):
            width * height

        case Shape.Empty:
            0.0

def main() -> void:
    shapes = [Shape.Circle(2.0), Shape.Rectangle(3.0, 4.0), Shape.Empty]
    for shape in shapes:
        print(area(shape))
```

Because `match` is exhaustive, adding a new variant to `Shape` and forgetting to
handle it becomes a **compile error**, not a runtime surprise.

→ [Pattern matching](../guide/pattern-matching.md)

## 5. Borrowing: `ref`, `mut ref`, and `at`

Passing an owned value can move it. To read or modify a value **without taking
ownership**, borrow it. TinyS uses words where Rust uses punctuation:

| TinyS         | Rust     | Meaning                    |
| ------------- | -------- | -------------------------- |
| `ref x`       | `&x`     | shared (read-only) borrow  |
| `mut ref x`   | `&mut x` | exclusive (mutable) borrow |
| `at x`        | `*x`     | dereference                |

```python
def increment(value: mut ref i32) -> void:
    at value += 1                # modify through the mutable reference

def sum(values: ref list[i32]) -> i32:
    mut total = 0
    for value in values:
        total += at value        # read through each shared reference
    return total

def main() -> void:
    mut count = 0
    increment(mut ref count)
    increment(mut ref count)
    print(count)                 # 2

    numbers: list[i32] = [1, 2, 3, 4]
    print(sum(ref numbers))      # 10  — numbers is only borrowed, still usable
```

Field access and method calls auto-dereference (you write `self.x`, not
`at self.x`), so `at` is only needed for direct reads and assignments through a
reference.

→ [Ownership & borrowing](../guide/ownership.md)

## 6. Loops that produce values

`loop` is an infinite loop that you exit with `break`. Like `if` and `match`, a
`loop` is an **expression**: `break value` makes the whole loop evaluate to that
value. Labeled loops (named with `as`) let an inner loop break out of an outer
one, optionally carrying a value with `with`.

```python
def first_positive_even() -> i32:
    mut n = 0
    return loop:
        n += 1
        if n % 2 == 0:
            break n              # the loop evaluates to n

def find_pair(target: i32) -> i32:
    return loop as search:
        for a in 0..5:
            for b in 0..5:
                if a + b == target:
                    break search with a * 10 + b
        break -1

def main() -> void:
    print(first_positive_even())  # 2
    print(find_pair(6))           # 24  (a=2, b=4)
    print(find_pair(99))          # -1  (no pair found)
```

→ [Control flow](../guide/control-flow.md)

## Put it together

Everything above lives in the repository's [`examples/`](https://github.com/ewiger/TinyS/tree/main/examples)
directory as standalone, runnable programs. Try them:

```bash
tinys run examples/structs.sn
tinys run examples/enums.sn
tinys run examples/loops.sn
tinys emit-rust examples/structs.sn   # see the Rust it becomes
```

## Where to go next

- **Systematic reference** — the [Language guide](../guide/index.md) covers each
  feature in depth, including ones this tour skipped (traits, generics, closures,
  error handling, modules).
- **Rust interop** — bring in crates like `serde` with
  [Rust interoperability](../advanced/interop.md).
- **Quick lookups** — the [keyword list](../reference/keywords.md) and the
  [TinyS ↔ Rust cheat sheet](../reference/cheatsheet.md).
