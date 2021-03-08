include ../../Makefile_common.mk

build:
	@env 'RUSTFLAGS=-C link-arg=-s' cargo build --lib --target wasm32-unknown-unknown --release
	@cd ../..; cp target/wasm32-unknown-unknown/release/*.wasm ./res/

# init:
# near call $GOLD_ACC new '{"owner_id": "nearswap.testnet", "total_supply": "1000000", "decimals": 24}' --accountId $MY_ACC
#
