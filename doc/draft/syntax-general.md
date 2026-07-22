You already have the core identity of TinyS. What remains is mostly the “language completeness” layer: expressions, data construction, modules, mutability, closures, async, and precise Rust interoperability.

The main omitted areas are these.

## 1. Variable declarations and mutability

We have used assignments, but have not formally defined declaration syntax.

```tinys
count: i32 = 0
name = "Ada"
mut total: i64 = 0
```

The key question is whether TinyS distinguishes declaration from reassignment.

A good design would be:

```tinys
count = 0
mut total = 0

total = total + 1
```

Immutable by default, like Rust.

Explicit type:

```tinys
count: i32 = 0
mut total: i64 = 0
```

Possibly Rust-style shadowing:

```tinys
value = "42"
value = parse_i32(value)?
```

That should probably be allowed, because it is useful and fits Rust semantics.

---

## 2. Constants and static values

```tinys
const MAX_RETRIES: i32 = 5
```

Potential static storage:

```tinys
static APPLICATION_NAME: str = "TinyS"
```

Mutable statics should either be omitted initially or require explicit unsafe code.

---

## 3. Primitive types and literals

The basic type vocabulary still needs to be fixed.

Likely primitives:

```tinys
bool
i8
i16
i32
i64
i128
isize

u8
u16
u32
u64
u128
usize

f32
f64
char
str
void
never
```

Literal forms also need specification:

```tinys
42
42u64
3.14
3.14f32
true
false
'a'
"hello"
b"binary"
r"raw string"
```

Hexadecimal, binary, and separators:

```tinys
0xff
0b1010
1_000_000
```

---

## 4. String ownership

This is important because `str` is ambiguous between Rust’s `String` and `str`.

You currently use:

```tinys
name: str
source: ref str
```

The simplest rule would be:

```tinys
str
```

means owned Rust `String`, while:

```tinys
ref str
```

means Rust `&str`.

That is elegant, but it must be stated explicitly.

For example:

```tinys
owned: str = "hello"
borrowed: ref str = ref owned
```

String literals may initially infer as `ref static str`, then convert to owned `str` when required.

---

## 5. Reference creation

We defined `ref T` as a type, but not fully how references are created.

Likely syntax:

```tinys
value_ref = ref value
value_mut_ref = mut ref value
```

Example:

```tinys
name = "Ada"
name_ref: ref str = ref name
```

This completes the symmetry:

```tinys
ref value
mut ref value
at reference
```

---

## 6. Assignment through references

We corrected this to:

```tinys
at target = at source
```

But compound assignment should also work:

```tinys
at counter += 1
```

Generated Rust:

```rust
*counter += 1;
```

---

## 7. Tuples

Tuples are fundamental in Rust and are already implicitly used in loop destructuring.

```tinys
point = (10, 20)
pair: (str, i32) = ("Ada", 42)
```

Access:

```tinys
x = point.0
y = point.1
```

Destructuring:

```tinys
name, age = user_info
```

Single-element tuple:

```tinys
value = (42,)
```

---

## 8. Arrays, lists, slices, and dictionaries

We have type names but not literal semantics.

Likely:

```tinys
numbers: list[i32] = [1, 2, 3]
mapping: dict[str, i32] = {"one": 1, "two": 2}
```

Fixed-size arrays need a distinct type:

```tinys
array[i32, 4]
```

or possibly:

```tinys
[i32; 4]
```

Since TinyS prefers square-bracket generics, this is more consistent:

```tinys
array[i32, 4]
```

Slices:

```tinys
slice[i32]
ref slice[i32]
mut ref slice[i32]
```

Examples:

```tinys
part = values[1..4]
```

The standard meaning could be:

```tinys
list[T]       # Vec[T]
array[T, N]   # [T; N]
slice[T]      # [T]
dict[K, V]    # HashMap[K, V]
set[T]        # HashSet[T]
```

---

## 9. Indexing

```tinys
first = values[0]
values[0] = 10
```

There is a major semantic question: should indexing panic like Rust, or return `Option`?

The best rule is probably to preserve Rust:

```tinys
value = values[index]
```

may panic when out of bounds.

