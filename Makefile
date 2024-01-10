# Vercel sets the `HOME` env var weirdly, so we define a few extra
# things to make sure it installs okay.
.PHONY: vercel-rustup
vercel-rustup:
		curl --proto '=https' --tlsv1.2 \
			--silent --show-error --fail https://sh.rustup.rs \
			| RUSTUP_HOME=/vercel/.rustup HOME=/root sh -s -- -y
		cp -R /root/.cargo /vercel/.cargo

# Installs `rustup` in a typical case.
.PHONY: rustup
rustup:
		curl --proto '=https' --tlsv1.2 \
			--silent --show-error --fail https://sh.rustup.rs \
			| sh -s -- -y

.PHONY: rust
rust:
		export PATH="${PATH}:${HOME}/.cargo/bin" rustup default stable \
		&& rustup show \
		&& cargo install --git https://github.com/bytecodealliance/cargo-component --locked cargo-component \
		apt update -y && apt-get install -y libssl-dev openssl pkg-config \
		&& cargo install cargo-risczero \
		&& cargo risczero install

# This target is specifically for generating API documentation from
# within a Vercel.com Project. It is used as the Projects `installCommand`.
vercel-install-api-docs :: vercel-rustup rust

# The Vercel Project's `buildCommand` is defined here.
vercel-build-api-docs ::
		export PATH="${PATH}:${HOME}/.cargo/bin" \
			&& cargo doc --workspace --exclude example-risc0 --release --no-deps
