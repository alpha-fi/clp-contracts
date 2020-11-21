export MY_ACC=robertz.testnet
export NMASTER_ACC=nearswap.testnet
export NCLP_ACC=beta-1.nearswap.testnet
export GOLD_ACC=gold.nearswap.testnet
export BAT_ACC=bat.nearswap.testnet
export USD_ACC=usd.nearswap.testnet
export USD24_ACC=usd24.nearswap.testnet

for a in $GOLD_ACC $BAT_ACC $USD24_ACC; do
    echo deploying for $a;
    near delete $a $NMASTER_ACC
    near create-account $a --masterAccount $NMASTER_ACC
    near deploy --wasmFile ./res/nep21_mintable.wasm --accountId $a
    near call $a new '{"owner_id": "nearswap.testnet", "total_supply": "1", "decimals": 24}' --accountId $MY_ACC
done;


# near call $GOLD_ACC new '{"owner_id": "nearswap.testnet", "total_supply": "1", "decimals": 24}' --accountId $MY_ACC
# near call $BAT_ACC new '{"owner_id": "nearswap.testnet", "total_supply": "1", "decimals": 24}' --accountId $MY_ACC
# near call $USD24_ACC new '{"owner_id": "nearswap.testnet", "total_supply": "1", "decimals": 24}' --accountId $MY_ACC

# near call $USD_ACC new '{"owner_id": "nearswap.testnet", "total_supply": "1", "decimals": 6}' --accountId $MY_ACC
