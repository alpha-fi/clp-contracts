# NEAR-CLP Usage

Our smart contracts are implemented in Rust. To investigate our API, please refer to the exported Rust crate documentation. All public functions are exported through a blockchain interface.
Currently, the crate documentation it's checked-in the github repository. To see it in a friendly way you need to clone this repository and open the `contract/target/doc/near_clp/index.html` file in your web browser.

You can call any `NEARClp` public function using RPC or the [near-cli](https://github.com/near/near-cli) tool. Example:

```
near view beta-1.nearswap.testnet price_near_to_token_out '{"token": "gold.nearswap.testnet", "tokens_out": "100000000000000000000"}' --accountId me.testnet
```


## Webapp

TODO: describe:
* current functionality
* limitation
* wallet integration
* ethereum integration


## nearswap-cli

We implemented a dedicated [CLI tool](https://github.com/luciotato/near-clp-beta-cli/) to directly interact with our smart contract. Please refer to the project page to see setup and usage instructions.
