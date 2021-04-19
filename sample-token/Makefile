#################
#   Sample Token    #

include ../Makefile_common.mk

export TOKEN1=token1.nearswap.testnet

deploy-token:
	near deploy --wasmFile target/wasm32-unknown-unknown/release/sample_token.wasm --accountId $(TOKEN1)  --initFunction "new"
