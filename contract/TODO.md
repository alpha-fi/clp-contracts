# TODO

+ replace UnorderedMap with LookupMap!

## Review and fix exploits

Our current implementation is targeted functionality, not a full security.
We firstly want to test the behavior and validate the approach.

Current code has few vulnerabilities due to asynchronous calls. We don't cover all edge cases. Also we should do local state changes only after a successful remote calls.

+ `token->* swaps`, `add_liquidity` functions.
  Currently the exception in the `transfer_from` function is not handled.
+ Validate the reception of the MultiToken transfers in `transfer_to_sc` before updating the local state.

Handle exceptions in _foreign_ contracts. Example: [StackOverflow question](https://stackoverflow.com/questions/62987417).


Tips:
+ https://github.com/nearprotocol/NEPs/pull/26

## Tests

+ inspect near balances using `runtime.view_account`
+ more more more tests


## CLP related functionality

+ add non integer type for balances to calculate expected amount. We can do it using [decimate](https://crates.io/crates/decimate)
+ change fees calculation for token 2 token swaps

+ `set_pool` should remove the pool if it's empty (as it's done by nep21)

### Economics

+ Review and add missing storage costs calculations.

### Multi Fungible Token standard

+ Review the standard and finalize the implementation.
+ We want to add operator related functions, which could do some breaking changes.

## Code organization

+ multiple contracts: https://stackoverflow.com/questions/64110056/how-to-build-and-deploy-multiple-contracts-in-near-protocol
