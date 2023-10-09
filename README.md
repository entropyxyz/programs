# Entropy Programs

Entropy allows the creation of decentralized signing authorities. Signing authorities exist as WebAssembly programs that can ingest a signature request, and a valid request is signed via a threshold signature scheme from a set of at-stake validators. These requests might include cryptocurrency-based transaction requests, certificate signing requests, or other mediums for cryptographic authentication.

This repository contains libraries, toolchains, utilities, and specifications for writing, configuring, building, and testing the Wasm-based applications for Entropy. In theory, programs can be written in any language that compiles to WebAssembly, which includes all LLVM-supported languages like Rust, AssemblyScript, Zig, C, etc. Thanks to the WebAssembly [Component Model](https://component-model.bytecodealliance.org), programs can even be reused languages, with the rich typing provided by the component model interfaces.

## Writing Programs

### Build Requirements

Besides the latest stable Rust toolchain, you will also need to install:
- [cargo component](https://github.com/bytecodealliance/cargo-component#installation), a Cargo extension for building Wasm components.
- [wasm-tools](https://github.com/bytecodealliance/wasm-tools#installation), to be used by `cargo-component`

## Example Program: `barebones`

To get started, clone this repository and build the example `barebones` program:

```bash
git clone https://github.com/entropyxy/constraints
cd constraints
cargo component build --release -p template-barebones --target wasm32-unknown-unknown
```

This creates the program as a Wasm component at `target/wasm32-unknown-unknown/release/template_barebones.wasm`.

Since this program is used in tests for the program runtime (`ec-runtime`), you can see the program get used by running `cargo test -p ec-runtime`.
