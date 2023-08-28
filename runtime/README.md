# `ec-runtime`

This contains the Wasm runtime for evaluaing, testing, and simulating constraints as *Wasm Components*.

## Running Tests

Before running the tests, you need to build the `barebones` component. Be sure to have `cargo component` installed, and run `cargo component build --release -p template-barebones --target wasm32-unknown-unknown`. This will create the files needed for testing at `target/wasm32-unknown-unknown/release/`.
