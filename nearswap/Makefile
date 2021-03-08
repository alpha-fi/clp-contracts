#################
#   NEARswap    #


include ../Makefile_common.mk

export NCLP_ACC=beta-1.nearswap.testnet

deploy-nearswap:
	near deploy --wasmFile target/wasm32-unknown-unknown/release/near_clp.wasm --accountId $(NCLP_ACC)  --initFunction "new" --initArgs "{\"owner\": \"$NMASTER_ACC\"}"

init-nearswap:
	@echo near sent ${NMASTER_ACC} ${NCLP_ACC} 200
# no need to call new because we call it during the deployment
#	@echo near call ${NCLP_ACC} new "{\"owner\": \"$NMASTER_ACC\"}" --accountId ${NCLP_ACC}
