# To build this program, and put the .wasm binary in the directory 'output':
# docker build --output=binary-dir .
FROM entropyxyz/build-entropy-programs:v0.0.1 AS base

WORKDIR /usr/src/programs
COPY . .

RUN cargo component build --release --target wasm32-unknown-unknown

FROM scratch AS binary
COPY --from=base /usr/src/programs/target/wasm32-unknown-unknown/release/*.wasm /
