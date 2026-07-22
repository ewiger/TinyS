# Unsafe

Unsafe operations stay **explicit** in TinyS, exactly as in Rust. Nothing about
the readable surface syntax weakens the safety boundary.

## Unsafe blocks

Wrap unsafe operations in an `unsafe` block:

```python
unsafe:
    perform_raw_operation()
```

```rust
unsafe {
    perform_raw_operation();
}
```

## Unsafe functions

A function that is unsafe to call is marked with `unsafe`:

```python
unsafe def read_address(address: usize) -> u8:
    ...
```

```rust
unsafe fn read_address(address: usize) -> u8 {
    // ...
}
```

## Raw pointers and FFI

Raw pointers and foreign-function interfaces are intentionally **postponed** until
the core ownership and Rust-interoperability model is stable. Their representation
is one of the language's open design questions.

!!! info "Later phase"

    Raw pointers, FFI, and unsafe implementations belong to the deeper
    Rust-interoperability phase. See the [roadmap](../about/roadmap.md) and
    [Language status](../about/status.md).

## Where to go next

- [Ownership & borrowing](../guide/ownership.md)
- [Rust interoperability](interop.md)
