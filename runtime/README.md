# `ec-runtime`

This contains the Wasm runtime for evaluaing, testing, and simulating constraints as *Wasm Components*.

## Building Example Constraints

To build the `barebones` Wasm *module*, run `cargo build --release --bin barebones --target wasm32-unknown-unknown`. This will create a `barebones.wasm` file in `target/wasm32-unknown-unknown/release/`.

To convert `barebones.wasm` into a Wasm *component*, we must use `wasm-tools component new target/wasm32-unknown-unknown/release/barebones.wasm -o runtime/tests/barebones-component.wasm`. The WIT that the component uses are generated here using `wasmtime::bindgen` as seen in `ec-runtime/tests/basic_runtime_call.rs`. We can see that the component is callable with rich types via WIT (ie `EvaluationState`).
