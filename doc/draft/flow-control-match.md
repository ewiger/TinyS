Python added structural pattern matching in **Python 3.10** using `match` and `case`. That maps very naturally to Rust’s `match`, so TinyS should probably follow Python’s surface syntax here.

A TinyS enum:

```tinys
enum Token[.source]:
    Identifier(ref[.source] str)
    Number(i64)
    Plus
    End
```

Could be matched as:

```tinys
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

This is clearer than omitting `case`, because each branch is visibly distinct from an ordinary nested block.

Generated Rust:

```rust
match token {
    Token::Identifier(name) => {
        println!("{}", name);
    }

    Token::Number(value) => {
        println!("{}", value);
    }

    Token::Plus => {
        println!("+");
    }

    Token::End => {}
}
```

## Wildcards

Python uses `_`, and Rust does too:

```tinys
match token:
    case Token.Identifier(name):
        print(name)

    case _:
        print("other")
```

## Multiple alternatives

Python uses `|`, which also matches Rust:

```tinys
match token:
    case Token.Plus | Token.Minus:
        print("operator")

    case _:
        pass
```

## Guards

Python supports `if` guards after the pattern:

```tinys
match value:
    case Number(number) if number > 0:
        print("positive")

    case Number(number):
        print("zero or negative")
```

Generated Rust:

```rust
match value {
    Number(number) if number > 0 => {
        println!("positive");
    }

    Number(number) => {
        println!("zero or negative");
    }
}
```

## Struct destructuring

Python’s matching syntax can also work well for TinyS structs:

```tinys
match point:
    case Point(x=0.0, y=0.0):
        print("origin")

    case Point(x=x, y=0.0):
        print("on x-axis:", x)

    case Point(x=x, y=y):
        print(x, y)
```

However, this is somewhat verbose. TinyS could also support Rust-like shorthand:

```tinys
match point:
    case Point(x=0.0, y=0.0):
        print("origin")

    case Point(x, 0.0):
        print("on x-axis:", x)

    case Point(x, y):
        print(x, y)
```

The second version only works cleanly when field order is part of the type’s public definition. Named matching is safer and more readable.

## Destructuring sequences

For list or slice matching:

```tinys
match values:
    case []:
        print("empty")

    case [first]:
        print("one value:", first)

    case [first, second]:
        print("two values")

    case [first, *rest]:
        print(first, rest)
```

But Rust slice patterns use `..`, not Python’s starred capture. TinyS has a design choice:

```tinys
case [first, *rest]:
```

or:

```tinys
case [first, ..rest]:
```

Given the Python-shaped design, `*rest` is probably better at the TinyS level.

## Matching `Option`

```tinys
match user:
    case Some(value):
        print(value)

    case None:
        print("not found")
```

Or, preserving the qualified Rust type:

```tinys
match user:
    case Option.Some(value):
        print(value)

    case Option.None:
        print("not found")
```

The shorter form is attractive, but it creates possible ambiguity. I would prefer Rust-compatible imported variant names:

```tinys
from rust.std.option.Option import Some, None
```

Although `None` is already likely to be a TinyS keyword, so a better language-level representation may be:

```tinys
match user:
    case Some(value):
        print(value)

    case none:
        print("not found")
```

## Matching `Result`

```tinys
match result:
    case Ok(value):
        print(value)

    case Err(error):
        print(error)
```

This is almost identical to idiomatic Rust, apart from indentation and `case`.

## Binding the complete matched value

Rust supports `name @ pattern`. Since TinyS already uses `at` for dereferencing, reusing `at` here would be confusing.

Instead, TinyS could use `as`, matching Python:

```tinys
match token:
    case Token.Number(value) as complete:
        debug(complete)
        print(value)
```

Generated Rust:

```rust
match token {
    complete @ Token::Number(value) => {
        dbg!(complete);
        println!("{}", value);
    }
}
```

That keeps `at` exclusively for dereferencing.

## Match as an expression

Rust’s `match` produces a value, while Python’s `match` is only a statement. TinyS should preserve Rust semantics:

```tinys
description: str = match token:
    case Token.Identifier(_):
        "identifier"

    case Token.Number(_):
        "number"

    case Token.Plus:
        "operator"

    case Token.End:
        "end"
```

This would generate:

```rust
let description: String = match token {
    Token::Identifier(_) => "identifier".into(),
    Token::Number(_) => "number".into(),
    Token::Plus => "operator".into(),
    Token::End => "end".into(),
};
```

For consistency, branch bodies could use their final expression as the result:

```tinys
value = match result:
    case Ok(number):
        number * 2

    case Err(error):
        return Err(error)
```

## Recommended TinyS syntax

I would revise the earlier rule to:

```tinys
match expression:
    case pattern:
        ...

    case pattern if condition:
        ...

    case pattern1 | pattern2:
        ...

    case _:
        ...
```

So the final design principle becomes:

* `match` introduces structural pattern matching.
* Every branch begins with `case`.
* `_` is the wildcard.
* `|` combines alternative patterns.
* `if` introduces a guard.
* `as` binds the complete matched value.
* `match` is an expression, unlike Python’s version.
* Exhaustiveness checking follows Rust, not Python.

That last point is important: TinyS can borrow Python’s syntax while preserving Rust’s compiler guarantees.
