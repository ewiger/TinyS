# Closures

Closures are anonymous functions that can capture variables from their
surroundings. TinyS writes them with `fn`.

## Basic closures

```python
double = fn(value: i32) -> i32:
    value * 2
```

Call it like any function:

```python
print(double(21))    // 42
```

## Multiline closures

A closure body is an indented block; its final expression is the result:

```python
transform = fn(value: i32) -> i32:
    adjusted = value * 2
    adjusted + 1
```

## Closures and iteration

Closures pair naturally with loops and iterators:

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

    print(adder(10, 32))   // 42
```

## Move closures

A closure that must **own** its captured values reuses the `move` keyword:

```python
worker = move fn():
    process(data)
```

This maps to Rust's `move ||  { ... }`. See
[Ownership & borrowing](ownership.md) for what `move` means.

## Generated Rust

```python
double = fn(value: i32) -> i32:
    value * 2
```

```rust
let double = |value: i32| -> i32 {
    value * 2
};
```

!!! info "Closure syntax is an active area"

    The precise closure surface syntax (type inference on parameters, capture
    rules) is one of the language's open design questions. The `fn(...)` form
    above compiles today — see the runnable
    [`closures.sn`](../examples/index.md) example.

## Where to go next

- [Functions](functions.md)
- [Ownership & borrowing](ownership.md)
