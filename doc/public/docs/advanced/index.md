# Advanced

Deeper topics for when you are ready to interoperate with the Rust ecosystem and
reason about the generated output.

<div class="grid cards" markdown>

- :material-timer-sand: [**Lifetimes**](lifetimes.md) — naming how long references live, with dot-names.
- :material-sync: [**Async**](async.md) — `async def` and postfix `.await`.
- :material-language-rust: [**Rust interoperability**](interop.md) — crates through the `rust` root.
- :material-code-tags: [**Macros**](macros.md) — imported macros, called without `!`.
- :material-alert-octagon-outline: [**Unsafe**](unsafe.md) — explicit `unsafe` blocks and functions.
- :material-file-code-outline: [**Reading the generated Rust**](generated-rust.md) — inspect and trust the output.

</div>

!!! info "Maturity varies by topic"

    Several features on these pages are **designed** (specified in the language
    reference) but not yet fully wired into the v0.1.0 compiler — lifetimes and
    async in particular. Each page flags its status, and
    [Language status](../about/status.md) is the authoritative summary.
