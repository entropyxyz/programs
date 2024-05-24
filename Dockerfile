# This is for building the example programs in this repo.
FROM entropyxyz/build-entropy-programs:v0.0.1 AS base
ARG PACKAGE=template-barebones

WORKDIR /usr/src/programs
COPY . .

RUN cargo component build --release -p $PACKAGE --target wasm32-unknown-unknown

FROM scratch AS binary
COPY --from=base /usr/src/programs/target/wasm32-unknown-unknown/release/*.wasm /