Safe access:

```tinys
value = values.get(index)
```

returns `Option[ref T]`.

---

## 10. Struct construction

We defined structs, but not how instances are built.

Python-like keyword construction fits well:

```tinys
point = Point(
    x=10.0,
    y=20.0,
)
```

Generated Rust:

```rust
Point {
    x: 10.0,
    y: 20.0,
}
```

Field shorthand:

```tinys
point = Point(x, y)
```

could be ambiguous with tuple structs.

A clearer shorthand may be:

```tinys
point = Point(
    x,
    y,
)
```

where matching variable names imply field shorthand.

But explicit `x=x` is easier for the first implementation.

---

## 11. Tuple structs and newtypes

Rust uses these heavily.

```tinys
struct UserId(u64)

struct Coordinates(f64, f64)
```

Usage:

```tinys
user_id = UserId(42)
```

This is useful for strongly typed wrappers.

---

## 12. Enum construction

Variants should be constructible naturally:

```tinys
token = Token.Number(42)
result = Ok(user)
error = Err(error)
optional = Some(value)
```

Unit variants:

```tinys
token = Token.End
```

Named-field variants may also be needed:

```tinys
enum Message:
    Move(x: i32, y: i32)
    Write(text: str)
    ChangeColor(red: u8, green: u8, blue: u8)
```

Or perhaps true named payloads:

```tinys
enum Message:
    Move:
        x: i32
        y: i32
```

That part needs a design decision.

---

## 13. `none`, `Some`, `Option`, and nullability

TinyS should probably have no general `null`.

Absence should use:

```tinys
Option[T]
```

Construction:

```tinys
user = Some(value)
user = none
```

Pattern matching:

```tinys
match user:
    case Some(value):
        ...
    case none:
        ...
```

This is cleaner than importing `None`, which conflicts with Python expectations.

Generated Rust:

```rust
Some(value)
None
```

---

## 14. Operators

The full operator table needs specification.

Arithmetic:

```tinys
+
-
*
/
%
**
```

Rust has no built-in `**`, so TinyS must either lower it to `.pow()` or omit it.

Comparison:

```tinys
==
!=
<
<=
>
>=
```

Boolean:

```tinys
and
or
not
```

Generated Rust:

```rust
&&
||
!
```

Bitwise:

```tinys
&
|
^
~
<<
>>
```

Assignment:

```tinys
+=
-=
*=
/=
%=
&=
|=
^=
<<=
>>=
```

Identity comparison such as Python’s `is` should probably not exist, or should have very narrow reference semantics.

---

## 15. Operator precedence

This is easy to forget, but essential.

TinyS should likely follow Python precedence wherever syntax matches Python, while ensuring generated Rust preserves the same parse tree.

The transpiler should not rely blindly on Rust precedence. It should emit parentheses where needed.

---

## 16. Conversions and casts

Rust distinguishes safe conversion from casting.

Explicit primitive cast:

```tinys
large: i64 = small as i64
```

This is already readable and could remain unchanged.

Trait-based conversion:

```tinys
value = i64.from(small)
```

Fallible conversion:

```tinys
value = i32.try_from(large)?
```

String parsing:

```tinys
number = source.parse[i32]()?
```

We should avoid Python-style implicit conversions.

---

## 17. Closures and lambda expressions

This is a major missing area.

Python lambda syntax is too limited:

```tinys
double = lambda value: value * 2
```

Rust closures are more powerful. A TinyS form could be:

```tinys
double = def(value: i32) -> i32:
    value * 2
```

But anonymous `def` looks odd.

A concise possibility:

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

Usage:

```tinys
values.map(fn(value): value * 2)
```

This is one of the most important remaining syntax decisions.

---

## 18. Capturing closures

TinyS also needs Rust closure capture semantics.

By-reference capture could remain inferred:

```tinys
prefix = "user:"
format_user = fn(name: ref str):
    format("{}{}", prefix, name)
```

Move closure:

```tinys
worker = move fn():
    process(data)
```

That maps well to Rust:

```rust
move || process(data)
```

This is a natural reuse of `move`.

---

## 19. Methods consuming or mutating `self`

