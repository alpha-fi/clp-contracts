test:
	cargo test  --

build:
#	@env 'RUSTFLAGS=-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
	@cargo build --target wasm32-unknown-unknown --release

link-to-web:
	@mkdir -p ../out
	@cd ../out && ln -s ../contract/target/wasm32-unknown-unknown/release/near_clp.wasm main.wasm

build-quick:
#	@env 'RUSTFLAGS=-C link-arg=-s' cargo build
	@cargo build

deploy:
	near deploy --wasmFile target/wasm32-unknown-unknown/release/near_clp.wasm --accountId $(NEAR_ACCOUNT)
	near call $(NEAR_ACCOUNT) new '{"owner_id": "'$ID'", "total_supply": "1000000000000000"}' --accountId $(NEAR_ACCOUNT)
