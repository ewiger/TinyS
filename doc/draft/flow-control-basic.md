# TinyS Basic Flow Control

## 1. Conditional statements

TinyS uses Python-style `if`, `elif`, and `else`.

```tinys
if temperature > 30:
    print("hot")
elif temperature > 20:
    print("warm")
else:
    print("cold")
```

Generated Rust:

```rust
if temperature > 30 {
    println!("hot");
} else if temperature > 20 {
    println!("warm");
} else {
    println!("cold");
}
```

Parentheses around the condition are not required.

---

## 2. Conditions must be boolean

TinyS follows Rust rather than Python truthiness.

Valid:

```tinys
if count > 0:
    print(count)

if user.is_active:
    print(user.name)
```

Invalid:

```tinys
if count:
    ...
```

unless `count` is itself a `bool`.

Similarly, collections and strings are not implicitly converted to booleans:

```tinys
if not values.is_empty():
    print(values)
```

This avoids ambiguous implicit conversions.

---

## 3. `if` as an expression

Like Rust, an `if` block may produce a value.

```tinys
status: str = if active:
    "active"
else:
    "inactive"
```

Generated Rust:

```rust
let status: String = if active {
    "active".into()
} else {
    "inactive".into()
};
```

A multiline expression works the same way:

```tinys
price = if customer.is_premium:
    base_price * 0.8
else:
    base_price
```

When used as an expression, every reachable branch must produce a compatible type.

This is invalid:

```tinys
value = if ready:
    42
```

because no `else` branch exists.

---

## 4. Conditional assignment shorthand

Ordinary Python-style conditional expressions may also be supported:

```tinys
status = "active" if active else "inactive"
```

However, this form is less natural for multiline typed code.

The recommended primary syntax is:

```tinys
status = if active:
    "active"
else:
    "inactive"
```

The inline Python form can remain optional convenience syntax.

---

## 5. Infinite loops

Rust’s `loop` maps cleanly into TinyS.

```tinys
loop:
    process_next_item()
```

Generated Rust:

```rust
loop {
    process_next_item();
}
```

Unlike `while true`, `loop` explicitly represents an intentional infinite loop.

---

## 6. Breaking from a loop

`break` exits the nearest loop.

```tinys
loop:
    item = queue.next()

    if item.is_none():
        break

    process(item)
```

---

## 7. Loops as expressions

Like Rust, `loop` may return a value through `break`.

```tinys
result: i32 = loop:
    value = read_value()

    if value >= 0:
        break value
```

Generated Rust:

```rust
let result: i32 = loop {
    let value = read_value();

    if value >= 0 {
        break value;
    }
};
```

A bare `break` returns `void`.

```tinys
break
```

A value-producing break uses:

```tinys
break value
```

---

## 8. Continuing a loop

`continue` skips the remainder of the current iteration.

```tinys
for item in items:
    if item.is_invalid():
        continue

    process(item)
```

---

## 9. `while` loops

TinyS uses Python-style `while`.

```tinys
while count > 0:
    print(count)
    count -= 1
```

Generated Rust:

```rust
while count > 0 {
    println!("{}", count);
    count -= 1;
}
```

Conditions must be boolean.

---

## 10. `while` with pattern matching

Rust supports `while let`. TinyS can express this using `case`.

```tinys
while case Some(item) = queue.pop():
    process(item)
```

Generated Rust:

```rust
while let Some(item) = queue.pop() {
    process(item);
}
```

For a `Result`:

```tinys
while case Ok(message) = receiver.try_receive():
    process(message)
```

This syntax reuses the same pattern language as `match`.

---

## 11. Conditional pattern matching

Rust’s `if let` can use the same TinyS form:

```tinys
if case Some(user) = find_user(user_id):
    print(user.name)
else:
    print("not found")
```

Generated Rust:

```rust
if let Some(user) = find_user(user_id) {
    println!("{}", user.name);
} else {
    println!("not found");
}
```

This is preferable to introducing the Rust-specific phrase `if let`.

The general TinyS form is:

```tinys
if case pattern = expression:
    ...
```

---

## 12. Pattern guards in conditional matching

A matched value may be followed by an additional condition.

```tinys
if case Some(user) = find_user(user_id) if user.is_active:
    print(user.name)
```

Conceptually, this means:

```tinys
match find_user(user_id):
    case Some(user) if user.is_active:
        print(user.name)

    case _:
        pass
```

Whether this compact guard syntax is included in the first version may depend on parser simplicity.

---

## 13. `for` loops

TinyS uses Python-style iteration.

```tinys
for user in users:
    print(user.name)
```

Generated Rust:

