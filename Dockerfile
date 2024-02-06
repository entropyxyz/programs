FROM rust:1.67 AS base
ARG PACKAGE=template-barebones

WORKDIR /usr/src/programs
COPY . .

RUN cargo install cargo-component --version 0.2.0
RUN cargo install wasm-tools
RUN cargo component build --release -p $PACKAGE --target wasm32-unknown-unknown

FROM scratch AS binary
COPY --from=base /usr/src/programs/target/wasm32-unknown-unknown/release /
# TODO this copies all files, should really be only $PACKAGE.wasm - but there is an issue with snake case vs kebab case
