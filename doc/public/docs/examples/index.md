# Examples

Every program below is a real, standalone `.sn` file in the repository's
[`examples/`](https://github.com/ewiger/TinyS/tree/main/examples) directory. The
first ten are pure-`std` and **compile and run today** with `rustc` alone; the
interop example is *emit-only*.

Run any of them:

```bash
tinys run examples/hello.sn
tinys emit-rust examples/structs.sn    # see the generated Rust
```

## Hello, world

The canonical first program.

```python
def main() -> void:
    print("Hello from TinyS")
```

[:octicons-file-code-16: examples/hello.sn](https://github.com/ewiger/TinyS/blob/main/examples/hello.sn)

## Functions

Explicit types, early returns, and implicit tail returns.

```python
def add(left: i64, right: i64) -> i64:
    return left + right

def square(value: i32) -> i32:
    value * value

def maximum(left: i32, right: i32) -> i32:
    if left >= right:
        return left

    return right

def main() -> void:
    print(add(3, 4))
    print(square(5))
    print(maximum(10, 20))
```

[:octicons-file-code-16: examples/functions.sn](https://github.com/ewiger/TinyS/blob/main/examples/functions.sn) · [Guide → Functions](../guide/functions.md)

## Control flow

`if` / `elif` / `else`, ranges, `while`, and mutable accumulation.

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
    for i in 0..=5:
        total += i
    print(total)

    mut count = 3
    while count > 0:
        print(count)
        count -= 1

    print(classify(-4))
    print(classify(7))
```

[:octicons-file-code-16: examples/control_flow.sn](https://github.com/ewiger/TinyS/blob/main/examples/control_flow.sn) · [Guide → Control flow](../guide/control-flow.md)

## FizzBuzz

Ranges, integer arithmetic, and owned-string returns.

```python
def fizzbuzz(n: i32) -> str:
    if n % 15 == 0:
        return "FizzBuzz"
    elif n % 3 == 0:
        return "Fizz"
    elif n % 5 == 0:
        return "Buzz"
    else:
        return format("{}", n)

def main() -> void:
    for i in 1..=15:
        print(fizzbuzz(i))
```

[:octicons-file-code-16: examples/fizzbuzz.sn](https://github.com/ewiger/TinyS/blob/main/examples/fizzbuzz.sn)

## Structs

Structs, associated functions, and methods with borrowed receivers.

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
    print(origin.length())

    bigger = origin.scaled(2.0)
    print(bigger.length())
```

[:octicons-file-code-16: examples/structs.sn](https://github.com/ewiger/TinyS/blob/main/examples/structs.sn) · [Guide → Structs & enums](../guide/structs-and-enums.md)

## Enums

Algebraic data types and exhaustive, expression-oriented matching.

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
    for shape in ref shapes:
        print(area(shape))
```

[:octicons-file-code-16: examples/enums.sn](https://github.com/ewiger/TinyS/blob/main/examples/enums.sn) · [Guide → Pattern matching](../guide/pattern-matching.md)

## References

Explicit borrowing (`ref` / `mut ref`) and dereferencing with `at`.

```python
def increment(value: mut ref i32) -> void:
    at value += 1

def sum(values: ref list[i32]) -> i32:
    mut total = 0
    for value in values:
        total += at value
    return total

def main() -> void:
    mut count = 0
    increment(mut ref count)
    increment(mut ref count)
    print(count)

    numbers: list[i32] = [1, 2, 3, 4]
    print(sum(ref numbers))
```

[:octicons-file-code-16: examples/references.sn](https://github.com/ewiger/TinyS/blob/main/examples/references.sn) · [Guide → Ownership & borrowing](../guide/ownership.md)

## Closures

Closures and iteration.

```python
def main() -> void:
    double = fn(x: i32) -> i32:
        x * 2

    adder = fn(a: i32, b: i32) -> i32:
        total = a + b
        total

    numbers = [1, 2, 3, 4, 5]
    for n in numbers:
        print(double(n))

    print(adder(10, 32))
```

[:octicons-file-code-16: examples/closures.sn](https://github.com/ewiger/TinyS/blob/main/examples/closures.sn) · [Guide → Closures](../guide/closures.md)

## Generics

Generic functions and trait bounds.

```python
def identity[T](value: T) -> T:
    return value

def max_of[T: PartialOrd](left: T, right: T) -> T:
    if left >= right:
        return left

    return right

def main() -> void:
    print(identity(42))
    print(identity(true))
    print(max_of(3, 9))
    print(max_of(2.5, 1.5))
```

[:octicons-file-code-16: examples/generics.sn](https://github.com/ewiger/TinyS/blob/main/examples/generics.sn) · [Guide → Generics](../guide/generics.md)

## Loops that produce values

Value-producing loops, labeled breaks, and `break … with`.

```python
def first_positive_even() -> i32:
    mut n = 0
    return loop:
        n += 1
        if n % 2 == 0:
            break n

def find_pair(target: i32) -> i32:
    return loop as search:
        for a in 0..5:
            for b in 0..5:
                if a + b == target:
                    break search with a * 10 + b
        break -1

def main() -> void:
    print(first_positive_even())
    print(find_pair(6))
    print(find_pair(99))
```

[:octicons-file-code-16: examples/loops.sn](https://github.com/ewiger/TinyS/blob/main/examples/loops.sn) · [Guide → Control flow](../guide/control-flow.md)

## Rust interop (emit-only)

A `serde_json` showcase. Inspect it with `tinys emit-rust examples/json_user.sn`;
building it needs Cargo-managed crates.

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

def describe(user: ref User) -> str:
    return if user.active:
        format("{} is active", user.name)
    else:
        format("{} is inactive", user.name)

def main() -> Result[void, json.Error]:
    source = r#"{"id":1,"name":"Ada","active":true}"#
    user = parse_user(source)?
    debug(ref user)
    print(describe(ref user))
    return Ok()
```

[:octicons-file-code-16: examples/json_user.sn](https://github.com/ewiger/TinyS/blob/main/examples/json_user.sn) · [Advanced → Rust interoperability](../advanced/interop.md)