```rust
for user in users {
    println!("{}", user.name);
}
```

Iteration semantics follow Rust’s `IntoIterator`.

The ownership behavior therefore depends on the iterated expression.

---

## 14. Iterating by ownership

Iterating over an owned collection consumes it when Rust would consume it.

```tinys
for user in users:
    process(user)
```

After the loop, `users` may no longer be available.

Conceptually generated Rust:

```rust
for user in users {
    process(user);
}
```

---

## 15. Iterating by shared reference

To preserve the collection, iterate over a shared reference.

```tinys
for user in ref users:
    print(user.name)
```

Conceptually generated Rust:

```rust
for user in &users {
    println!("{}", user.name);
}
```

Here, `user` is inferred as a shared reference to an element.

---

## 16. Iterating by mutable reference

Mutable iteration uses `mut ref`.

```tinys
for user in mut ref users:
    user.active = true
```

Generated Rust:

```rust
for user in &mut users {
    user.active = true;
}
```

This keeps borrowing explicit without requiring `.iter()` or `.iter_mut()` in ordinary cases.

---

## 17. Explicit iterator methods remain available

Rust iterator APIs are still accessible.

```tinys
for user in users.iter():
    print(user.name)

for user in users.iter_mut():
    user.active = true
```

The following forms are therefore equivalent in common cases:

```tinys
for user in ref users:
    ...
```

```tinys
for user in users.iter():
    ...
```

The `ref` form is shorter and emphasizes borrowing.

The `.iter()` form is useful when composing iterator operations.

---

## 18. Numeric ranges

Ranges follow Rust semantics but use familiar syntax.

Exclusive upper bound:

```tinys
for index in 0..10:
    print(index)
```

This produces values from `0` through `9`.

Inclusive upper bound:

```tinys
for index in 0..=10:
    print(index)
```

This produces values from `0` through `10`.

Generated Rust remains almost identical:

```rust
for index in 0..10 {
    println!("{}", index);
}
```

```rust
for index in 0..=10 {
    println!("{}", index);
}
```

Keeping Rust’s range syntax is preferable to translating Python’s `range(...)`, because ranges are first-class values in Rust.

---

## 19. Open-ended ranges

Open-ended ranges may be used where the type supports them.

```tinys
for index in 0..:
    process(index)
```

This represents an unbounded range.

Similarly:

```tinys
slice[start..]
slice[..end]
slice[..]
```

can retain Rust-compatible range semantics.

---

## 20. Enumeration

Python’s `enumerate` style is useful and maps naturally to Rust iterators.

```tinys
for index, user in enumerate(ref users):
    print(index, user.name)
```

Generated Rust:

```rust
for (index, user) in users.iter().enumerate() {
    println!("{} {}", index, user.name);
}
```

`enumerate` may be provided as a TinyS prelude function or compiler-recognized iterator helper.

---

## 21. Zipping iterators

```tinys
for left, right in zip(ref left_values, ref right_values):
    compare(left, right)
```

Generated Rust may use:

```rust
for (left, right) in left_values.iter().zip(right_values.iter()) {
    compare(left, right);
}
```

This keeps common iteration syntax compact while preserving iterator semantics.

---

## 22. Destructuring in loops

Loop variables may use patterns.

Tuple destructuring:

```tinys
for key, value in entries:
    print(key, value)
```

Enum destructuring:

```tinys
for result in results:
    match result:
        case Ok(value):
            process(value)

        case Err(error):
            report(error)
```

More directly, pattern-based loop headers may be supported:

```tinys
for Ok(value) in results:
    process(value)
```

However, this would silently skip or reject nonmatching values depending on semantics.

That behavior is potentially unclear, so the initial language should restrict ordinary `for` patterns to irrefutable patterns such as tuples and structs.

Valid:

```tinys
for index, value in values:
    ...
```

Potentially invalid:

```tinys
for Some(value) in values:
    ...
```

Refutable matching should instead use explicit iterator filtering or `match`.

---

## 23. Loop labels

Rust supports labels for controlling nested loops. TinyS needs an alternative to Rust’s apostrophe syntax because dots already represent lifetimes.

A readable option is `label`.

```tinys
label search:
    for row in rows:
        for value in row:
            if value == target:
                break search
```

Generated Rust:

```rust
'search: for row in rows {
    for value in row {
        if value == target {
            break 'search;
        }
    }
}
```

However, the block introduced by `label` is visually unusual.

A cleaner syntax is to attach the label directly to the loop:

```tinys
for row in rows as search:
    for value in row:
        if value == target:
            break search
```

For an infinite loop:

