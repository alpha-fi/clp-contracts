build:
	@env 'RUSTFLAGS=-C link-arg=-s' cargo +stable build --target wasm32-unknown-unknown --release
	@cp target/wasm32-unknown-unknown/release/nep21_mintable.wasm res/

# init:
# near call $GOLD_ACC new '{"owner_id": "nearswap.testnet", "total_supply": "1000000", "decimals": 24}' --accountId $MY_ACC
#
