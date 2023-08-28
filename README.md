# Entropy Constraints

Entropy is a decentralized platform for securely managing digital assets. It uses threshold cryptography to sign messages such as transaction requests, and users write constraints to define the conditions under which signature requests are valid.

This repository contains libraries, toolchains, utilities, and specifications for writing, configuring, building, and testing the Wasm-based applications for Entropy. Programs can be written in any language that compiles to WebAssembly, including LLVM-supported languages like Rust, AssemblyScript, Zig, C, etc. Thanks to Wasm interface types, programs can even be reused languages, with the rich typing provided by the WIT specification.

## Build Requirements

Besides the latest stable Rust toolchain, you will also need [cargo component](https://github.com/bytecodealliance/cargo-component) for building Wasm components. You can install it by running:

```bash
cargo install wasm-tools
```

## Example Constraint: `barebones`

To get started, clone this repository and build the example `barebones` program:

```bash
git clone https://github.com/entropyxy/constraints
cd constraints
cargo component build --release -p template-barebones --target wasm32-unknown-unknown
```

This creates the program as a Wasm component at `target/wasm32-unknown-unknown/release/template_barebones.wasm`.

Since this program is used in tests for the program runtime (`ec-runtime`), you can see the program get used by running `cargo test -p ec-runtime`.
