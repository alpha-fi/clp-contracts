# Continuous Liquidity Provider smart-contracts

Continuous Liquidity Provider smart-contracts for NEAR Blockchain.

Liquidity pools hold one or more token reserves. These reserves are programmed to perform trades according to a predetermined formula which continuously measures the supply of tokens held in each reserve.

## Building and development

To build run:
```bash
./build.sh
```

To test run:
```bash
cargo test --package status-message -- --nocapture
```


## Changes to Uniswap v1

+ removed `deadline` argument from `remove_liqidity` and `add_liqidity`. NEAR process transactions in shards and we don't expect to have stalling transactions.