```tinys
loop as retry:
    ...
```

For a while loop:

```tinys
while connection.is_open() as receive:
    ...
```

This uses `as` consistently as a naming construct.

Recommended syntax:

```tinys
loop as outer:
    ...

while condition as outer:
    ...

for item in items as outer:
    ...
```

Control transfer then uses:

```tinys
break outer
continue outer
```

A value returned from a labeled loop can use:

```tinys
break outer with result
```

This avoids ambiguity between the label name and returned value.

---

## 24. Recommended labeled-break syntax

Because this is ambiguous:

```tinys
break outer value
```

TinyS should use `with` for a value-producing labeled break:

```tinys
break outer with value
```

Examples:

```tinys
result = loop as search:
    for item in items:
        if item.matches():
            break search with item
```

Generated Rust:

```rust
let result = 'search: loop {
    for item in items {
        if item.matches() {
            break 'search item;
        }
    }
};
```

Ordinary unlabeled value breaks stay concise:

```tinys
break value
```

---

## 25. Early function return

`return` exits the current function.

```tinys
def find_user(id: u64) -> Option[User]:
    if id == 0:
        return none

    return repository.find(id)
```

A function returning `void` may use a bare return:

```tinys
if not ready:
    return
```

TinyS may allow the final expression of a function to become the return value:

```tinys
def square(value: i32) -> i32:
    value * value
```

This matches Rust semantics.

Explicit `return` remains useful for early exits:

```tinys
def square_positive(value: i32) -> Result[i32, Error]:
    if value < 0:
        return Err(NegativeValue)

    Ok(value * value)
```

---

## 26. Final expressions in blocks

A block’s final expression may determine its value.

```tinys
value = if condition:
    first_result()
else:
    second_result()
```

Likewise:

```tinys
value = match result:
    case Ok(number):
        number

    case Err(_):
        0
```

And:

```tinys
value = loop:
    candidate = next_candidate()

    if candidate.is_valid():
        break candidate
```

This is central to preserving Rust’s expression-oriented model.

---

## 27. Empty blocks

An empty block uses `pass`.

```tinys
if debug_enabled:
    pass
```

For matching:

```tinys
match token:
    case Token.End:
        pass

    case _:
        process(token)
```

`pass` generates no operation and has type `void`.

---

## 28. Assertions

Assertions are macros rather than special control-flow syntax.

```tinys
from macro import assert

assert(index < values.len())
```

Optional messages:

```tinys
assert(index < values.len(), "index out of bounds")
```

Generated Rust:

```rust
assert!(index < values.len(), "index out of bounds");
```

---

## 29. Panic and unreachable paths

Rust-style terminal operations may be imported as macros.

```tinys
from macro import panic, unreachable

if state.is_corrupt():
    panic("corrupt state")
```

```tinys
match token:
    case Token.Identifier(name):
        process(name)

    case _:
        unreachable()
```

These expressions have the never type and may satisfy branches requiring another type.

---

## 30. The never type

TinyS may expose Rust’s never type as `never`.

```tinys
def fail(message: ref str) -> never:
    panic(message)
```

Generated Rust:

```rust
fn fail(message: &str) -> ! {
    panic!("{}", message)
}
```

Functions returning `never` do not return normally.

This is useful for:

* panic helpers;
* process termination;
* infinite loops;
* impossible branches.

---

## 31. Recommended core control-flow syntax

```tinys
if condition:
    ...
elif other_condition:
    ...
else:
    ...
```

```tinys
value = if condition:
    first
else:
    second
```

```tinys
match expression:
    case pattern:
        ...
    case pattern if guard:
        ...
    case _:
        ...
```

```tinys
while condition:
    ...
```

```tinys
while case pattern = expression:
    ...
```

```tinys
for item in iterable:
    ...
```

```tinys
for item in ref collection:
    ...
```

```tinys
for item in mut ref collection:
    ...
```

```tinys
loop:
    ...
```

```tinys
break
break value
continue
return
return value
```

Labeled control flow:

```tinys
loop as outer:
    ...
    break outer
    continue outer
    break outer with value
```

---

## 32. Design principle

TinyS should borrow the visual structure of Python while preserving the semantic strength of Rust:

* conditions require `bool`;
* `if`, `match`, and `loop` may produce values;
* ownership rules apply during iteration;
* borrowing is explicit through `ref` and `mut ref`;
* pattern matching is exhaustive;
* loops may return values;
* early returns remain explicit;
* iterator behavior follows Rust;
* no Python-style implicit truthiness is introduced.

The syntax should feel familiar to Python developers without weakening Rust’s type, ownership, or control-flow model.
