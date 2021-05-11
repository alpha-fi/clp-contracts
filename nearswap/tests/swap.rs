use std::convert::TryFrom;

use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::AccountId;
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, view, ContractAccount, UserAccount};

use near_sdk_sim::transaction::ExecutionStatus;
use nearswap::{NearSwapContract, PoolInfo};
use std::collections::HashMap;
use sample_token::ContractContract as SampleToken;

mod nep141_utils;
use nep141_utils::*;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    NEARSWAP_WASM_BYTES => "../res/nearswap.wasm",
}

#[test]
fn swap_test() {
    let root = init_simulator(None);
    let owner = root.create_user("owner".to_string(), to_yocto("100000"));
    let nearswap = deploy!(
        contract: NearSwapContract,
        contract_id: clp_contract(),
        bytes: &NEARSWAP_WASM_BYTES,
        signer_account: owner,
        init_method: new(to_va("owner".to_string()))
    );
    let token1 = sample_token(&owner, dai(), vec![clp_contract()]);
    let token2 = sample_token(&owner, eth(), vec![clp_contract()]);
    call!(
        owner,
        nearswap.extend_whitelisted_tokens(vec![to_va(dai()), to_va(eth())])
    );
    call!(
        owner,
        nearswap.create_pool(to_va("dai".into())),
        deposit = to_yocto("1")
    )
    .assert_success();
    call!(
        owner,
        nearswap.create_pool(to_va("eth".into())),
        deposit = to_yocto("1")
    )
    .assert_success();

    // Register account
    call!(
        owner,
        nearswap.storage_deposit(None, Some(true)),
        deposit = to_yocto("1")
    )
    .assert_success();
    // Deposit more near in account deposit
    call!(
        owner,
        nearswap.storage_deposit(None, None),
        deposit = to_yocto("35")
    )
    .assert_success();

    call!(
        owner,
        nearswap.add_to_account_whitelist(&vec![to_va(dai()), to_va(eth())])
    )
    .assert_success();
    // Deposit tokens
    call!(
        owner,
        token1.ft_transfer_call(to_va(clp_contract()), to_yocto("105").into(), None, "".to_string()),
        deposit = 1
    )
    .assert_success();
    call!(
        owner,
        token2.ft_transfer_call(to_va(clp_contract()), to_yocto("105").into(), None, "".to_string()),
        deposit = 1
    )
    .assert_success();

    // Add liquidity: dai pool
    call!(
        owner,
        nearswap.add_liquidity(dai(), U128(to_yocto("10")), U128(to_yocto("10")), U128(0)),
        deposit = 1
    )
    .assert_success();

    // Add liquidity: eth pool
    call!(
        owner,
        nearswap.add_liquidity(eth(), U128(to_yocto("10")), U128(to_yocto("10")), U128(0)),
        deposit = 1
    )
    .assert_success();
   
    // Swap near to dai
    let mut before_swap_token = view!(
        nearswap.get_deposit_token("owner".to_string(), dai())
    ).unwrap_json::<U128>();
    assert_close(before_swap_token, to_yocto("95"), 0);

    let price_n2t = view!(
            nearswap.price_near_to_token_in(dai(), U128(to_yocto("1")))
        ).unwrap_json::<U128>();

    call!(
        owner,
        nearswap.swap_near_to_token_exact_in(U128(to_yocto("1")), dai(), price_n2t),
        deposit = 1
    ).assert_success();

    let mut after_swap_token = view!(
            nearswap.get_deposit_token("owner".to_string(), dai())
        ).unwrap_json::<U128>();

    assert_eq!(
        to_u128(before_swap_token) + to_u128(price_n2t),
        to_u128(after_swap_token), "Near to token swap unsuccessful");


    // Swap dai to near
    let before_swap_near = view!(
        nearswap.get_deposit_near("owner".to_string())
    ).unwrap_json::<U128>();

    let price_t2n = view!(
            nearswap.price_token_to_near_in(dai(), U128(to_yocto("1")))
        ).unwrap_json::<U128>();

    call!(
        owner,
        nearswap.swap_token_to_near_exact_in(dai(), U128(to_yocto("1")), price_t2n),
        deposit = 1
    ).assert_success();

    let after_swap_near = view!(
            nearswap.get_deposit_near("owner".to_string())
        ).unwrap_json::<U128>();

    assert_eq!(
        to_u128(before_swap_near) + to_u128(price_t2n), to_u128(after_swap_near)
            , "Token to near swap unsuccessful");


    // Swap dai to eth(token to token)
    before_swap_token = view!(
        nearswap.get_deposit_token("owner".to_string(), eth())
    ).unwrap_json::<U128>();

    let price_t2t = view!(
            nearswap.price_token_to_token_in(dai(), eth(), U128(to_yocto("1")))
        ).unwrap_json::<U128>();

    call!(
        owner,
        nearswap.swap_tokens_exact_in(dai(), U128(to_yocto("1")), eth(), price_t2t),
        deposit = 1
    ).assert_success();

    after_swap_token = view!(
            nearswap.get_deposit_token("owner".to_string(), eth())
        ).unwrap_json::<U128>();

    assert_eq!(
        to_u128(before_swap_token) + to_u128(price_t2t), to_u128(after_swap_token)
            , "Token to token swap unsuccessful");
}

fn to_u128(num: U128) -> u128 {
    return num.into();
}

fn assert_close(a1: U128, a2: u128, margin: u128) {
    let a1: u128 = a1.into();
    let diff = if a1 > a2 { a1 - a2 } else { a2 - a1 };
    assert!(
        diff <= margin,
        format!(
            "Expect to be close (margin={}):\n  left: {}\n right: {}\n  diff: {}\n",
            margin, a1, a2, diff
        )
    )
}