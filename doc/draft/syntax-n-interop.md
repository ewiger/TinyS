# TinyS Syntax and Interoperability Summary

## 1. Compilation pipeline

```text
.sn source → generated .rs → rustc/Cargo → native binary
```

TinyS uses Python-shaped syntax with Rust-oriented semantics.

---

## 2. Indentation-based blocks

Blocks are defined by indentation rather than braces.

```tinys
def maximum(left: i32, right: i32) -> i32:
    if left >= right:
        return left

    return right
```

---

## 3. Functions

Functions use `def` and require explicit parameter and return types.

```tinys
def add(left: i64, right: i64) -> i64:
    return left + right
```

---

## 4. Generic types and functions

Generic parameters use square brackets.

```tinys
list[i32]
dict[str, User]
Result[User, Error]
Option[str]
```

Generic functions use the same syntax:

```tinys
def identity[T](value: T) -> T:
    return value
```

Trait bounds remain explicit:

```tinys
def copy_value[T: Copy](value: T) -> T:
    return value
```

---

## 5. Lifetimes

Lifetime names begin with a dot.

```tinys
.a
.source
.store
```

They are declared alongside type parameters:

```tinys
def longest[.a](
    left: ref[.a] str,
    right: ref[.a] str,
) -> ref[.a] str:
    ...
```

This generates Rust lifetime syntax such as:

```rust
'a
```

---

## 6. Lifetime-parameterized types

Types parameterized by lifetimes use square brackets.

```tinys
ConfigView[.store]
Parser[.source]
Window[.buffer]
```

Example:

```tinys
struct ConfigView[.store]:
    primary: ref[.store] str
    fallback: ref[.store] str
```

---

## 7. Shared references

Shared borrowing uses `ref`.

```tinys
ref T
ref[.a] T
```

Examples:

```tinys
value: ref i32
name: ref[.source] str
```

These generate Rust references such as:

```rust
&T
&'a T
```

---

## 8. Mutable references

Exclusive mutable borrowing uses `mut ref`.

```tinys
mut ref T
mut ref[.a] T
```

Example:

```tinys
def sort(values: mut ref list[i32]) -> void:
    values.sort()
```

This generates a Rust mutable reference:

```rust
&mut Vec<i32>
```

---

## 9. Owned values

Values are owned by default.

```tinys
def consume(value: Data) -> void:
    ...
```

Passing an owned non-`Copy` value transfers ownership according to Rust semantics.

An optional `own` annotation may be supported when explicit documentation is useful:

```tinys
def consume(value: own Data) -> void:
    ...
```

`own` does not introduce a different runtime representation. It only makes ownership explicit in source code.

---

## 10. Explicit ownership transfer

Ownership transfer may be emphasized with `move`.

```tinys
consume(move data)
```

After the move, `data` is unavailable.

The compiler should also infer ordinary Rust moves when no explicit `move` keyword is present.

---

## 11. Explicit dereferencing

Explicit dereferencing uses `at`.

```tinys
copied: i32 = at source
```

Example:

```tinys
def copy_into[T: Copy](
    target: mut ref T,
    source: ref T,
) -> void:
    at target = at source
```

Generated Rust:

```rust
*target = *source;
```

Using `at target` on the assignment side makes the destination dereference explicit and avoids treating `target` as if the reference itself were being reassigned.

---

## 12. Automatic dereferencing

Normal field access and method calls automatically dereference references when Rust would do so.

```tinys
user: ref User

print(user.name)
user.display()
```

Explicit `at` is reserved for cases where the referred value itself must be read, copied, moved, or assigned through.

---

## 13. Dereferencing does not clone

`at` never implicitly clones a value.

For a `Copy` type:

```tinys
number: i32 = at number_ref
```

For a heap-owned value, cloning must be explicit:

```tinys
text: str = clone text_ref
```

Moving out of a shared reference remains invalid.

---

## 14. Structs

Structs use `struct` with indentation-based fields.

```tinys
struct Point:
    x: f64
    y: f64
```

---

## 15. Implementations and methods

Methods are declared inside `impl` blocks.

```tinys
impl Point:

    def length(self: ref Self) -> f64:
        return sqrt(
            self.x * self.x +
            self.y * self.y
        )
```

The receiver type expresses ownership explicitly:

```tinys
self: Self
self: ref Self
self: mut ref Self
```

These correspond approximately to:

```rust
self
&self
&mut self
```

---

## 16. Traits

Traits retain Rust-like semantics while using indentation.

```tinys
trait Display:
    def display(self: ref Self) -> str
```

Trait implementation:

```tinys
impl Display for Point:

    def display(self: ref Self) -> str:
        return format("Point({}, {})", self.x, self.y)
```

---

## 17. Enums

Enums are algebraic data types.

