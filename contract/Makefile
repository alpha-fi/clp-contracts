export NCLP_ACC=beta-1.nearswap.testnet

test:
	@cargo +nightly test

test-debug:
	@RUST_BACKTRACE=1 cargo +nightly test  -- --nocapture

test-unit:
	@cargo +nightly test --lib -- --nocapture


build:
# more about flags: https://github.com/near-examples/simulation-testing#gotchas
# we don't add this flags to `cargo.yaml` build section because it would affect
# tests as well, which we don't need
	@env 'RUSTFLAGS=-C link-arg=-s' cargo +stable build --lib --target wasm32-unknown-unknown --release
	@cp target/wasm32-unknown-unknown/release/near_clp.wasm res/

# link-to-web:
# 	@mkdir -p ../out
# 	@cd ../out && ln -s ../contract/target/wasm32-unknown-unknown/release/near_clp.wasm main.wasm


deploy:
	near deploy --wasmFile target/wasm32-unknown-unknown/release/near_clp.wasm --accountId $(NCLP_ACC)

init:
	@echo near sent ${NMASTER_ACC} ${NCLP_ACC} 200
	@echo near call ${NCLP_ACC} new "{\"owner\": \"$NMASTER_ACC\"}" --accountId ${NCLP_ACC}