We mentioned receiver types, but should define them fully.

```tinys
impl Builder:

    def build(self: Self) -> Product:
        ...

    def inspect(self: ref Self) -> void:
        ...

    def update(self: mut ref Self) -> void:
        ...
```

Potential shorthand:

```tinys
def build(self) -> Product:
def inspect(ref self) -> void:
def update(mut ref self) -> void:
```

But the current typed form is more regular.

---

## 20. Constructors

Rust has no language-level constructor, but TinyS may benefit from convention.

```tinys
impl User:

    def new(name: str) -> Self:
        return Self(
            name=name,
        )
```

Call:

```tinys
user = User.new("Ada")
```

This naturally maps to `User::new`.

No special constructor syntax is necessary.

---

## 21. Visibility

Rust has `pub`, `pub(crate)`, and related forms. TinyS needs an equivalent.

The simplest syntax is to preserve Rust keywords:

```tinys
pub struct User:
    pub id: u64
    name: str
```

Public function:

```tinys
pub def load_user(id: u64) -> Result[User, Error]:
    ...
```

Restricted visibility:

```tinys
pub[crate] def helper():
    ...
```

or keep Rust unchanged:

```tinys
pub(crate) def helper():
```

Since TinyS otherwise uses square brackets for parameters, `pub[crate]` fits its style.

---

## 22. Module definitions

**Resolved: modules are derived entirely from files. TinyS has no `mod` keyword,
not even as an optional form.**

```text
src/
    app.sn              the file you build — the crate root
    models.sn           → crate::models
    models_test.sn      → crate::models_test, declared #[cfg(test)]
    services/
        mod.sn          → crate::services
        database.sn     → crate::services::database
```

The compiler walks `src/`, derives the tree, and writes the `mod` declarations
itself. This is the important usability improvement over Rust: a file is a
module because it exists, the way it works in Python and Go, and no declaration
can fall out of sync with the filesystem.

Rejecting the alternative — `mod models` as an optional declaration — is
deliberate. Two ways to say one thing would force the compiler to reconcile
hand-written declarations against what it found on disk, for no expressive gain.

### What `mod` is still needed for, and what replaces it

Rust's `mod` does three jobs. Only the first is about files:

1. **Attaching a file to the tree.** Fully derived from the directory walk.
2. **Grouping items inline** (`#[cfg(test)] mod tests`). Replaced by the
   `_test.sn` suffix: such a file is declared `#[cfg(test)]`, giving colocated
   tests without a keyword.
3. **Carrying attributes** (`#[cfg(unix)] mod platform`). Use an inner attribute
   at the top of the module's own file:

```tinys
#![cfg(unix)]

pub def open_tty() -> void:
    ...
```

### Discovery rules

* `src/` is the switch. A package with `tinys.toml` and no `src/` is a directory
  of single-file programs, which is how a flat `examples/` layout keeps working.
* The file you build is the crate root, and is never also a module. `main.sn` is
  therefore not required — any `.sn` file with a `def main()` can be the entry.
* A directory is a module. Its own source is `mod.sn`; a directory without one
  still exists as a module holding its children.
* `foo.sn` beside `foo/` is an error — one module name, two sources.
* Synthesized declarations are always `pub mod`. The `pub` a user writes on
  *items* is what defines the public surface; a private module would make that
  a lie.
* Imports are absolute from the crate root. There is no relative form yet
  (`from . import x`), matching Python 3's own default.
* `[package] exclude` in `tinys.toml` keeps files out of the tree. `*` matches
  within one path segment, and naming a directory excludes everything under it.

Mutual imports between modules are fine — the whole tree is one compilation
unit, so TinyS inherits Rust's freedom here rather than Python's import cycles.

---

## 23. Re-exports

Rust libraries depend heavily on re-exports.

```tinys
export from models import User
export from errors import Error
```

Generated Rust:

```rust
pub use models::User;
pub use errors::Error;
```

This is more readable than exposing Rust’s `pub use` directly.

---

## 24. Package and crate boundaries

You have `tinys.toml`, but should define:

```text
package
crate
module
file
workspace
```

Likely rules:

