FROM rust:1.75 AS base

WORKDIR /usr/src/programs

RUN rustup update stable
RUN rustup target add wasm32-unknown-unknown
RUN rustup target add wasm32-wasi
RUN cargo install cargo-component --version 0.2.0
RUN cargo install wasm-tools
