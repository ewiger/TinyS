# TinyS

TinyS is a small statically typed programming language with Python-shaped syntax and Rust-oriented semantics.

```text
.sn source → generated Rust → rustc/Cargo → native binary
```

TinyS aims to make systems programming more readable without weakening the ownership, borrowing, type-safety, and interoperability guarantees that make Rust valuable.

> 📖 **[Read the TinyS language manual →](https://ewiger.github.io/TinyS/)**
> — the complete language reference, tutorial, syntax guide, and examples,
> published as a static site on GitHub Pages. Source lives in [doc/public/](doc/public/).

```tinys
def maximum(left: i32, right: i32) -> i32:
    if left >= right:
        return left

    return right
```

## Installation

Install the TinyS CLI from [crates.io](https://crates.io/crates/tinys):

```bash
cargo install tinys
```

Or add TinyS as a dependency to a Rust project:

```bash
cargo add tinys
```

## Project status

TinyS is an experimental language design and compiler project.

A working **v0.1.0 compiler** now lives in [src/](src/): an indentation-aware
lexer, recursive-descent parser, and Rust source generator, driven by a `tinys`
CLI. It covers the Phase 1 core plus a good slice of Phase 2 — functions,
ownership/borrowing (`ref`/`mut ref`/`at`/`move`/`clone`), structs, enums,
exhaustive `match`, traits and `impl`, generics, closures, `Result`/`Option`
with `?`, expression-oriented `if`/`match`/`loop`, and Rust interop through the
`rust`/`macro` import roots. Runnable programs live in [examples/](examples/).

```bash
cargo build                                # build the tinys compiler
cargo run -- run    examples/fizzbuzz.sn   # transpile → cargo → run
cargo run -- emit-rust examples/hello.sn   # inspect the generated Rust
cargo test                                 # lexer, codegen, and end-to-end tests
```

The syntax and semantics are still evolving, and TinyS is not yet ready for
production use. Each `.sn` file is compiled as its own Cargo package, with
dependencies resolved from the nearest `tinys.toml`; multi-file module discovery
is still on the roadmap.

## Design goals

TinyS combines:

```text
Python-style layout and readability
+
Rust ownership, borrowing, lifetimes, traits, enums, error handling,
native compilation, and Cargo interoperability
-
most C-style pointer punctuation and unnecessary syntactic ceremony
```

The central rule is:

> Simplify Rust syntax where the replacement improves clarity, while preserving Rust constructs directly where the existing syntax is already concise and expressive.

TinyS is not intended to hide Rust semantics.

It should remain clear when values are owned, borrowed, moved, cloned, mutated, or passed into external Rust code.

## Non-goals

TinyS is not intended to be:

* a dynamically typed Python implementation;
* a garbage-collected language;
* a wrapper around Python;
* a language with implicit null values;
* a replacement for Cargo or the Rust ecosystem;
* a simplified language that removes ownership or lifetime correctness;
* a source-compatible subset of Python.

TinyS borrows Python’s visual structure, not Python’s runtime model.

## Compilation model

TinyS compiles through Rust:

```text
source.sn
    ↓
TinyS parser and semantic analysis
    ↓
generated source.rs
    ↓
rustc and Cargo
    ↓
native executable or Rust library
```

The generated Rust should be:

* deterministic;
* readable;
* inspectable;
* compatible with standard Rust tooling;
* mapped back to the original `.sn` source for diagnostics.

A future compiler command may expose the generated source directly:

```text
tinys emit-rust
```

## Hello world

```tinys
def main() -> void:
    print("Hello from TinyS")
```

Conceptual generated Rust:

```rust
fn main() {
    println!("Hello from TinyS");
}
```

## Functions

Functions use `def` with explicit parameter and return types.

```tinys
def add(left: i64, right: i64) -> i64:
    return left + right
```

The final expression of a function may also become its return value:

```tinys
def square(value: i32) -> i32:
    value * value
```

## Variables and mutability

Values are immutable by default.

```tinys
name = "Ada"
count: i32 = 0
```

Mutable variables are declared explicitly:

```tinys
mut total: i64 = 0

total += 1
```

TinyS follows Rust-style ownership and mutability rather than Python-style unrestricted reassignment.

## Owned values

Values are owned by default.

```tinys
def consume(value: Data) -> void:
    process(value)
```

Passing an owned non-`Copy` value transfers ownership when Rust would do so.

Ownership transfer may be emphasized explicitly:

```tinys
consume(move data)
```

After the move, `data` is unavailable.

An optional `own` type annotation may be supported for documentation:

```tinys
def consume(value: own Data) -> void:
    ...
```

## References

Shared borrowing uses `ref`.

```tinys
value: ref i32
name: ref str
```

A reference is created with:

```tinys
value_ref = ref value
```

This maps conceptually to Rust’s `&value`.

Mutable borrowing uses `mut ref`.

```tinys
def increment(value: mut ref i32) -> void:
    at value += 1
```

A mutable reference is created with:

```tinys
value_ref = mut ref value
```

This maps conceptually to Rust’s `&mut value`.

## Dereferencing

Explicit dereferencing uses `at`.

```tinys
copied: i32 = at source
```

Assignment through a mutable reference is also explicit:

```tinys
at target = at source
```

Generated Rust:

```rust
*target = *source;
```

Ordinary field access and method calls use automatic dereferencing where Rust would do so:

```tinys
user: ref User

print(user.name)
user.display()
```

`at` never implicitly clones.

```tinys
number: i32 = at number_ref
text: str = clone text_ref
```

## Lifetimes

Lifetime names begin with a dot.

```tinys
.a
.source
.store
```

They are declared alongside generic parameters:

```tinys
def longest[.a](
    left: ref[.a] str,
    right: ref[.a] str,
) -> ref[.a] str:
    ...
```

This generates Rust lifetime syntax such as `'a`.

Lifetime-parameterized types use square brackets:

```tinys
struct ConfigView[.store]:
    primary: ref[.store] str
    fallback: ref[.store] str
```

## Generic types and functions

Generic types use square brackets:

```tinys
list[i32]
dict[str, User]
Option[str]
Result[User, Error]
```

Generic functions follow the same form:

```tinys
def identity[T](value: T) -> T:
    return value
```

Trait bounds are explicit:

```tinys
def clone_value[T: Clone](value: ref T) -> T:
    return value.clone()
```

## Strings

The intended core distinction is:

```tinys
str
```

for an owned string, corresponding roughly to Rust `String`, and:

```tinys
ref str
```

for a borrowed string slice, corresponding to Rust `&str`.

Example:

```tinys
name: str = "Ada"
view: ref str = ref name
```

String literal inference and conversion rules are still being finalized.

## Structs

```tinys
struct Point:
    x: f64
    y: f64
```

Construction uses keyword-style fields:

```tinys
point = Point(
    x=10.0,
    y=20.0,
)
```

## Methods and implementations

```tinys
impl Point:

    def length(self: ref Self) -> f64:
        return sqrt(
            self.x * self.x +
            self.y * self.y
        )
```

Receiver types express ownership explicitly:

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

Associated functions use ordinary dot syntax:

```tinys
point = Point.new(10.0, 20.0)
mapping = HashMap.new()
```

The compiler emits Rust `::` where required.

## Traits

```tinys
trait Display:
    def display(self: ref Self) -> str
```

Implementation:

```tinys
impl Display for Point:

    def display(self: ref Self) -> str:
        return format("Point({}, {})", self.x, self.y)
```

Default methods are supported conceptually:

```tinys
trait Display:

    def display(self: ref Self) -> str

    def debug_display(self: ref Self) -> str:
        return self.display()
```

## Enums

TinyS enums are algebraic data types.

```tinys
enum Token[.source]:
    Identifier(ref[.source] str)
    Number(i64)
    Plus
    End
```

Construction:

```tinys
token = Token.Number(42)
end = Token.End
```

## Pattern matching

TinyS uses Python-shaped structural matching with Rust semantics.

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

Wildcard:

```tinys
case _:
    ...
```

Alternative patterns:

```tinys
case Token.Plus | Token.Minus:
    ...
```

Guards:

```tinys
case Token.Number(value) if value > 0:
    ...
```

Binding the entire matched value uses `as`:

```tinys
case Token.Number(value) as complete:
    debug(complete)
```

TinyS `match` is an expression and must be exhaustive.

```tinys
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

## Conditional flow

```tinys
if temperature > 30:
    print("hot")
elif temperature > 20:
    print("warm")
else:
    print("cold")
```

Conditions must have type `bool`.

TinyS does not use Python-style truthiness.

```tinys
if not values.is_empty():
    process(values)
```

An `if` block may produce a value:

```tinys
status = if active:
    "active"
else:
    "inactive"
```

## Pattern conditions

Rust’s `if let` is represented using `if case`.

```tinys
if case Some(user) = find_user(user_id):
    print(user.name)
else:
    print("not found")
```

Rust’s `while let` becomes:

```tinys
while case Some(item) = queue.pop():
    process(item)
```

## Loops

Infinite loop:

```tinys
loop:
    process_next_item()
```

Conditional loop:

```tinys
while count > 0:
    count -= 1
```

Iteration:

```tinys
for user in users:
    process(user)
```

Iterating over the owned collection may consume it.

Shared iteration is explicit:

```tinys
for user in ref users:
    print(user.name)
```

Mutable iteration is explicit:

```tinys
for user in mut ref users:
    user.active = true
```

Ranges follow Rust semantics:

```tinys
for index in 0..10:
    print(index)
```

Inclusive range:

```tinys
for index in 0..=10:
    print(index)
```

## Loop control

```tinys
break
continue
return
```

Loops may produce values:

```tinys
result = loop:
    value = read_value()

    if value >= 0:
        break value
```

Labeled loops use `as`:

```tinys
loop as search:
    for item in items:
        if item.matches():
            break search
```

A labeled break that returns a value uses `with`:

```tinys
result = loop as search:
    for item in items:
        if item.matches():
            break search with item
```

## Error handling

TinyS follows Rust’s `Result` and `Option` model.

```tinys
def parse(source: ref str) -> Result[User, Error]:
    return json.from_str[User](source)
```

Error propagation uses `?`.

```tinys
user = parse(source)?
```

Absence is represented with `Option[T]`, not `null`.

```tinys
user = Some(value)
user = none
```

Matching:

```tinys
match user:
    case Some(value):
        print(value)

    case none:
        print("not found")
```

TinyS does not use exceptions for ordinary recoverable errors.

## Collections

Planned standard collection names:

```tinys
list[T]
array[T, N]
slice[T]
dict[K, V]
set[T]
```

Approximate Rust mappings:

```text
list[T]       → Vec<T>
array[T, N]   → [T; N]
slice[T]      → [T]
dict[K, V]    → HashMap<K, V>
set[T]        → HashSet<T>
```

Examples:

```tinys
numbers: list[i32] = [1, 2, 3]
mapping: dict[str, i32] = {
    "one": 1,
    "two": 2,
}
```

Indexing follows Rust-style bounds behavior:

```tinys
value = values[index]
```

Safe lookup uses collection methods:

```tinys
value = values.get(index)
```

## Tuples

```tinys
pair: (str, i32) = ("Ada", 42)
```

Destructuring:

```tinys
name, age = pair
```

Tuple field access:

```tinys
name = pair.0
age = pair.1
```

## Closures

The proposed closure syntax uses `fn`.

```tinys
double = fn(value: i32) -> i32:
    value * 2
```

Multiline closure:

```tinys
transform = fn(value: i32) -> i32:
    adjusted = value * 2
    adjusted + 1
```

Move closures reuse the `move` keyword:

```tinys
worker = move fn():
    process(data)
```

## Async code

Async functions retain familiar syntax:

```tinys
async def fetch_user(id: u64) -> Result[User, Error]:
    response = client.get(id).await?
    return response.json[User]().await
```

Postfix `.await` is preserved because it composes naturally with Rust-style method calls and `?`.

Async runtimes remain Rust-library concerns:

```tinys
#[tokio::main]
async def main() -> Result[void, Error]:
    run().await?
    return Ok()
```

## Attributes

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

Attributes are already concise and interoperable, so TinyS does not replace them.

## Macros

Macros are imported explicitly.

```tinys
from macro import assert, debug, format
from macro.std import vec
```

Calls do not use an exclamation mark:

```tinys
debug(user)
assert(user.id > 0)

values = vec(1, 2, 3)
message = format("Hello {}", user.name)
```

These generate Rust macro invocations such as:

```rust
dbg!(user);
assert!(user.id > 0);
vec![1, 2, 3];
format!("Hello {}", user.name);
```

Aliases are supported; the alias becomes the call-site name:

```tinys
from macro import debug as dbg
from macro import assert as require
```

`macro` and `macro.std` name the prelude and std macros, which are callable
unqualified. Any other root is a crate namespace, and the generated call is
qualified with it:

```tinys
from macro.serde_json import json
from macro.regex import regex
```

```rust
serde_json::json!(...)
regex::regex!(...)
```

The `macro` root is routing only — it never appears in the generated Rust, and
no `use` line is emitted for it. A small prelude (`print`, `format`, `debug`,
`assert`, `assert_eq`, `panic`, `vec`) is in scope without an import; importing
explicitly is what enables aliases and crate macros. See
[examples/macros.sn](examples/macros.sn).

Crate-specific macros use the macro namespace:

```tinys
from macro.serde_json import json
from macro.regex import regex
```

Custom user-defined TinyS macros are outside the initial core language scope.

## Modules and imports

Native TinyS modules use Python-style imports.

```tinys
from models import User
import services.database as database
```

Rust crates and modules use the explicit `rust` root.

```tinys
from rust.regex import Regex
from rust.serde import Serialize, Deserialize
from rust.std.collections import HashMap
```

Module aliases:

```tinys
import rust.serde_json as json
```

Rust generic calls use square brackets:

```tinys
user = json.from_str[User](source)?
```

Generated Rust:

```rust
let user = serde_json::from_str::<User>(source)?;
```

Rust interoperability remains visible by design.

## Visibility

Public declarations use `pub`.

```tinys
pub struct User:
    pub id: u64
    name: str
```

```tinys
pub def load_user(id: u64) -> Result[User, Error]:
    ...
```

More restricted visibility may use square brackets:

```tinys
pub[crate] def helper() -> void:
    ...
```

## Dependencies

Cargo dependencies are declared in `tinys.toml`.

```toml
[package]
name = "example"
version = "0.1.0"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
regex = "1"
```

TinyS generates and manages the corresponding `Cargo.toml`: `build`, `run` and
`check` look for the nearest `tinys.toml` above the source file, wrap the
generated Rust in a scratch Cargo package under `target/tinys-generated/`, and
drive `cargo`. Only the crates a program imports are carried into the generated
manifest, so dependency-free programs still build without a network.

The dependency model should remain close to Cargo so that existing Rust documentation and tooling stay useful.

## Proposed project layout

```text
example/
├── tinys.toml
├── src/
│   ├── main.sn
│   ├── models.sn
│   └── services/
│       ├── mod.sn
│       └── database.sn
└── target/
    └── tinys-generated/
        └── ...
```

One TinyS package maps approximately to one Cargo package.

Applications generate `main.rs`. Libraries generate `lib.rs`.

## Unsafe code

Unsafe operations remain explicit.

```tinys
unsafe:
    perform_raw_operation()
```

Unsafe functions:

```tinys
unsafe def read_address(address: usize) -> u8:
    ...
```

Raw pointers and foreign-function interfaces are intentionally postponed until the core ownership and Rust interoperability model is stable.

## Comments and documentation

TinyS keeps Rust-compatible comment syntax.

```tinys
// Ordinary comment

/// Public API documentation
pub def load_user(id: u64) -> Option[User]:
    ...
```

Module documentation may use:

```tinys
//! User-service module.
```

Using `//` avoids ambiguity with Rust-style attributes beginning with `#`.

## Command-line interface

```text
tinys build     <file.sn>   implemented — generate Rust and compile a binary
tinys run       <file.sn>   implemented — build and run an application
tinys check     <file.sn>   implemented — parse and type-check via `cargo check`
tinys emit-rust <file.sn>   implemented — expose generated Rust for inspection
tinys version               implemented — print the compiler version
tinys test                  planned     — run tests through Cargo
tinys fmt                   planned     — format `.sn` source files
```

Responsibilities:

* `build` — generate Rust and compile the package;
* `run` — build and run an application;
* `check` — perform TinyS and Rust type checking;
* `test` — run tests through Cargo;
* `fmt` — format `.sn` source files;
* `emit-rust` — expose generated Rust for inspection.

## Compiler diagnostics

A core requirement is mapping generated Rust diagnostics back to TinyS source.

Errors should reference `.sn` locations:

```text
error: value `data` was moved here
  --> src/main.sn:14:13
```

Users should not need to debug generated line numbers in internal `.rs` files.

## Planned compiler architecture

A possible first implementation:

```text
Tokenizer
    ↓
Indentation-aware parser
    ↓
TinyS abstract syntax tree
    ↓
Name and type resolution
    ↓
Rust-oriented semantic model
    ↓
Rust source generator
    ↓
Cargo invocation
    ↓
Diagnostic remapping
```

The compiler should avoid performing semantic transformations that silently differ from Rust.

Where possible, Rust should remain the source of truth for:

* ownership checking;
* borrow checking;
* trait resolution;
* monomorphization;
* native code generation;
* platform support;
* dependency linking.

## Example

```tinys
from macro import debug, format
import rust.serde_json as json

#[derive(Debug)]
struct User:
    id: u64
    name: str
    active: bool

def parse_user(source: ref str) -> Result[User, json.Error]:
    return json.from_str[User](source)

def describe(user: ref User) -> str:
    return if user.active:
        format("{} is active", user.name)
    else:
        format("{} is inactive", user.name)

def main() -> Result[void, json.Error]:
    source = r#"{"id":1,"name":"Ada","active":true}"#

    user = parse_user(source)?

    debug(user)
    print(describe(ref user))

    return Ok()
```

Conceptual generated Rust:

```rust
use serde_json;

#[derive(Debug)]
struct User {
    id: u64,
    name: String,
    active: bool,
}

fn parse_user(source: &str) -> Result<User, serde_json::Error> {
    serde_json::from_str::<User>(source)
}

fn describe(user: &User) -> String {
    if user.active {
        format!("{} is active", user.name)
    } else {
        format!("{} is inactive", user.name)
    }
}

fn main() -> Result<(), serde_json::Error> {
    let source = r#"{"id":1,"name":"Ada","active":true}"#;

    let user = parse_user(source)?;

    dbg!(&user);
    println!("{}", describe(&user));

    Ok(())
}
```

## Roadmap

### Phase 1: language core

* indentation-aware parser;
* functions and explicit types;
* immutable and mutable variables;
* structs and enums;
* `if`, `match`, `while`, `for`, and `loop`;
* ownership, moves, references, and dereferencing;
* basic generics;
* Rust source generation;
* Cargo-backed builds.

### Phase 2: practical language support

* traits and implementations;
* lifetime syntax;
* collections and literals;
* closures;
* module discovery;
* public APIs and re-exports;
* async functions;
* source-mapped compiler diagnostics;
* formatter and test runner.

### Phase 3: deeper Rust interoperability

* advanced trait bounds;
* associated types;
* trait objects;
* smart pointers;
* workspaces and multiple targets;
* foreign-function interfaces;
* raw pointers;
* unsafe implementations;
* procedural and user-defined macro integration.

## Open design questions

Several areas remain intentionally undecided:

* exact string literal inference;
* closure syntax details;
* syntax for complex `where` clauses;
* named enum payload syntax;
* standard prelude size;
* comprehension or pipeline syntax;
* module discovery rules;
* raw pointer representation;
* degree of generated-Rust stability;
* whether `own` remains documentation-only;
* whether Python-style inline conditional expressions are supported.

These should be resolved through implementation experiments rather than syntax design alone.

## Why TinyS?

Rust has a powerful semantic model, but some of its syntax reflects historical C-family conventions and the needs of a language designed around explicit low-level control.

Python has highly readable layout and familiar control-flow syntax, but its runtime model does not provide Rust’s ownership, native performance, deterministic destruction, or compile-time safety.

TinyS explores a narrow question:

> What would a Rust-oriented systems language look like if its surface syntax had been designed around indentation, readable type expressions, and explicit words rather than pointer punctuation?

The goal is not to make Rust disappear.

The goal is to make Rust’s strongest ideas easier to read and write.

## License

TinyS is licensed under the [MIT License](LICENSE).

## Contributing

TinyS is still in the language-design stage.

Useful contributions include:

* syntax proposals with concrete examples;
* ambiguity analysis;
* Rust translation examples;
* parser prototypes;
* compiler architecture experiments;
* diagnostic mapping;
* formatter design;
* ownership and lifetime edge cases;
* interoperability tests against real Rust crates.

Proposals should include both TinyS source and the expected generated Rust.

## Name and file extension

The language is called **TinyS**.

TinyS source files use the `.sn` extension.

```text
hello.sn
```
