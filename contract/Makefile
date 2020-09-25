export NCLP_ACC=beta-1.nearswap.testnet

test:
	cargo +nightly test  --
# Test output can be recovered (e.g., for debugging) by passing `--nocapture`
# cargo test  -- --nocapture

test-unit:
	@cargo +nightly test --lib

build:
# more about flags: https://github.com/near-examples/simulation-testing#gotchas
	@env 'RUSTFLAGS=-C link-arg=-s' cargo +stable build --target wasm32-unknown-unknown --release
	@cp target/wasm32-unknown-unknown/release/near_clp.wasm res/

link-to-web:
	@mkdir -p ../out
	@cd ../out && ln -s ../contract/target/wasm32-unknown-unknown/release/near_clp.wasm main.wasm

build-quick:
	@cargo build

deploy:
	@near deploy --wasmFile target/wasm32-unknown-unknown/release/near_clp.wasm --accountId $(NCLP_ACC)

init:
	@echo near call $(NCLP_ACC) new '{"owner_id": "nearswap.testnet"}' --accountId $(NCLP_ACC)