```tinys
enum Token[.source]:
    Identifier(ref[.source] str)
    Number(i64)
    Plus
    End
```

Variants may contain zero or more typed fields.

---

## 18. Pattern matching

Pattern matching uses `match`.

```tinys
match token:
    Token.Identifier(name):
        print(name)

    Token.Number(value):
        print(value)

    Token.Plus:
        print("+")

    Token.End:
        pass
```

---

## 19. Rust attributes

Rust attributes remain unchanged.

```tinys
#[derive(Debug, Clone, PartialEq)]
struct User:
    id: u64
    name: str
```

Other examples:

```tinys
#[test]
#[inline]
#[repr(C)]
#[cfg(target_os = "linux")]
```

Attributes are already concise, declarative, and visually similar to comments, so TinyS does not replace them.

---

## 20. Macro imports

Macros are imported explicitly.

```tinys
from macro import assert, debug, format
from macro.std import vec
```

Macro calls do not use an exclamation mark:

```tinys
debug(user)
assert(user.id > 0)

values = vec(1, 2, 3)
message = format("Hello {}", user.name)
```

These may generate Rust macro calls such as:

```rust
dbg!(user);
assert!(user.id > 0);
vec![1, 2, 3];
format!("Hello {}", user.name);
```

The import binding determines that the called name refers to a macro rather than a function.

---

## 21. Macro aliases

Macro imports support aliases.

```tinys
from macro import debug as dbg
from macro import assert as require
from macro.std import vec as list_of
```

Usage:

```tinys
dbg(user)
require(user.id > 0)
values = list_of(1, 2, 3)
```

---

## 22. TinyS module imports

Native TinyS modules use ordinary Python-style imports.

```tinys
from models import User
import services.database as database
```

These refer to TinyS modules within the current package or workspace.

---

## 23. Rust crate imports

Rust crates and modules are accessed through the explicit `rust` root.

```tinys
from rust.regex import Regex
from rust.serde import Serialize, Deserialize
from rust.std.collections import HashMap
```

Module aliasing is supported:

```tinys
import rust.serde_json as json
```

The `rust` prefix makes the interoperability boundary visible.

---

## 24. Rust generic calls

Explicit generic arguments in calls use square brackets.

```tinys
user = json.from_str[User](source)?
```

Generated Rust:

```rust
let user = serde_json::from_str::<User>(source)?;
```

This avoids Rust’s turbofish syntax while retaining the same meaning.

---

## 25. Associated functions

Rust associated functions use ordinary dot syntax.

```tinys
pattern = Regex.new(expression)?
mapping = HashMap.new()
path = PathBuf.from(source)
```

The transpiler determines when the Rust output requires `::`.

Generated forms may include:

```rust
Regex::new(expression)?
HashMap::new()
PathBuf::from(source)
```

Instance methods use the same surface syntax:

```tinys
pattern.is_match(source)
```

The distinction is resolved from type information.

---

## 26. Crate-specific macros

Macros exported by Rust crates use the `macro` namespace.

```tinys
from macro.serde_json import json
from macro.regex import regex
```

Usage:

```tinys
document = json(
    name="Ada",
    active=true,
)

pattern = regex(r"^[a-z]+$")
```

These generate the corresponding crate macro invocations.

---

## 27. Cargo dependencies

Dependencies are declared in `tinys.toml`.

```toml
[package]
name = "example"
version = "0.1.0"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
regex = "1"
```

TinyS generates or manages the corresponding `Cargo.toml`.

The initial design should keep dependency declarations close to Cargo semantics so that existing Rust documentation remains applicable.

---

## 28. Explicit Rust interoperability

Rust interoperability is always syntactically visible.

```tinys
from rust.some_crate import SomeType
```

This distinguishes:

* native TinyS modules;
* imported Rust types and functions;
* imported Rust macros;
* generated Rust implementation details.

TinyS should integrate closely with Rust without pretending that the distinction does not exist.

---

## 29. Error handling

Error handling follows Rust’s `Result` model.

```tinys
def parse(source: ref str) -> Result[User, Error]:
    return json.from_str[User](source)
```

Error propagation uses `?`.

```tinys
user = parse(source)?
```

TinyS does not initially replace `Result`, `Option`, or `?` with exception-based semantics.

---

## 30. Central design principle

TinyS combines:

```text
Python-style layout and readability
+
Rust ownership, borrowing, lifetimes, traits, enums, error handling,
native compilation, and Cargo ecosystem
-
most C-style pointer punctuation and unnecessary syntactic ceremony
```

The guiding rule is:

> Simplify Rust syntax where the replacement improves clarity, but preserve Rust constructs directly where the existing syntax is already concise, expressive, and interoperable.

TinyS should remain understandable to Rust developers, while being substantially easier to read and write for developers accustomed to Python-shaped languages.