* one TinyS package maps to one Cargo package;
* a library entry point maps to `lib.rs`;
* an application entry point maps to `main.rs`;
* multiple binaries can be declared in `tinys.toml`;
* workspace members map to Cargo workspace members.

Implemented so far: `tinys.toml` defines the package, `[dependencies]` is
resolved through Cargo, and `src/` defines the module tree (§22). Each `.sn`
entry point currently generates its own Cargo package with a `main.rs`; library
targets (`lib.rs`) and multiple declared binaries are still open.

---

## 25. Main function

Application entry point:

```tinys
def main() -> void:
    print("Hello")
```

Fallible main:

```tinys
def main() -> Result[void, Error]:
    run()?
    return Ok()
```

You may define:

```tinys
Ok()
```

as equivalent to Rust’s `Ok(())`.

---

## 26. Unit values

TinyS uses `void` as the unit type, but it still needs a unit value.

Options:

```tinys
void
()
```

The cleanest approach may be:

```tinys
return
```

for functions returning `void`, and:

```tinys
Ok()
```

for `Result[void, Error]`.

Internally, this becomes Rust `()`.

There is probably no need for a visible `void` value.

---

## 27. Error definition

Custom error types are central to real Rust programs.

```tinys
enum ParseError:
    EmptyInput
    InvalidNumber(str)
```

Attributes:

```tinys
#[derive(Debug)]
enum ParseError:
    ...
```

Integration with `std::error::Error` and formatting may require trait implementations.

You may later support libraries like `thiserror` through attributes and Rust imports rather than adding language syntax.

---

## 28. `try` blocks

Rust’s `?` operator is already included, but expression-scoped propagation may be useful.

```tinys
result = try:
    user = load_user(id)?
    profile = load_profile(user)?
    profile.name
```

This could generate a Rust closure or native try block if stabilized and suitable.

This is optional, but not necessarily “advanced.” It could be very useful.

---

## 29. Resource cleanup and deterministic destruction

Rust uses RAII rather than `finally`.

TinyS should explicitly state:

* values are dropped at the end of scope;
* destructors run deterministically;
* no garbage collector is assumed;
* ownership determines cleanup.

You may expose explicit dropping:

```tinys
drop(connection)
```

probably imported from the prelude or `rust.std.mem`.

A Python-style `with` statement may eventually map to scoped ownership, but it is not essential.

---

## 30. `defer`

TinyS might be tempted to add:

```tinys
defer cleanup()
```

But Rust’s RAII normally makes this unnecessary.

I would omit `defer` initially. When needed, users can use a guard type or a crate.

---

## 31. Exceptions

TinyS should explicitly say that it does not have Python-style exceptions as its normal error model.

No:

```tinys
try:
    ...
except Error:
    ...
```

Instead:

```tinys
Result[T, E]
?
match
```

Panics remain for unrecoverable errors, not ordinary control flow.

---

## 32. Async functions

This is no longer really advanced; it is required for practical Rust interoperability.

```tinys
async def fetch_user(id: u64) -> Result[User, Error]:
    response = client.get(id).await?
    return response.json[User]().await
```

Generated Rust:

```rust
async fn fetch_user(id: u64) -> Result<User, Error> {
    let response = client.get(id).await?;
    response.json::<User>().await
}
```

`await` should probably remain postfix:

```tinys
result = operation.await?
```

This matches Rust and avoids Python’s prefix `await`.

---

## 33. Async entry point attributes

```tinys
#[tokio::main]
async def main() -> Result[void, Error]:
    run().await?
    return Ok()
```

Attributes already solve this without special language constructs.

---

## 34. Concurrency basics

The language does not need new concurrency syntax initially.

Rust crates can provide:

```tinys
from rust.std.thread import spawn
from rust.tokio import spawn
```

Possibly:

```tinys
handle = spawn(fn():
    process(data)
)
```

Channels, mutexes, atomics, and tasks should initially remain library concepts.

---

## 35. `unsafe`

TinyS must define how unsafe Rust operations are expressed.

The simplest option is to preserve the keyword:

```tinys
unsafe:
    raw_operation()
```

Unsafe function:

