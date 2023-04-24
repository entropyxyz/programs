# Entropy Constraints

Entropy is a decentralized platform for securely managing digital assets. It uses threshold cryptography to sign messages, like transaction requests, and users write constraints to define the conditions under which signature requests are valid.

This repository contains libraries, toolchains, utilities, and specifications for writing, configuring, building, and testing the Wasm-based constraints for Entropy. Constraints can be written in any language that compiles to WebAssembly, including LLVM-supported languages like Rust, AssemblyScript, Zig, C, etc. Resultingly, constraints can be reused across *all* of these languages.

## Requirements

To get started, you will need the stable Rust toolchain with the `wasm32-unknown-unknown` target installed.

## Getting started

To get started, clone this repository and build an example constraints package:

```bash
git clone https://github.com/entropyxy/constraints
cd constraints/examples/acl
cargo build --release
```

This will build a valid constraints program for the Entropy constraints runtime, which can be uploaded to the Entropy Network.

To write your own constraints, you can use any language that compiles to WebAssembly, such as Rust, C++, or AssemblyScript. You will need to define a function that takes a transaction request or message as input and returns a result indicating whether the constraints are satisfied.

## Conventions

Packages should use the `ec` prefix, which stands for "Entropy Constraints". For example, `ec-core` is the "Entropy Constraints Core" package, and `ec-acl` is the "Entropy Constraints Access Control List" package.

<!-- 
The function should be exported and have the following signature:

Copy code
int32_t check_constraints(uint8_t* request, size_t request_len);
Once you have written your constraints function, you can compile it to WebAssembly using your language's WebAssembly compiler. For example, if you are using Rust, you can compile your code as follows:

```sh
rustc --target wasm32-wasi -O --crate-type=cdylib my_constraints.rs
```
This will compile your code to a Wasm module that can be imported into the DACP.

## Usage
To use the constraints system in the Entropy, you will need to import the basic constraints package library and any additional constraints modules that you have written. You can do this using the wasmtime runtime and the wasmtime-interface-types crate.

Here's an example of how to load and call a constraints function:

```rust
use wasmtime_interface_types::{HostFunctions, WasmtimeCxt};
use wasmtime_runtime::Instance;

// Load the basic constraints package library
let module = wasmtime_runtime::Module::from_file("basic_constraints.wasm")?;

// Load the constraints module
let module2 = wasmtime_runtime::Module::from_file("my_constraints.wasm")?;

// Create an instance of the constraints module
let instance = Instance::new(&module2, &[])?;

// Define the transaction request or message as a byte array
let request = vec![0x01, 0x02, 0x03];

// Define the host functions that the constraints function can call
let host_fns = HostFunctions::new();

// Create a context for the Wasmtime runtime
let cxt = WasmtimeCxt::new(&host_fns);

// Call the constraints function
let check_constraints = instance
    .get_typed_func::<(i32, i32), i32, _>(&mut *cxt, "check_constraints")?;
let result = check_constraints.call(&mut *cxt, (request.as_ptr() as i32, request.len() as i32))?;

// Check the result of the constraints function
if result == -->
``` -->
