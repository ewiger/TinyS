# Ownership & borrowing

Ownership is where TinyS keeps Rust's semantics exactly while trading punctuation
for words. There is no garbage collector: every value has an owner, and you
borrow instead of copying by default.

## Owned values

Values are owned by default. Passing an owned, non-`Copy` value **transfers
ownership** exactly where Rust would:

```python
def consume(value: Data) -> void:
    process(value)
```

You can emphasize the transfer with `move`:

```python
consume(move data)
// after this, `data` is unavailable
```

An optional `own` annotation may be used for documentation:

```python
def consume(value: own Data) -> void:
    ...
```

## Shared references — `ref`

A shared (read-only) borrow uses `ref`, both in types and when creating one:

```python
value: ref i32
name: ref str

value_ref = ref value      // like Rust's &value
```

`ref value` maps to Rust's `&value`.

## Mutable references — `mut ref`

An exclusive (mutable) borrow uses `mut ref`:

```python
def increment(value: mut ref i32) -> void:
    at value += 1

value_ref = mut ref value  // like Rust's &mut value
```

## Dereferencing — `at`

Explicit dereferencing uses `at`, replacing Rust's prefix `*`:

```python
copied: i32 = at source

at target = at source      // *target = *source;
```

```rust
*target = *source;
```

`at` **never implicitly clones**. To read through a reference you use `at`; to get
an owned copy of a borrowed value you `clone` it:

```python
number: i32 = at number_ref     // copy (i32 is Copy)
text: str = clone text_ref      // explicit clone of an owned String
```

## Automatic dereferencing for fields and methods

Ordinary field access and method calls **auto-dereference** where Rust would, so
you rarely write `at` for those:

```python
user: ref User

print(user.name)     // no `at` needed
user.display()
```

You only reach for `at` when reading a scalar through a reference or assigning
through a mutable one.

## A complete example

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
    print(count)                 // 2

    numbers: list[i32] = [1, 2, 3, 4]
    print(sum(ref numbers))      // 10 — numbers is only borrowed
```

## Iterating by borrow

Iterating an owned collection may consume it. Borrow it to iterate without moving:

```python
for user in users:            // may consume `users`
    process(user)

for user in ref users:        // shared borrow
    print(user.name)

for user in mut ref users:    // mutable borrow
    user.active = true
```

See [Control flow](control-flow.md#for-loops) for more on iteration.

## The keyword ↔ Rust map

| TinyS         | Rust     | Meaning                       |
| ------------- | -------- | ----------------------------- |
| `ref x`       | `&x`     | create a shared reference     |
| `mut ref x`   | `&mut x` | create a mutable reference    |
| `ref T`       | `&T`     | shared-reference type         |
| `mut ref T`   | `&mut T` | mutable-reference type        |
| `at x`        | `*x`     | dereference                   |
| `move x`      | `x`      | emphasize ownership transfer  |
| `clone x`     | `x.clone()` (conceptually) | explicit deep copy |

## Where to go next

- [Lifetimes](../advanced/lifetimes.md) — naming how long references live.
- [Functions](functions.md) — parameter ownership.
- [Structs & enums](structs-and-enums.md) — receiver ownership in methods.
