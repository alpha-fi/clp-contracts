# Continuous Liquidity Provider smart-contracts

Continuous Liquidity Provider smart-contracts for NEAR Blockchain.

Liquidity pools hold one or more token reserves. These reserves are programmed to perform trades according to a predetermined formula which continuously measures the supply of tokens held in each reserve.

Requirements and related discussion is available in [GitHub](https://github.com/near/bounties/issues/6).

## Building and development

To build run:
```bash
make build
```

### Testing

To run simulation tests, we need extra dependencies:
`libclang-dev`, `llvm`


To test run:
```bash
make test
```




## Changes to Uniswap v1

#### Deadline

+ removed `deadline` argument from `remove_liqidity` and `add_liqidity**. NEAR process transactions in shards and we don't expect to have stalling transactions.

#### Factory pattern for CLPs

Factory could allow changing a contract which creates a new pool. But this doesn't solve the problem of updating already deployed pools.

**Benefits of not having a pool factory**. In fact there are clear benefits of dropping the factory pattern:

1. Since NEAR cross contract calls are async and more complicated, a simple exchange can be do much easier
1. Removing some attack vectors: front-runners could manipulate a price when they see a cross contract swap (token-token).
1. With all liquidity residing in a single contract we can have a shared NEAR pool (version 2). I'm still building that idea. But this allows few new opportunities.
1. We can reduce fees for token-token swaps (currently this is a double of token-near swap fee - as in the Uniswap).

#### Mutable Fees

Highly volatile markets should have bigger fees to recompensate impermanent losses.
However this leads to a problem : who can change fees.
Good news is that we can build on that without changing the CLP contract. An entity who is allowed to do modifications can be a contract and owner / contract can always change to a new owner / contract, enhancing the governance. At the beginning this can be managed by a foundation. This is fine - centralized governance is good for start. And foundation is a perfect candidate.
