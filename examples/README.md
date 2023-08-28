# `examples`

This contains examples of programs.

## Setup

Building a program requires `cargo component`, which is the cargo helper for building Wasm components. As a reminder, Entropy programs are simply Wasm components that satisfy an interface.

## Barebones Example

`template-barebones` provides a basic example of a program that has the minimal dependencies to build a program (). It simply checks that the data to be signed is less than 10 bytes.

## Basic Transaction Example

`template-basic-transaction` provides an example of how to constrain an Ethereum transaction request to a recipient in an access control list.

## Building Components

To build the `barebones` component, run `cargo component build --release -p template-barebones --target wasm32-unknown-unknown`. This will create the files needed for testing at `target/wasm32-unknown-unknown/release/`.
