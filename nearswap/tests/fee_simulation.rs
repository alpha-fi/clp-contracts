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
fn fee_simulation_test() {
    let root = init_simulator(None);
    let lp1 = root.create_user("lp1".to_string(), to_yocto("100"));
    let lp2 = root.create_user("lp2".to_string(), to_yocto("100"));
    let alice = root.create_user("alice".to_string(), to_yocto("100"));
    let nearswap = deploy!(
        contract: NearSwapContract,
        contract_id: clp_contract(),
        bytes: &NEARSWAP_WASM_BYTES,
        signer_account: root,
        init_method: new(to_va(root.account_id.clone()))
    );

    let token1 = sample_token(&root, dai(), vec![clp_contract()]);
    // mint for liquidity providers
    call!(
        root,
        token1.mint(to_va(lp1.account_id.clone()), to_yocto("1000").into())
    )
    .assert_success();
    call!(
        root,
        token1.mint(to_va(lp2.account_id.clone()), to_yocto("1000").into())
    )
    .assert_success();
    print!("Token minted for liquidators");
    call!(
        root,
        nearswap.extend_whitelisted_tokens(vec![to_va(dai())])
    );

    // Pool creation by root account
    call!(
        root,
        nearswap.create_pool(to_va("dai".into())),
        deposit = to_yocto("1")
    )
    .assert_success();

    // Register account lp1
    call!(
        lp1,
        nearswap.storage_deposit(None, Some(true)),
        deposit = to_yocto("1")
    )
    .assert_success();
    // Register account lp2
    call!(
        lp2,
        nearswap.storage_deposit(None, Some(true)),
        deposit = to_yocto("1")
    )
    .assert_success();
    print!("Account Registered!");
    // Deposit near in account deposit lp1
    call!(
        lp1,
        nearswap.storage_deposit(None, None),
        deposit = to_yocto("35")
    )
    .assert_success();
    // Deposit near in account deposit lp2
    call!(
        lp2,
        nearswap.storage_deposit(None, None),
        deposit = to_yocto("35")
    )
    .assert_success();

    // Add to accounts whitelist
    call!(
        lp1,
        nearswap.add_to_account_whitelist(&vec![to_va(dai())])
    )
    .assert_success();
    call!(
        lp2,
        nearswap.add_to_account_whitelist(&vec![to_va(dai())])
    )
    .assert_success();

    // Depositing tokens in account deposit
    call!(
        lp1,
        token1.ft_transfer_call(to_va(clp_contract()), to_yocto("100").into(), None, "".to_string()),
        deposit = 1
    )
    .assert_success();
    call!(
        lp2,
        token1.ft_transfer_call(to_va(clp_contract()), to_yocto("100").into(), None, "".to_string()),
        deposit = 1
    )
    .assert_success();

    print!("Token deposited in contract account deposit");

    // Adding liquidity
    call!(
        lp1,
        nearswap.add_liquidity(dai(), U128(to_yocto("9")), U128(to_yocto("90")), U128(0)),
        deposit = 1
    )
    .assert_success();
    print!("OK ---- ");
    call!(
        lp2,
        nearswap.add_liquidity(dai(), U128(to_yocto("1")), U128(to_yocto("10")), U128(0)),
        deposit = 1
    )
    .assert_success();
    print!("Added Liquidity!");
    // Register alice before swapping
    call!(
        alice,
        nearswap.storage_deposit(None, Some(true)),
        deposit = to_yocto("1")
    )
    .assert_success();
    // Deposit near in account deposit
    call!(
        alice,
        nearswap.storage_deposit(None, None),
        deposit = to_yocto("5")
    )
    .assert_success();
    call!(
        alice,
        nearswap.add_to_account_whitelist(&vec![to_va(dai())])
    )
    .assert_success();
    print!("Alice Registered!");
    // Alice buy token by paying 5 NEAR
    let mut before_swap_token = view!(
        nearswap.get_deposit_token(alice.account_id.clone(), dai())
    ).unwrap_json::<U128>();
    assert_close(before_swap_token, to_yocto("0"), 0);

    let price_n2t = view!(
            nearswap.price_near_to_token_in(dai(), U128(to_yocto("5")))
        ).unwrap_json::<U128>();

    call!(
        alice,
        nearswap.swap_near_to_token_exact_in(U128(to_yocto("5")), dai(), price_n2t),
        deposit = 1
    ).assert_success();

    let mut after_swap_token = view!(
            nearswap.get_deposit_token(alice.account_id, dai())
        ).unwrap_json::<U128>();

    assert_eq!(
        to_u128(before_swap_token) + to_u128(price_n2t),
        to_u128(after_swap_token), "Near to token swap unsuccessful");
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