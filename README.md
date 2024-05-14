# Entropy Programs

Entropy allows the creation of decentralized signing authorities. Signing authorities exist as WebAssembly programs that can ingest a signature request, and a valid request is signed via a threshold signature scheme from a set of at-stake validators. These requests might include cryptocurrency-based transaction requests, certificate signing requests, or other mediums for cryptographic authentication.

This repository contains libraries, toolchains, utilities, and specifications for writing, configuring, building, and testing the Wasm-based applications for Entropy. Programs can be written in any language that compiles to WebAssembly, which includes all LLVM-supported languages like Rust, AssemblyScript, Zig, and C. All the examples in this repository are written in Rust.

## Prerequisites

To edit and compile the programs in the repository, you will need the following tools:

1. The lastest stable Rust toolchain. This can be installed with:

    ```shell
    curl https://sh.rustup.rs -sSf | sh
    ```

1. The [cargo-component v0.2.0](https://github.com/bytecodealliance/cargo-component#installation) extension; used for building Wasm components:

    ```shell
    cargo install cargo-component --locked
    ```

1. The [cargo-generate](https://github.com/cargo-generate/cargo-generate) extension; used to generate project templates:

    ```shell
    cargo install cargo-generate
    ```

1. The [wasm-tools](https://github.com/bytecodealliance/wasm-tools#installation) package. This is used by `cargo-component`:

    ```shell
    cargo install wasm-tools
    ```

1. Verify that you have everything installed by running:

    ```shell
    rustc --version && cargo-component --version && cargo-generate --version && wasm-tools --version
    ```

    This should output something like:

    ```plaintext
    rustc 1.78.0 (9b00956e5 2024-04-29)
    cargo-component 0.11.0 (wasi:040ec92)
    cargo generate 0.21.0
    wasm-tools 1.207.0
    ```

1. [Optional] Install [Docker](https://docs.docker.com/get-docker/) to run the associated Dockerfiles found within the repository.
  

### Basic length-check program 

An example of a barebones program can be found at [`examples/barebones/src/lib.rs`](./examples/barebones/src/lib.rs). This example does a simple check on the length of the message to be signed. You can compile the program by running:

```bash
cargo component build --release -p template-barebones --target wasm32-unknown-unknown
```

This builds the program as a Wasm component at `target/wasm32-unknown-unknown/release/template_barebones.wasm`.

### Custody program with configuration

This example validates that an an EVM transaction request recipient exists on a list of allow-listed addresses. It also uses a configuration which allows the user to modify the allow-listed addresses without having to recompile the program.

You can compile the program by running:

```bash
cargo component build --release -p example-basic-transaction --target wasm32-unknown-unknown
```

## Writing your own programs

You can get started with a template program using `cargo-generate`:

```bash
cargo generate entropyxyz/programs --name my-program --tag testnet
```

Make sure to attach the `--tag testnet` argument. This tells Cargo to use the `testnet` tag in the `github.com/entropyxyz/core` repository.

Your template program is now in the `./my-program` directory and ready to be edited. You can run tests as you would a normal rust project with `cargo test`.

You can compile your program with `cargo component`:

You can generate your types by `cargo run generate-types`. If you change the type names of `UserConfig` or `AuxData`, you will need to change those names in `generate-types`.

```bash
cargo component build --release --target wasm32-unknown-unknown
```

If you want to make your program publicly available and open source, and make it possible for others to verify that the source code corresponds to the on-chain binary, you can build it with the Dockerfile included in the template: 

```bash
docker build --output=binary-dir .
```

This will compile your program and put the `.wasm` binary file in `./binary-dir`. 

## Running Tests

Before running the runtime tests, you need to build the `template-barebones`, `infinite-loop` and `example-custom-hash` components. To do this, execute:

```bash
cargo component build --release -p template-barebones -p infinite-loop -p example-custom-hash --target wasm32-unknown-unknown
```

This will create the components in `target/wasm32-unknown-unknown/release/`.

## Licensing

For the most part, the code in this repository is licensed under [AGPL-3.0](./LICENSE).

There are some exceptions however:

- The original code in the `examples/risc0-zkvm-verification` crate comes from RISC Zero's [`risc0`](https://github.com/risc0/risc0) project, which is licensed under
  `Apache License 2.0`.

Modifications made by Entropy to these crates are licensed under `AGPL-3.0`.
