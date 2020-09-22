# Rainbow bridge on testnet and rinkeby network (Using)

1. Install rainbow-bridge-cli using `npm i -g rainbow-bridge-cli`

2. `rainbow clean` - deletes any previous files created by rainbow

3. create `.rainbow/config.json` in home directory

4. Paste the file `config.json` in `.rainbow/config.json`

5. change `nearMasterAccount, nearMasterSk, ethMasterSk` if you want, it will work even if you don't.

6. Run `rainbow transfer-eth-erc20-to-near --amount 10 --eth-sender-sk <eth_token_holder_address> --near-receiver-account <near_token_holder_account>`

7. You should see a successful transfer of ERC-20 tokens from Ethereum chain to NEAR chain.

# Setting up bridge for your token

1. Install rainbow-bridge-cli using `npm i -g rainbow-bridge-cli`

2. `rainbow clean` - deletes any previous files created by rainbow

3. create `.rainbow/config.json` in home directory

4. Paste the data of file `generic-config` in `.rainbow/config.json`

5. Set up a infura node on rinkeby network, Paste the link in `ethNodeUrl`

6. Change `nearMasterAccount, nearMasterSk, ethMasterSk` to your Sk and accounts.

7. Put address of ERC-20 token you want to bridge in `ethErc20Address`

8. `rainbow init-eth-locker` deploy locker contract
 
9. `rainbow init-near-fun-token` create nep21 token account and deploy contract

10. Run `rainbow transfer-eth-erc20-to-near --amount 10 --eth-sender-sk <eth_token_holder_address> --near-receiver-account <near_token_holder_account>`