# Functions

Functions are introduced with `def` and use explicit parameter and return types.

## Definition

```python
def add(left: i64, right: i64) -> i64:
    return left + right
```

- Parameters are written `name: Type`.
- The return type follows `->`.
- The body is an indented block.

## Return values

You can return explicitly with `return`, or let the **final expression** of the
body become the return value (as in Rust):

```python
def square(value: i32) -> i32:
    value * value          // implicit return

def maximum(left: i32, right: i32) -> i32:
    if left >= right:
        return left        // early return

    return right           // final expression
```

Both styles can appear in the same function: use early `return` for guard clauses
and a trailing expression for the main result.

## The `void` return type

A function that returns nothing declares `-> void`, which maps to Rust's unit type
`()`:

```python
def greet(name: ref str) -> void:
    print(format("Hello, {}", name))
```

## Parameters and ownership

A parameter's type says how the value is passed. An owned parameter takes
ownership; a `ref` / `mut ref` parameter borrows it:

```python
def consume(value: Data) -> void:     // takes ownership of value
    process(value)

def observe(value: ref Data) -> void: // borrows value; caller keeps it
    print(value.name)

def bump(value: mut ref i32) -> void: // borrows mutably
    at value += 1
```

See [Ownership & borrowing](ownership.md) for the full model.

## Generic functions

Type parameters go in square brackets after the name; trait bounds are explicit:

```python
def identity[T](value: T) -> T:
    return value

def max_of[T: PartialOrd](left: T, right: T) -> T:
    if left >= right:
        return left

    return right
```

See [Generics](generics.md) for details.

## Visibility

Functions are private to their module by default. Use `pub` to export them:

```python
pub def load_user(id: u64) -> Result[User, Error]:
    ...
```

More restricted visibility uses square brackets:

```python
pub[crate] def helper() -> void:
    ...
```

See [Modules & imports](modules.md).

## Async functions

```python
async def fetch_user(id: u64) -> Result[User, Error]:
    response = client.get(id).await?
    return response.json[User]().await
```

Async keeps postfix `.await`. See [Async](../advanced/async.md).

## Generated Rust

```python
def add(left: i64, right: i64) -> i64:
    return left + right

def square(value: i32) -> i32:
    value * value
```

```rust
fn add(left: i64, right: i64) -> i64 {
    return left + right;
}

fn square(value: i32) -> i32 {
    value * value
}
```

## Where to go next

- [Ownership & borrowing](ownership.md)
- [Closures](closures.md) — anonymous functions with `fn`.
- [Generics](generics.md)
