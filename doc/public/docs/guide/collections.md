# Collections & tuples

TinyS provides Python-shaped literals for the standard Rust collection types,
plus tuples.

## Standard collections

| TinyS         | Rust           |
| ------------- | -------------- |
| `list[T]`     | `Vec<T>`       |
| `array[T, N]` | `[T; N]`       |
| `slice[T]`    | `[T]`          |
| `dict[K, V]`  | `HashMap<K, V>`|
| `set[T]`      | `HashSet<T>`   |

## List literals

```python
numbers: list[i32] = [1, 2, 3]
```

Iterating and indexing work as you would expect:

```python
for n in ref numbers:
    print(n)

first = numbers[0]
```

Indexing follows Rust-style bounds behavior (out-of-bounds panics); use a method
for a safe, optional lookup:

```python
value = values[index]      // panics if out of bounds
value = values.get(index)  // returns Option[T]
```

## Dict literals

```python
mapping: dict[str, i32] = {
    "one": 1,
    "two": 2,
}
```

## Tuples

A tuple groups a fixed number of values of possibly different types:

```python
pair: (str, i32) = ("Ada", 42)
```

Destructure it:

```python
name, age = pair
```

Or access fields by position:

```python
name = pair.0
age = pair.1
```

## Generated Rust

```python
numbers: list[i32] = [1, 2, 3]
pair: (str, i32) = ("Ada", 42)
```

```rust
let numbers: Vec<i32> = vec![1, 2, 3];
let pair: (String, i32) = ("Ada".to_string(), 42);
```

!!! info "Literal inference is still being finalized"

    The exact rules for `dict`/`set` literals, `array` vs `list`, and string
    literal inference are being finalized. Programs that run today (see the
    [examples](../examples/index.md)) rely mainly on `list` literals, ranges, and
    tuples. Check [Language status](../about/status.md) for current coverage.

## Where to go next

- [Pattern matching](pattern-matching.md) — destructure tuples and enums.
- [Control flow](control-flow.md) — iterate collections.
- [Generics](generics.md) — the type parameters behind these collections.
