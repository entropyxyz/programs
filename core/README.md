# `entropy-programs-core`

This contains core traits and types for writing modular programs code, including programs, runtimes, architectures (for writing architecture-agnostic programs and dynamic parsing) and signature-request interfaces.

## `.wit`

User applications can generate and use the required WITs in two ways:

1. `cargo component` - prefered, since this doesn't require the user to build the wasm-component manually;
2. reexported from `entropy-programs-core` via `wit-bindgen` - this is a fallback for when `cargo component` is not available.
