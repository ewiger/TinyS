# Control flow

TinyS control flow reads like Python but keeps Rust's rules: conditions must be
`bool`, there is no truthiness, and `if`, `match`, and `loop` are **expressions**
that can produce values.

## Conditionals

```python
if temperature > 30:
    print("hot")
elif temperature > 20:
    print("warm")
else:
    print("cold")
```

Conditions must have type `bool`. TinyS does **not** use Python-style truthiness,
so write the comparison explicitly:

```python
if not values.is_empty():
    process(values)
```

### `if` as an expression

An `if` block may produce a value; every branch must yield the same type:

```python
status = if active:
    "active"
else:
    "inactive"
```

## `while` loops

```python
while count > 0:
    count -= 1
```

## `for` loops

Iterate over any iterable. Ranges follow Rust semantics — `a..b` is exclusive,
`a..=b` is inclusive:

```python
for index in 0..10:      // 0 through 9
    print(index)

for index in 0..=10:     // 0 through 10
    print(index)
```

Iterating an owned collection may consume it; borrow to iterate without moving:

```python
for user in users:            // may consume users
    process(user)

for user in ref users:        // shared borrow
    print(user.name)

for user in mut ref users:    // mutable borrow
    user.active = true
```

See [Ownership & borrowing](ownership.md).

## `loop` — infinite loops that produce values

`loop` repeats until you `break`. Like `if` and `match`, it is an expression:
`break value` makes the whole loop evaluate to that value.

```python
result = loop:
    value = read_value()
    if value >= 0:
        break value
```

## Loop control

```python
break
continue
return
```

### Labeled loops — `as`

Name a loop with `as` so an inner loop can break out of an outer one:

```python
loop as search:
    for item in items:
        if item.matches():
            break search
```

### Breaking with a value — `with`

A labeled break that returns a value uses `with`:

```python
result = loop as search:
    for item in items:
        if item.matches():
            break search with item
```

Here is the pattern in a runnable function:

```python
def find_pair(target: i32) -> i32:
    return loop as search:
        for a in 0..5:
            for b in 0..5:
                if a + b == target:
                    break search with a * 10 + b
        break -1
```

## Pattern conditions — `if case` / `while case`

Rust's `if let` and `while let` are written with `case`:

```python
if case Some(user) = find_user(user_id):
    print(user.name)
else:
    print("not found")

while case Some(item) = queue.pop():
    process(item)
```

See [Pattern matching](pattern-matching.md).

## Generated Rust

```python
result = loop:
    n += 1
    if n % 2 == 0:
        break n
```

```rust
let result = loop {
    n += 1;
    if n % 2 == 0 {
        break n;
    }
};
```

## Where to go next

- [Pattern matching](pattern-matching.md)
- [Error handling](error-handling.md)
- [Closures](closures.md)
