# Bevy Book Examples

Compilable examples for the Bevy 2D and ECS book, targeting Bevy 0.19 and Rust
Edition 2024.

This repository turns the book's illustrative snippets into executable,
testable code. It intentionally keeps chapter crates independent so that each
chapter can enable only the Bevy features it teaches.

## Requirements

- Rust 1.96.0 (installed automatically by `rustup` from `rust-toolchain.toml`)
- Cargo

## Layout

```text
chapters/
  chapter-04-first-contact/
    src/lib.rs                 # Testable chapter behavior
    examples/first_ecs.rs      # Executable section example
```

Use one crate per chapter. Within a chapter, use one executable under
`examples/` per independently runnable section. Keep behavior in `src/lib.rs`
when it can be verified without a renderer or window.

## Validation

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Run the first headless example with:

```bash
cargo run -p bevy-book-chapter-04 --example first_ecs
```

The workspace pins Bevy to `0.19.0`. Headless chapters inherit Bevy with
default features disabled; chapter crates should opt into rendering, audio, or
platform features only when their examples require them.

## Adding An Example

1. Create or reuse the corresponding `chapters/chapter-NN-topic` crate.
2. Name the executable after the book section's concept, not its listing number.
3. Add a behavioral test for deterministic ECS or domain logic.
4. Run the full validation commands before opening a pull request.
