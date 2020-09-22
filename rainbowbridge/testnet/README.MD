# Rainbow bridge setup 

1. Install rainbow-bridge-cli using `npm i -g rainbow-bridge-cli`
2. `rainbow clean` - deletes any previous files created by rainbow
3. create `.rainbow/config.json` in home directory

Check [rainbow-bridge-cli](https://github.com/near/rainbow-bridge-cli) repository for more details.

## NEAR testnet and rinkeby


1. Paste the file `config.json` in `.rainbow/config.json`
1. change `nearMasterAccount, nearMasterSk, ethMasterSk` if you want, it will work even if you don't.
1. Run `rainbow transfer-eth-erc20-to-near --amount 10 --eth-sender-sk <eth_token_holder_address> --near-receiver-account <near_token_holder_account>`
1. You should see a successful transfer of ERC-20 tokens from Ethereum chain to NEAR chain.


## Generic config

1. Paste the data of file `generic-config` in `.rainbow/config.json`
1. Set up a infura node on rinkeby network, Paste the link in `ethNodeUrl`
1. Change `nearMasterAccount, nearMasterSk, ethMasterSk` to your Sk and accounts.
1. Put address of ERC-20 token you want to bridge in `ethErc20Address`
1. `rainbow init-eth-locker` deploy locker contract
1. `rainbow init-near-fun-token` create nep21 token account and deploy contract
1. Run `rainbow transfer-eth-erc20-to-near --amount 10 --eth-sender-sk <eth_token_holder_address> --near-receiver-account <near_token_holder_account>`