```tinys
unsafe def read_address(address: usize) -> u8:
    ...
```

Unsafe implementation:

```tinys
unsafe impl Send for Device:
    ...
```

Even if raw pointers are postponed, unsafe may still be needed for Rust interoperability.

---

## 36. Raw pointers

You previously questioned whether TinyS needs them. The answer is: not for ordinary TinyS code, but probably for full Rust FFI.

They can remain explicit Rust interop types:

```tinys
rust.ptr.const[T]
rust.ptr.mut[T]
```

or:

```tinys
raw ref T
raw mut ref T
```

I would avoid designing this until FFI is addressed.

---

## 37. Foreign function interface

Eventually:

```tinys
extern "C":

    def strlen(value: rust.ptr.const[u8]) -> usize
```

Or preserve Rust-like declarations with indentation.

This can be postponed without weakening the first language version.

---

## 38. Trait bounds and `where`

Simple bound:

```tinys
def clone_value[T: Clone](value: ref T) -> T:
    return value.clone()
```

Multiple bounds:

```tinys
def render[T: Display + Debug](value: ref T) -> str:
    ...
```

Complex constraints need a `where` form:

```tinys
def combine[T, U](left: T, right: U) -> str
where:
    T: Display
    U: Display + Clone
:
    ...
```

That double colon is awkward.

A better syntax:

```tinys
def combine[T, U](left: T, right: U) -> str:
    where T: Display
    where U: Display + Clone

    ...
```

Or preserve Rust’s header shape:

```tinys
def combine[T, U](left: T, right: U) -> str
where T: Display, U: Display + Clone:
    ...
```

This is an important unfinished area.

---

## 39. Associated types

Traits frequently need associated types.

```tinys
trait Iterator:
    type Item

    def next(self: mut ref Self) -> Option[Self.Item]
```

Associated constants:

```tinys
trait Limits:
    const MAX: usize
```

Implementations:

```tinys
impl Iterator for Counter:
    type Item = i32

    def next(self: mut ref Self) -> Option[i32]:
        ...
```

---

## 40. Default trait methods

```tinys
trait Display:

    def display(self: ref Self) -> str

    def debug_display(self: ref Self) -> str:
        return self.display()
```

This should map directly to Rust default trait methods.

---

## 41. Generic implementations

```tinys
impl[T] Box[T]:

    def value(self: ref Self) -> ref T:
        return ref self.inner
```

Bounded implementation:

```tinys
impl[T: Display] Container[T]:

    def display(self: ref Self) -> str:
        return self.value.display()
```

---

## 42. Type aliases

```tinys
type UserId = u64
type UserResult = Result[User, UserError]
```

Generic alias:

```tinys
type Callback[T] = fn(T) -> void
```

Newtypes remain structs, not aliases.

---

## 43. Function types

Closures and callbacks require syntax.

```tinys
callback: fn(i32) -> bool
```

Possibly borrowed dynamic callable:

```tinys
callback: ref dyn Fn(i32) -> bool
```

TinyS could simplify Rust trait-object syntax later, but initially direct Rust forms may be safest.

---

## 44. Trait objects and `dyn`

```tinys
handler: box[dyn Handler]
renderer: ref dyn Display
```

This is fundamental for dynamic dispatch.

You may define:

```tinys
box[T]
rc[T]
arc[T]
```

as prelude aliases for common Rust ownership containers.

---

## 45. Smart pointers

Likely standard generic types:

```tinys
box[T]
rc[T]
arc[T]
weak[T]
cell[T]
ref_cell[T]
mutex[T]
rw_lock[T]
```

These can be library/prelude types rather than language primitives.

But `box[T]` may deserve special treatment because recursive data structures need it:

```tinys
enum Node:
    Value(i32)
    Next(box[Node])
```

---

## 46. Derivation and generated implementations

Attributes already support:

```tinys
#[derive(Debug, Clone, PartialEq)]
```

But TinyS should document that derive macros may generate Rust traits normally.

No special TinyS syntax is needed.

---

## 47. Documentation comments

Rust documentation tooling matters.

Possible syntax:

