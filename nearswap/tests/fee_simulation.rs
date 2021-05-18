use std::convert::TryFrom;

use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::AccountId;
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, view, ContractAccount, UserAccount};
use uint::construct_uint;

use near_sdk_sim::transaction::ExecutionStatus;
use nearswap::{NearSwapContract, PoolInfo};
use std::collections::HashMap;
use sample_token::ContractContract as SampleToken;

mod simulation_utils;
use simulation_utils::*;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    NEARSWAP_WASM_BYTES => "../res/nearswap.wasm",
}

#[test]
fn fee_simulation_test() {
    let (root, alice, nearswap) = deploy(&"root".to_string());
    let lp1 = root.create_user("lp1".to_string(), to_yocto("100"));
    let lp2 = root.create_user("lp2".to_string(), to_yocto("100"));

    let token1 = sample_token(&root, dai(), vec![clp_contract()]);
    // mint for liquidity providers
    mint(&token1, &lp1, &root, to_yocto("1000"));
    mint(&token1, &lp2, &root, to_yocto("1000"));
    print!("Token minted for liquidity providers");

    call!(
        root,
        nearswap.extend_whitelisted_tokens(vec![to_va(dai())])
    );

    // Pool creation by root account
    call!(
        root,
        nearswap.create_pool(to_va("dai".into()))
    )
    .assert_success();

    // register and add deposit to accounts
    register_deposit_acc(&nearswap, &lp1, to_yocto("35"));
    register_deposit_acc(&nearswap, &lp2, to_yocto("35"));

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
    let lp1_shares = call!(
        lp1,
        nearswap.add_liquidity(dai(), to_yocto_str("9"), to_yocto_str("90"), U128(0)),
        deposit = 1
    )
    .unwrap_json::<U128>();
    let lp2_shares = call!(
        lp2,
        nearswap.add_liquidity(dai(), to_yocto_str("1"), to_yocto_str("10.01"), U128(0)),
        deposit = 1
    )
    .unwrap_json::<U128>();

    print!("Added Liquidity!");
    register_deposit_acc(&nearswap, &alice, to_yocto("25"));
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

    let expected_receive = view!(
            nearswap.price_near_to_token_in(dai(), to_yocto_str("5"))
        ).unwrap_json::<U128>();

    let mut pool_before = view!(nearswap.pool_info(&dai())).unwrap_json::<PoolInfo>();

    call!(
        alice,
        nearswap.swap_near_to_token_exact_in(to_yocto_str("5"), dai(), expected_receive),
        deposit = 1
    ).assert_success();

    let mut after_swap_token = view!(
            nearswap.get_deposit_token(alice.account_id, dai())
        ).unwrap_json::<U128>();

    assert_eq!(
        to_u128(before_swap_token) + to_u128(expected_receive),
        to_u128(after_swap_token), "Near to token swap unsuccessful");

    // Check if fee is deducted - Near deposited into pool must be 0.997 * near amount
    // Fee - 0.3%
    let out = calc_out_expected(
        to_yocto("5")*997/1000,
        to_u128(pool_before.ynear), to_u128(pool_before.tokens)
    );
    assert_eq!(out, to_u128(after_swap_token), "Wrong amount of fee deducted");

    pool_before = view!(nearswap.pool_info(&dai())).unwrap_json::<PoolInfo>();
    let before_withdraw_token_lp1 = view!(
        nearswap.get_deposit_token(lp1.account_id.clone(), dai())
    ).unwrap_json::<U128>();
    let before_withdraw_near_lp1 = view!(
        nearswap.get_deposit_near(lp1.account_id.clone())
    ).unwrap_json::<U128>();
    let before_withdraw_token_lp2 = view!(
        nearswap.get_deposit_token(lp2.account_id.clone(), dai())
    ).unwrap_json::<U128>();
    let before_withdraw_near_lp2 = view!(
        nearswap.get_deposit_near(lp2.account_id.clone())
    ).unwrap_json::<U128>();

    // withdraw liquidity
    call!(
        lp1,
        nearswap.withdraw_liquidity(dai(), lp1_shares, U128(1), U128(1))
    ).assert_success();

    let pool_after = view!(nearswap.pool_info(&dai())).unwrap_json::<PoolInfo>();
    let after_withdraw_token_lp1 = view!(
        nearswap.get_deposit_token(lp1.account_id.clone(), dai())
    ).unwrap_json::<U128>();
    let after_swap_near_lp1 = view!(
        nearswap.get_deposit_near(lp1.account_id.clone())
    ).unwrap_json::<U128>();

    // Check If ~90% of total shares are received by lp1
    let tokens_received_lp1 = to_u128(after_withdraw_token_lp1) - to_u128(before_withdraw_token_lp1);
    assert_eq!(to_u128(pool_before.tokens)*9/10, tokens_received_lp1, "Redeemed liquidity is not correct for lp1");

    let near_received_lp1 = to_u128(after_swap_near_lp1) - to_u128(before_withdraw_near_lp1);
    assert_eq!(to_u128(pool_before.ynear)*9/10, near_received_lp1, "Redeemed Near incorrect - lp1");
    
    call!(
        lp2,
        nearswap.withdraw_liquidity(dai(), lp2_shares, U128(1), U128(1))
    ).assert_success();

    let after_withdraw_token_lp2 = view!(
        nearswap.get_deposit_token(lp2.account_id.clone(), dai())
    ).unwrap_json::<U128>();
    let after_swap_near_lp2 = view!(
        nearswap.get_deposit_near(lp2.account_id.clone())
    ).unwrap_json::<U128>();

    // Check If ~10% of total shares are received by lp2
    let tokens_received_lp2 = to_u128(after_withdraw_token_lp2) - to_u128(before_withdraw_token_lp2);
    assert_close(U128(to_u128(pool_before.tokens)*1/10), tokens_received_lp2, 1);

    let near_received_lp2 = to_u128(after_swap_near_lp2) - to_u128(before_withdraw_near_lp2);
    assert_eq!(to_u128(pool_before.ynear)*1/10, near_received_lp2, "Redeemed Near incorrect - lp2");

    // verify pool is empty after redeeming all liquidity
    let pool = view!(nearswap.pool_info(&dai())).unwrap_json::<PoolInfo>();
    assert!(to_u128(pool.ynear) == 0, "Near in pool incorrect");
    assert!(to_u128(pool.tokens) == 0, "Tokens in pool incorrect");
    assert!(to_u128(pool.total_shares) == 0, "Total shares in pool incorrect");
}

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct u256(4);
}

// Mock calculation of price without deducting fee
fn calc_out_expected(amount: u128, in_bal: u128, out_bal: u128) -> u128 {
    let X = u256::from(in_bal);
    let x = u256::from(amount);
    let numerator = ( x * u256::from(out_bal) * X);
    let mut denominator = (x + X);
    denominator *= denominator;
    return (numerator / denominator).as_u128();
}

fn to_u128(num: U128) -> u128 {
    return num.into();
}

fn to_yocto_str(x: &str) -> U128 {
    return U128(to_yocto(&x));
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