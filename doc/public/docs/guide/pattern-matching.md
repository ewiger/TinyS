# Pattern matching

TinyS uses Python-shaped structural matching with Rust semantics. `match` is an
**expression** and must be **exhaustive** — every possible value has to be
covered, which the compiler checks for you.

## Basic match

```python
match token:
    case Token.Identifier(name):
        print(name)

    case Token.Number(value):
        print(value)

    case Token.Plus:
        print("+")

    case Token.End:
        pass
```

Each `case` names a pattern; matching a variant with a payload **binds** its
fields to the names you give.

## match is an expression

Because `match` produces a value, you can assign its result directly:

```python
description = match token:
    case Token.Identifier(_):
        "identifier"

    case Token.Number(_):
        "number"

    case Token.Plus:
        "operator"

    case Token.End:
        "end"
```

Every arm must produce the same type, and — since the match is exhaustive — adding
a new enum variant without handling it becomes a compile error.

## Wildcards

`_` matches anything and ignores it. Use it to bind-and-ignore a payload, or as a
catch-all arm:

```python
case Token.Rectangle(_, _):
    "rectangle"

case _:
    "something else"
```

## Alternative patterns

Match several patterns in one arm with `|`:

```python
case Token.Plus | Token.Minus:
    "operator"
```

## Guards

Add a boolean condition to an arm with `if`:

```python
case Token.Number(value) if value > 0:
    "positive number"
```

## Binding the whole value — `as`

Bind the entire matched value (in addition to destructuring it) with `as`:

```python
case Token.Number(value) as complete:
    debug(complete)
```

## Matching options

`Option` participates in matching like any other enum. Absence is `none`, never
`null`:

```python
match user:
    case Some(value):
        print(value)

    case none:
        print("not found")
```

See [Error handling](error-handling.md) for `Result` and `Option` in depth.

## Pattern conditions — `if case` and `while case`

Rust's `if let` and `while let` become `if case` and `while case`:

```python
if case Some(user) = find_user(user_id):
    print(user.name)
else:
    print("not found")
```

```python
while case Some(item) = queue.pop():
    process(item)
```

See [Control flow](control-flow.md) for the loop forms.

## Generated Rust

```python
description = match shape:
    case Shape.Circle(_):
        "circle"

    case Shape.Empty:
        "nothing"
```

```rust
let description = match shape {
    Shape::Circle(_) => "circle",
    Shape::Empty => "nothing",
};
```

## Where to go next

- [Structs & enums](structs-and-enums.md) — the types you match on.
- [Error handling](error-handling.md) — `Result` / `Option` patterns.
- [Control flow](control-flow.md) — `if case` and `while case`.