```tinys
/// Loads a user by identifier.
///
/// Returns `none` when no user exists.
pub def load_user(id: u64) -> Option[User]:
    ...
```

Since Rust comments are clear, keep them unchanged:

```tinys
// normal comment
/// documentation comment
//! module documentation
```

Python-style `#` comments should probably not be used because they conflict visually with Rust attributes:

```tinys
#[derive(...)]
```

Using `//` keeps generated source closer to Rust.

---

## 48. Formatting and interpolation

You already have imported `format`.

Potential convenience:

```tinys
message = f"Hello {user.name}"
```

But this introduces a substantial formatting translation layer.

I would initially keep:

```tinys
message = format("Hello {}", user.name)
```

It is explicit and interoperable.

---

## 49. Comprehensions

Python developers may expect:

```tinys
squares = [value * value for value in values]
```

But Rust iterator chains have clearer ownership semantics.

TinyS could support comprehensions later, but the initial language may use:

```tinys
squares = values
    .into_iter()
    .map(fn(value): value * value)
    .collect[list[i32]]()
```

A cleaner TinyS pipeline syntax may eventually be valuable, but it is not essential to the core.

---

## 50. Destructuring assignment

```tinys
left, right = pair
Point(x=x, y=y) = point
```

The first is straightforward.

The second is refutable or structural and must follow Rust rules. A declaration-specific syntax may be safer:

```tinys
let Point(x=x, y=y) = point
```

But TinyS currently avoids `let`.

One possibility:

```tinys
Point(x=x, y=y) := point
```

I would initially support only irrefutable tuple destructuring:

```tinys
left, right = pair
```

and use `match` for more complicated patterns.

---

## 51. Scope and temporary lifetime rules

Because TinyS generates Rust, it should preserve Rust’s borrow checker exactly.

But some surface syntax may produce surprising temporary lifetimes:

```tinys
reference = ref create_value()
```

The compiler must either reject this clearly or introduce a hidden local only when doing so is semantically safe and unsurprising.

The safest principle is:

> TinyS may simplify syntax, but it must not silently extend the lifetime of temporaries in ways that differ materially from Rust.

---

## 52. Compiler diagnostics

This is not syntax, but it is essential to the project.

TinyS should map Rust errors back to `.sn` source locations.

The generated Rust should contain source mapping metadata or stable line correspondence.

For example:

```text
error: value `data` was moved here
  --> src/main.sn:14:13
```

not merely:

```text
target/tinys-generated/main.rs:82:19
```

Without this, the language will feel like a preprocessing experiment rather than a real compiler.

---

## 53. Generated-code stability

TinyS should define whether generated `.rs` files are:

* visible;
* editable;
* deterministic;
* committed to source control;
* regenerated on every build.

I would recommend:

* generated Rust is visible for debugging;
* generated files are deterministic;
* users should not edit them;
* source maps connect them back to `.sn`;
* generated Rust can be inspected with a command such as:

```text
tinys emit-rust
```

---

## 54. Formatting tool

A `tinys fmt` command is almost mandatory.

Because indentation has semantic meaning, formatting rules should be strict and deterministic.

It should handle:

* indentation;
* multiline parameters;
* imports;
* attributes;
* match branches;
* trailing commas;
* line wrapping.

---

## 55. Prelude

TinyS needs a small standard prelude.

Likely automatically available:

```tinys
Option
Result
Some
Ok
Err
none

list
dict
set
box

print
len
clone
drop
enumerate
zip
```

But the prelude should remain small. Too much implicit importing makes Rust interop harder to understand.

---

# Highest-priority missing pieces

For the next design pass, I would focus on these ten:

1. variable declaration, mutability, and shadowing;
2. owned `str` versus borrowed `ref str`;
3. creating references with `ref value` and `mut ref value`;
4. struct and enum construction;
5. arrays, lists, slices, tuples, and dictionaries;
6. closures using something like `fn(...):`;
7. visibility with `pub`;
8. async functions and `.await`;
9. trait bounds, `where`, associated types, and generic `impl`;
10. module/file layout and re-exports.

After those are settled, TinyS would no longer just have attractive syntax fragments. It would have enough surface area to write realistic applications and libraries.
