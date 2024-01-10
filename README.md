# Entropy Programs

Entropy allows the creation of decentralized signing authorities. Signing authorities exist as WebAssembly programs that can ingest a signature request, and a valid request is signed via a threshold signature scheme from a set of at-stake validators. These requests might include cryptocurrency-based transaction requests, certificate signing requests, or other mediums for cryptographic authentication.

This repository contains libraries, toolchains, utilities, and specifications for writing, configuring, building, and testing the Wasm-based applications for Entropy. In theory, programs can be written in any language that compiles to WebAssembly, which includes all LLVM-supported languages like Rust, AssemblyScript, Zig, C, etc. Thanks to the WebAssembly [Component Model](https://component-model.bytecodealliance.org), programs can even be reused languages, with the rich typing provided by the component model interfaces.

## Writing Programs

### Build Requirements

Besides the latest stable Rust toolchain, you will also need to install:

- [cargo component v0.2.0](https://github.com/bytecodealliance/cargo-component#installation), a Cargo extension for building Wasm components.
- [wasm-tools](https://github.com/bytecodealliance/wasm-tools#installation), to be used by `cargo-component`.

These can be installed as follows:

```bash
cargo install cargo-component --version 0.2.0 &&
cargo install wasm-tools
```

## Example Program: `template-barebones`

An example of a barebones program is at [`examples/barebones/src/lib.rs`](./examples/barebones/src/lib.rs). This example does a simple check on the length of the message to be signed.

You can compile the program by running:

```bash
cargo component build --release -p template-barebones --target wasm32-unknown-unknown
```

This builds the program as a Wasm component at `target/wasm32-unknown-unknown/release/template_barebones.wasm`.

## Running Tests

Before running the runtime tests, you need to build the `template-barebones` and `infinite-loop` components. To do this, execute:

```bash
cargo component build --release -p template-barebones -p infinite-loop --target wasm32-unknown-unknown`
```

This will create the components in `target/wasm32-unknown-unknown/release/`.

## Licensing

For the most part, the code in this repository is licensed under [AGPL-3.0](./LICENSE).

There are some exceptions however:

- The original code in the `examples/risc0-zkvm-verification` crate comes from RISC Zero's [`risc0`](https://github.com/risc0/risc0) project, which is licensed under
  `Apache License 2.0`.

Modifications made by Entropy to these crates are licensed under `AGPL-3.0`.
