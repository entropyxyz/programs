# `ec-core`

This contains core traits and types for writing modular constraints code, including constraints, runtimes, architectures (for writing architecture-agnostic constraints and dynamic parsing) and signature-request interfaces.

## `.wit`

User applications can generate and use the required WITs in two ways:

1. `cargo component` - prefered, since this doesn't require the user to build the wasm-component manually;
2. reexported from `ec-core` via `wit-bindgen` - this is a fallback for when `cargo component` is not available.
