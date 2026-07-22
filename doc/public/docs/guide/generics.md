# Generics

Generic types and functions are parameterized by type. TinyS uses **square
brackets** for type parameters, where Rust uses angle brackets.

## Generic types

```python
list[i32]
dict[str, User]
Option[str]
Result[User, Error]
```

These map onto Rust's `Vec<i32>`, `HashMap<String, User>`, `Option<String>`, and
`Result<User, Error>`.

## Generic functions

Type parameters go in square brackets after the function name:

```python
def identity[T](value: T) -> T:
    return value
```

## Trait bounds

Constrain a type parameter with `:` and a trait:

```python
def max_of[T: PartialOrd](left: T, right: T) -> T:
    if left >= right:
        return left

    return right

def clone_value[T: Clone](value: ref T) -> T:
    return value.clone()
```

This is a runnable example — try it:

```python
def identity[T](value: T) -> T:
    return value

def main() -> void:
    print(identity(42))
    print(identity(true))
    print(max_of(3, 9))
    print(max_of(2.5, 1.5))
```

## Generic calls into Rust

When you call a Rust generic function, the type argument also uses square
brackets and TinyS emits the Rust turbofish:

```python
user = json.from_str[User](source)?
```

```rust
let user = serde_json::from_str::<User>(source)?;
```

See [Rust interoperability](../advanced/interop.md).

## Lifetime parameters

Lifetimes are a kind of generic parameter and use the same square-bracket syntax;
their names begin with a dot:

```python
def longest[.a](
    left: ref[.a] str,
    right: ref[.a] str,
) -> ref[.a] str:
    ...
```

See [Lifetimes](../advanced/lifetimes.md).

## Generated Rust

```python
def identity[T](value: T) -> T:
    return value

def max_of[T: PartialOrd](left: T, right: T) -> T:
    ...
```

```rust
fn identity<T>(value: T) -> T {
    return value;
}

fn max_of<T: PartialOrd>(left: T, right: T) -> T {
    // ...
}
```

!!! info "`where` clauses"

    Syntax for complex `where` clauses is one of the language's open design
    questions. See [Language status](../about/status.md).

## Where to go next

- [Traits](traits.md) — the bounds you place on type parameters.
- [Collections & tuples](collections.md) — the most common generic types.
- [Lifetimes](../advanced/lifetimes.md)
