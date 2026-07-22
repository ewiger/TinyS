# Async

Async functions keep familiar syntax. TinyS preserves Rust's postfix `.await`
because it composes naturally with method calls and the `?` operator.

## Async functions

Prefix a function with `async`:

```python
async def fetch_user(id: u64) -> Result[User, Error]:
    response = client.get(id).await?
    return response.json[User]().await
```

`.await` is postfix and chains cleanly with `?`:

```python
response = client.get(id).await?
```

## Async `main` and runtimes

Async runtimes remain Rust-library concerns, wired up with the usual attributes:

```python
#[tokio::main]
async def main() -> Result[void, Error]:
    run().await?
    return Ok()
```

## Generated Rust

```python
async def fetch_user(id: u64) -> Result[User, Error]:
    response = client.get(id).await?
    return response.json[User]().await
```

```rust
async fn fetch_user(id: u64) -> Result<User, Error> {
    let response = client.get(id).await?;
    return response.json::<User>().await;
}
```

!!! info "Designed feature"

    Async is part of the Phase 2 language design. Because it depends on async
    runtime crates (e.g. `tokio`), it also depends on the planned Cargo-backed
    build path. See the [roadmap](../about/roadmap.md) and
    [Language status](../about/status.md).

## Where to go next

- [Error handling](../guide/error-handling.md) — `?` and `Result`.
- [Rust interoperability](interop.md) — bringing in runtime crates.
