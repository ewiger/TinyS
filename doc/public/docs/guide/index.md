# Language guide

A topic-by-topic reference for TinyS. If you are new, start with the
[tutorial](../getting-started/tutorial.md); this guide is where you come back to
look things up in depth.

## Core

<div class="grid cards" markdown>

- :material-format-align-left: [**Syntax overview**](syntax.md) — indentation blocks, comments, literals, operators.
- :material-variable: [**Variables & mutability**](variables.md) — immutable by default, `mut`, type annotations.
- :material-function: [**Functions**](functions.md) — `def`, parameters, returns, tail expressions.
- :material-transit-connection-variant: [**Ownership & borrowing**](ownership.md) — `ref`, `mut ref`, `at`, `move`, `clone`.

</div>

## Data & types

<div class="grid cards" markdown>

- :material-cube-outline: [**Structs & enums**](structs-and-enums.md) — records, algebraic data types, `impl`.
- :material-sitemap: [**Pattern matching**](pattern-matching.md) — exhaustive `match`/`case`, guards, bindings.
- :material-shape-outline: [**Traits**](traits.md) — shared behavior and implementations.
- :material-code-brackets: [**Generics**](generics.md) — type parameters and trait bounds.
- :material-view-list: [**Collections & tuples**](collections.md) — `list`, `dict`, `set`, tuples.

</div>

## Flow & abstraction

<div class="grid cards" markdown>

- :material-directions-fork: [**Control flow**](control-flow.md) — `if`, `while`, `for`, `loop`, expression forms.
- :material-alert-circle-outline: [**Error handling**](error-handling.md) — `Result`, `Option`, `?`, no `null`.
- :material-lambda: [**Closures**](closures.md) — `fn` closures and `move`.
- :material-folder-multiple-outline: [**Modules & imports**](modules.md) — modules, `pub`, and the `rust` root.

</div>

!!! info "Runs today vs. designed"

    TinyS is at **v0.1.0**. Most of this guide describes behavior you can run
    right now; a few pages describe designed-but-not-yet-complete features and say
    so with a callout. The [Language status](../about/status.md) page is the
    single source of truth for what compiles today.
