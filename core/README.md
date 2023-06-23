# `ec-core`

This contains core traits and types for writing modular constraints code, including constraints, runtimes, architectures (for writing architecture-agnostic constraints and dynamic parsing) and signature-request interfaces.

## `.wit` generation

The `index.wit` file is generated from `witgen` and its associated CLI tool. With `wit-bindgen`, this allows the Wasm host and guest to share richer types across the FFI boundary.
