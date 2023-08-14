# Entropy Constraints

Entropy is a decentralized platform for securely managing digital assets. It uses threshold cryptography to sign messages such as transaction requests, and users write constraints to define the conditions under which signature requests are valid.

This repository contains libraries, toolchains, utilities, and specifications for writing, configuring, building, and testing the Wasm-based applications for Entropy. Constraints can be written in any language that compiles to WebAssembly, including LLVM-supported languages like Rust, AssemblyScript, Zig, C, etc. With WIT binding generation, constraints can be reused across *all* of these languages, with rich typing (not the C ABI).

## Build Requirements

Besides the latest stable Rust toolchain, you will need to install [wasm-tools](https://github.com/bytecodealliance/wasm-tools) for constructing Wasm components. You can install it by running:

```bash
cargo install wasm-tools
```

## Example Constraint: `barebones`

To get started, clone this repository and build the example `barebones` constraint binary:

```bash
git clone https://github.com/entropyxy/constraints
cd constraints
cargo build --release --bin barebones --target wasm32-unknown-unknown
```

This creates the constraint as a *Wasm Module* a `target/wasm32-unknown-unknown/release/barebones.wasm`.

To convert it into a *Wasm Component*, we must use `wasm-tools component new target/wasm32-unknown-unknown/release/barebones.wasm -o runtime/tests/barebones-component.wasm`. The Using `wasmtime::bindgen` as seen in `ec-runtime/tests/basic_runtime_call.rs`, we can see that the component is callable with rich types via WIT.

Check `runtime/tests/basic_runtime_call.rs` to see how you can perform assertions on the constraint based on the input data to the runtime.
