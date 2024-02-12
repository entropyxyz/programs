<!-- Generated with cargo generate entropyxyz/programs -->
# {{project-name}}

## Running tests

`cargo test`

## Building the program

Get the necessary build tools with:
```bash
cargo install cargo-component --version 0.2.0 &&
cargo install wasm-tools
```

Then build with:
```bash
cargo component build --release --target wasm32-unknown-unknown`
```

The `.wasm` binary can be found in `./target/wasm32-unknown-unknown/release`

## Building with docker

If you want to make your program publicly available and open source, it is recommended to build it with the Dockerfile included in the template. This makes it possible for others to verify that the source code does correspond to the on-chain binary.

```
docker build --output=binary-dir .
```

This will compile your program and put the `.wasm` binary file in `./binary-dir`. 
