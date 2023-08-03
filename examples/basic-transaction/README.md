# `examples`

This contains examples of constraints. Currently, barebones works best.

## Building Components

To build the `barebones` component, run `cargo build --release --bin barebones --target wasm32-unknown-unknown`. This will create a `barebones.wasm` file in `target/wasm32-unknown-unknown/release/`.

This creates the constraint as a Wasm *module*. To convert it into a *component*, we must use `wasm-tools component new target/wasm32-unknown-unknown/release/barebones.wasm -o runtime/tests/barebones-component.wasm`. Using `wasmtime::bindgen` as seen in `ec-runtime/tests/basic_runtime_call.rs`, we can see that the component is callable with rich types via WIT.
