# NEAR-CLP Usage

Our smart contract are implemented in Rust. To investigate our API, please refer to the exported crate documentation. Currently, it's checked in the github repository. To see it in a friendly way you need to clone this repository and open the `contract/target/doc/near_clp/index.html` file in your web browser.

All public functions are exported through a blockchain interface and you can call them using RPC or the [near-cli](https://github.com/near/near-cli) tool. Example:

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

By using nearswap-cli, liquidity providers can operate the contract from the command line: create a new pool, config the pool, add and withdraw liquidity, retrieve pool status and prices.

The nearswap-cli also can interact with the NEP-21 contracts, by using the command `inc_allowance` you can call directly to a NEP21 token contract from the nearswap-cli

Example:

```
> nearswap list_pools

View call: beta-1.nearswap.testnet.list_pools()
[ 'gold.nearswap.testnet', 'usd.nearswap.testnet', [length]: 2 ]

```
```
> nearswap pool_info { token:gold.nearswap.testnet }

View call: beta-1.nearswap.testnet.pool_info({"token":"gold.nearswap.testnet"})
{
  ynear: "12998486894407298000000000",
  reserve: "221800000030020300000000",
  total_shares: "1000000000022852100000"
}
```

Call the NEP21 token contract
```
> nearswap inc_allowance usd24.nearswap.testnet 500
```

Please go to [nearswap CLI tool](https://github.com/luciotato/near-clp-beta-cli/) for full instructions.
