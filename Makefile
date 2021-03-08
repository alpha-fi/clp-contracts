build-doc:
	cargo doc

build-all:
	@env 'RUSTFLAGS=-C link-arg=-s' cargo build --all --lib --target wasm32-unknown-unknown --release
	@cp target/wasm32-unknown-unknown/release/*.wasm ./res/
