//! Integration test scaffolding.
//!
//! Each `.rs` file in this folder is compiled as a *separate* test crate
//! and run by `cargo test`. They're intended for tests that exercise the
//! engine's public API — i.e., what an external consumer would touch.
//!
//! Right now there is no public API, because `game_engine` is a binary
//! crate and binary crates cannot expose items to external test code.
//! The conventional Rust refactor when integration tests start to matter
//! is:
//!
//!   1. Move engine code into `src/lib.rs` (a library crate that
//!      exposes a public `pub fn run()` or similar).
//!   2. Reduce `src/main.rs` to a thin wrapper:
//!        fn main() { game_engine::run(); }
//!   3. Integration tests in this folder can then `use game_engine::*;`
//!      and call into the engine like any external crate.
//!
//! Until then, real test cases live alongside the code in
//! `src/main.rs` under `#[cfg(test)] mod tests` (those have access to
//! private items, which integration tests don't).
//!
//! The single `#[test]` below exists only so this scaffold compiles
//! and `cargo test` reports an integration-test stage. Replace it with
//! something useful the day a public API ships.
//!   https://doc.rust-lang.org/book/ch11-03-test-organization.html

#[test]
fn integration_scaffold_compiles() {
    // Placeholder — replace once the engine has a public API.
    assert_eq!(2 + 2, 4);
}
