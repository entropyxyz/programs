# `entropy-programs-runtime`

This contains the Wasm runtime for evaluaing, testing, and simulating programs as *Wasm Components*.

## Running Tests

Before running the tests, you need to build the `template-barebones` and `infinite-loop` components. Be sure to have `cargo component` installed, and run `cargo component build --release -p template-barebones -p infinite-loop --target wasm32-unknown-unknown`. This will create the files needed for testing at `target/wasm32-unknown-unknown/release/`.
