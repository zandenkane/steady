# how to contribute

This is a Rust project. You need cargo and rustc (stable channel is fine).

Build: `cargo build`
Test: `cargo test`
Lint: `cargo clippy`

If you want to add a new filter type, add it under src/engine/ and wire it into the pipeline in pipeline.rs. Each filter implements the same trait so it should be straightforward.

Bug reports welcome, especially around platform-specific input handling.
