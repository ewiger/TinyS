# Traits

Traits describe shared behavior — a set of methods a type can implement. They map
directly onto Rust traits.

## Declaring a trait

```python
trait Display:
    def display(self: ref Self) -> str
```

A method with no body is a required method: implementers must provide it.

## Implementing a trait

Use `impl Trait for Type`:

```python
impl Display for Point:

    def display(self: ref Self) -> str:
        return format("Point({}, {})", self.x, self.y)
```

## Default methods

A trait method **with** a body is a default; implementers may use it as-is or
override it:

```python
trait Display:

    def display(self: ref Self) -> str

    def debug_display(self: ref Self) -> str:
        return self.display()
```

## Trait bounds

Constrain a generic type parameter to types that implement a trait using `:`:

```python
def clone_value[T: Clone](value: ref T) -> T:
    return value.clone()

def max_of[T: PartialOrd](left: T, right: T) -> T:
    if left >= right:
        return left

    return right
```

See [Generics](generics.md) for multiple bounds and more.

## Inherent `impl` vs. trait `impl`

- `impl Type:` adds **inherent** methods and associated functions (constructors,
  helpers) — see [Structs & enums](structs-and-enums.md).
- `impl Trait for Type:` provides a trait's methods for that type.

## Generated Rust

```python
trait Display:
    def display(self: ref Self) -> str

impl Display for Point:
    def display(self: ref Self) -> str:
        return format("Point({}, {})", self.x, self.y)
```

```rust
trait Display {
    fn display(&self) -> String;
}

impl Display for Point {
    fn display(&self) -> String {
        return format!("Point({}, {})", self.x, self.y);
    }
}
```

!!! info "Advanced trait features"

    Trait objects, associated types, and advanced bounds/`where` clauses are part
    of the deeper Rust-interoperability phase. See the
    [roadmap](../about/roadmap.md) and [Language status](../about/status.md).

## Where to go next

- [Generics](generics.md)
- [Structs & enums](structs-and-enums.md)
