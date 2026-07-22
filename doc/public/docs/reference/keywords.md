# Keywords

The TinyS keyword vocabulary, grouped by purpose, with the Rust concept each maps
to.

## Declarations

| Keyword  | Purpose                                   | Rust           |
| -------- | ----------------------------------------- | -------------- |
| `def`    | function / method definition              | `fn`           |
| `struct` | record type                               | `struct`       |
| `enum`   | algebraic data type                       | `enum`         |
| `trait`  | shared-behavior interface                 | `trait`        |
| `impl`   | inherent / trait implementation           | `impl`         |
| `pub`    | public visibility                         | `pub`          |
| `mut`    | mutable binding / receiver                | `mut`          |
| `async`  | asynchronous function                     | `async`        |
| `unsafe` | unsafe block or function                  | `unsafe`       |
| `fn`     | closure literal                           | `\| … \|`      |

## Ownership & references

| Keyword  | Purpose                             | Rust     |
| -------- | ----------------------------------- | -------- |
| `ref`    | shared borrow (value and type)      | `&`      |
| `mut ref`| mutable borrow (value and type)     | `&mut`   |
| `at`     | dereference                         | `*`      |
| `move`   | emphasize ownership transfer        | `move`   |
| `clone`  | explicit deep copy                  | `.clone()` |
| `own`    | ownership annotation (documentation)| (owned)  |
| `Self`   | the implementing type               | `Self`   |
| `self`   | method receiver                     | `self`   |

## Control flow

| Keyword    | Purpose                          | Rust        |
| ---------- | -------------------------------- | ----------- |
| `if`       | conditional                      | `if`        |
| `elif`     | else-if                          | `else if`   |
| `else`     | fallback branch                  | `else`      |
| `match`    | pattern match (expression)       | `match`     |
| `case`     | a match arm                      | arm `=>`    |
| `for`      | iteration                        | `for`       |
| `in`       | iterator source in `for`         | `in`        |
| `while`    | conditional loop                 | `while`     |
| `loop`     | infinite loop (expression)       | `loop`      |
| `break`    | exit a loop                      | `break`     |
| `continue` | next iteration                   | `continue`  |
| `return`   | return from a function           | `return`    |
| `as`       | loop label / match binding / alias | `'label`, `@`, `as` |
| `with`     | value carried by a labeled break | `break … value` |
| `pass`     | empty block                      | (empty)     |

## Pattern conditions

| Keyword     | Purpose                       | Rust        |
| ----------- | ----------------------------- | ----------- |
| `if case`   | conditional pattern binding   | `if let`    |
| `while case`| loop while a pattern matches  | `while let` |
| `_`         | wildcard pattern              | `_`         |
| `\|`        | alternative patterns          | `\|`        |

## Logical operators

| Keyword | Rust  |
| ------- | ----- |
| `and`   | `&&`  |
| `or`    | `\|\|`|
| `not`   | `!`   |

## Values & types

| Keyword         | Purpose                     | Rust        |
| --------------- | --------------------------- | ----------- |
| `true` / `false`| boolean literals            | `true`/`false` |
| `none`          | absence (with `Option`)     | `None`      |
| `Some`          | present optional value      | `Some`      |
| `Ok` / `Err`    | result variants             | `Ok`/`Err`  |
| `void`          | unit / no return value      | `()`        |

## Imports

| Keyword  | Purpose                          | Rust    |
| -------- | -------------------------------- | ------- |
| `import` | import a module                  | `use`   |
| `from`   | import names from a module       | `use`   |
| `rust`   | import root for Rust crates      | (crate) |
| `macro`  | import root for macros           | (macro) |

## Primitive type names

TinyS uses Rust's primitive type names directly:

```text
i8  i16  i32  i64  i128  isize
u8  u16  u32  u64  u128  usize
f32 f64
bool  char  str
```

See the [cheat sheet](cheatsheet.md) for the full surface-syntax mapping.
