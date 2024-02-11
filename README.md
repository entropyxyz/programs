# Entropy Programs

Entropy allows the creation of decentralized signing authorities. Signing authorities exist as WebAssembly programs that can ingest a signature request, and a valid request is signed via a threshold signature scheme from a set of at-stake validators. These requests might include cryptocurrency-based transaction requests, certificate signing requests, or other mediums for cryptographic authentication.

This repository contains libraries, toolchains, utilities, and specifications for writing, configuring, building, and testing the Wasm-based applications for Entropy. In theory, programs can be written in any language that compiles to WebAssembly, which includes all LLVM-supported languages like Rust, AssemblyScript, Zig, C, etc. Thanks to the WebAssembly [Component Model](https://component-model.bytecodealliance.org), programs can even be reused languages, with the rich typing provided by the component model interfaces.

## Example Programs

### Build Requirements

Besides the latest stable Rust toolchain, you will also need to install:

- [cargo component v0.2.0](https://github.com/bytecodealliance/cargo-component#installation), a Cargo extension for building Wasm components.
- [wasm-tools](https://github.com/bytecodealliance/wasm-tools#installation), to be used by `cargo-component`.

These can be installed as follows:

```bash
cargo install cargo-component --version 0.2.0 &&
cargo install wasm-tools
```

Alternatively you can build them using the included Dockerfile:
```bash
docker build --build-arg PACKAGE=<example name> --output=example-binary --target=binary .
```
This will build the specified example and put comiled `.wasm` in the director `./example-binary`.

### A Barebones Program: [`template-barebones`](./examples/barebones/src/lib.rs)

An example of a barebones program is at [`examples/barebones/src/lib.rs`](./examples/barebones/src/lib.rs). This example does a simple check on the length of the message to be signed.

You can compile the program by running:

```bash
cargo component build --release -p template-barebones --target wasm32-unknown-unknown
```

This builds the program as a Wasm component at `target/wasm32-unknown-unknown/release/template_barebones.wasm`.

### Example Custody Program with Config: [`example-basic-transaction`](./examples/basic-transaction/src/lib.rs)

This example validates that an an EVM transaction request recipient exists on a list of allowlisted addresses. It also uses a configuration, which allows the user to modify the allowlisted addresses without having to recompile the program (i.e. update the allowlist from the browser).

You can compile the program by running:

```bash
cargo component build --release -p example-basic-transaction --target wasm32-unknown-unknown
```

## Writing your own programs

You can get started with a template program using cargo-generate.

Install cargo-generate:

```bash
cargo install cargo-generate
```

```bash
cargo generate entropyxyz/programs --name my-program
```

You template program is now in the `./my-program` directory and ready to be edited. You can run tests as you would a normal rust project with `cargo test`.

If you want to make your program publicly available and open source, it is recommended to build it with the Dockerfile included in the template. This makes it possible for others to verify that the source code does correspond to the on-chain binary.

```
docker build --output=binary-dir --target=binary .
```

This will compile your program and put the `.wasm` binary file in `./binary-dir`. 

## Running Tests

Before running the runtime tests, you need to build the `template-barebones`, `infinite-loop` and `example-custom-hash` components. To do this, execute:

```bash
cargo component build --release -p template-barebones -p infinite-loop -p example-custom-hash --target wasm32-unknown-unknown`
```

This will create the components in `target/wasm32-unknown-unknown/release/`.

## Licensing

For the most part, the code in this repository is licensed under [AGPL-3.0](./LICENSE).

There are some exceptions however:

- The original code in the `examples/risc0-zkvm-verification` crate comes from RISC Zero's [`risc0`](https://github.com/risc0/risc0) project, which is licensed under
  `Apache License 2.0`.

Modifications made by Entropy to these crates are licensed under `AGPL-3.0`.
