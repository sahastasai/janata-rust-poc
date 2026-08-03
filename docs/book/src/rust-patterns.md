# Rust patterns used here

This is a project glossary, not a tour of the whole language.

## Ownership: who cleans up a value?

Rust values have one owner. Passing a `String` by value transfers it; passing
`&str` borrows it temporarily. Validation functions usually borrow input and
return an owned, normalized `String`:

```rust,ignore
let title = validate_required_text("title", raw_title, 120)?;
```

The caller can keep using `raw_title`, while `title` is safe to store. This is
why API parsing, validation, and persistence appear as separate steps.

## `Result`: success or an explained failure

A function returning `Result<T, E>` produces either `Ok(T)` or `Err(E)`. The
`?` operator returns early when it sees an error:

```rust,ignore
let body = validate_required_text("body", input, 4_096)?;
```

At the Worker boundary, internal errors become a stable `ApiError`. Do not send
database messages, stack traces, or secret values to clients.

## `Option`: absence is explicit

`Option<T>` is either `Some(T)` or `None`. It replaces sentinel empty strings
and many null-related crashes. Match it when absence changes behavior; use
helpers such as `map` only when that remains easy to read.

## Newtype IDs prevent category mistakes

`UserId`, `CenterId`, and `EventId` wrap UUIDs. Even though the wire value is a
UUID, Rust will reject passing an event ID where a center ID is required.

## Feature flags separate runtimes

The Dioxus crate has `web` and `mobile` features. Platform-specific code uses
`#[cfg(feature = "web")]` or a small adapter, while components and domain logic
remain shared. Never enable both renderer features just to silence a build
error; fix or isolate the incompatible dependency.
