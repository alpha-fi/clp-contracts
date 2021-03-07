export NCLP_ACC=beta-1.nearswap.testnet

test:
	@cargo test

test-debug:
# "--" allows to pass extra arguments
# "--nocapture" disables stdout capturing (testes print all println)
	@RUST_BACKTRACE=1 cargo test  -- --nocapture

test-unit:
	@cargo test --lib -- --nocapture
# run specific tests: cargo test --lib <testname>  -- --nocapture


build:
# more about flags: https://github.com/near-examples/simulation-testing#gotchas
# env setting instruments cargo to optimize the the build for size (link-args=-s)
	@env 'RUSTFLAGS=-C link-arg=-s' cargo build --all --lib --target wasm32-unknown-unknown --release
	@cp target/wasm32-unknown-unknown/release/*.wasm ./res/

# link-to-web:
# 	@mkdir -p ../out
# 	@cd ../out && ln -s ../contract/target/wasm32-unknown-unknown/release/near_clp.wasm main.wasm

build-doc:
	cargo doc


#################
#   NEARswap

deploy-nearswap:
	near deploy --wasmFile target/wasm32-unknown-unknown/release/near_clp.wasm --accountId $(NCLP_ACC)  --initFunction "new" --initArgs "{\"owner\": \"$NMASTER_ACC\"}"

init-nearswap:
	@echo near sent ${NMASTER_ACC} ${NCLP_ACC} 200
# no need to call new because we call it during the deployment
#	@echo near call ${NCLP_ACC} new "{\"owner\": \"$NMASTER_ACC\"}" --accountId ${NCLP_ACC}
