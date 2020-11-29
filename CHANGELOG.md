* UNRELEASED

+ renamed `transfer_to_sc` to `transfer_call`
+ renamed swap functions to use nep21 instead of token (`swap_near_to_token_exact_in` -> `swap_near_to_nep21_exact_in`). Note: we don't need to rename price functions because they don't depend on the token transfer function.
+ renamed `swap_tokens_exact_in` to `swap_nep21s_exact_in`



* beta-1

First Testnet version with Uniswap constant product and fee model
