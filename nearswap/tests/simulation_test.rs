// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

#![allow(unused)]

mod test_utils;
use test_utils::*;

use near_sdk_sim::{
    call, deploy, init_simulator, near_crypto::Signer, to_yocto, view, ContractAccount,
    UserAccount, STORAGE_AMOUNT,
};
use near_sdk_sim::account::AccessKey;

use nearswap::util::*;
use nearswap::PoolInfo;
use nearswap::NearCLPContract;
use nep21_mintable::FungibleTokenContract;
//use near_primitives::errors::{ActionErrorKind, TxExecutionError};
use near_primitives::transaction::ExecutionStatus;
use near_primitives::types::{AccountId, Balance};
use near_sdk_sim::runtime::{init_runtime, RuntimeStandalone};
use near_sdk::json_types::{U128, U64};
use serde_json::json;
use std::convert::TryInto;

// utility, get pool info from CLP
//------------------------

#[test]
fn test_clp_add_liquidity_and_swap() {
    let (master_account, clp_contract, token, alice, carol) = deploy_clp();
    println!("NearSwap Contract Deployed");

    let token_contract = deploy_nep21(&token, U128(1_000_000 * NDENOM));
    println!("Token deployed");
    
    let near_deposit = 3_000 * NDENOM;
    let token_deposit = 30_000 * NDENOM;

    //---------------
    create_pool_add_liquidity(
        &clp_contract,
        &token_contract,
        &alice,
        &token,
        near_deposit,
        token_deposit,
    );
    // TOOD: let's add a bit more liquidity
    // println!("Alice increases the liquidity right first top up");

    // add_liquidity(
    //     &mut ctx.r,
    //     &ctx.clp,
    //     &ctx.alice,
    //     &ctx.nep21_1,
    //     3 * NDENOM,
    //     30 * NDENOM + 1,
    // );

    //---------------

    // get pool state before swap
    let pooli_before = get_pool_info(&clp_contract, &TOKEN_CONTRACT_ID.to_string());
    assert_eq!(
        pooli_before,
        PoolInfo {
            ynear: (near_deposit).into(),
            reserve: (token_deposit).into(),
            total_shares: (near_deposit).into()
        },
        "new pool balance should be from first deposit"
    );
    println!("pool_info:{}", pooli_before);

    println!("Sending nep21 to Carol");
    call!(
        token,
        token_contract.transfer(carol.account_id(), U128(to_yocto("19000"))),
        deposit = NEP21_STORAGE_DEPOSIT
    );

    let carol_t_balance_pre =
        show_nep21_bal(&token_contract, &carol.account_id());

    println!("carol swaps some near for tokens");
    let carol_deposit_yoctos: u128 = to_yocto("10");
    let min_token_expected: u128 = to_yocto("98"); //1-10 relation near/token
    let res = call!(
        carol,
        clp_contract.swap_near_to_token_exact_in(TOKEN_CONTRACT_ID.to_string(), U128(min_token_expected)),
        deposit = carol_deposit_yoctos.into()
    );
    println!("{:#?}\n Cost:\n{:#?}", res.status(), res.profile_data());
    let log = res.logs();

    assert!(res.is_ok());
    println!("pool_info:{}", get_pool_info(&clp_contract, &TOKEN_CONTRACT_ID.to_string()));
    println!("let's see how many token carol has after the swap");
    let carol_t_balance_post =
        show_nep21_bal(&token_contract, &carol.account_id());

    let carol_received = carol_t_balance_post - carol_t_balance_pre;
    assert!(
        carol_received >= min_token_expected,
        "carol should have received at least min_token_expected"
    );

    /*let pooli_after = get_pool_info(&clp_contract, &TOKEN_CONTRACT_ID.to_string());
    assert_eq!(
        pooli_after,
        PoolInfo {
            ynear: (u128::from(pooli_before.ynear) + carol_deposit_yoctos).into(),
            reserve: (u128::from(pooli_before.reserve) - carol_received).into(),
            total_shares: pooli_before.total_shares,
        },
        "new pool balance after swap"
    );

    //
    // bob creates another pool with nep21_2
    create_pool_add_liquidity(
        &mut ctx.r,
        &ctx.clp,
        &ctx.bob,
        &ctx.nep21_2,
        1000 * NDENOM,
        500 * NDENOM,
    );
    //---------------

    //
    // carol tries to swap nep1 she owns with nep2 from bob's pool

    println!(">> nep21-1 {:?}", pooli_after);
    println!(">> nep21-2 {:?}", get_pool_info(&ctx.r, &NEP21_ACC2));

    // liquidity1: 3000 NEAR -- 30_000 nep21_1  (1:10)  -- originally
    // liquidity1: 3010 NEAR -- 29_900 nep21_1  (1:~10)  -- after the swap
    // liquidity2: 1000 NEAR --    500 nep21_2  (2:1)

    // token1:token2 = 20 : 1
    // buying 3 nep21_2 requires 60 nep21_1 + fees

    let buy_amount = (3 * NDENOM).to_string();
    let carol_allowance = 61 * NDENOM;
    call(
        &mut ctx.r,
        &ctx.carol,
        &ctx.nep21_1,
        "inc_allowance",
        &json!({"escrow_account_id": CLP_ACC, "amount": carol_allowance.to_string()}),
        NEP21_STORAGE_DEPOSIT, //refundable, required if the nep21-contract needs more storage
    );

    let price: U128 = near_view(
        &ctx.r,
        &CLP_ACC.into(),
        "price_token_to_token_out",
        &json!({"from": &ctx.nep21_1.account_id,
                "to":  &ctx.nep21_2.account_id,
                "tokens_out": buy_amount,
        }),
    );
    println!(
        ">> price for {} {} is {} {}; allowance={}",
        buy_amount, &ctx.nep21_2.account_id, price.0, &ctx.nep21_1.account_id, carol_allowance
    );
    call(
        &mut ctx.r,
        &ctx.carol,
        &ctx.clp,
        "swap_tokens_exact_out",
        &json!({"from": &ctx.nep21_1.account_id,
                "to":  &ctx.nep21_2.account_id,
                "tokens_out": buy_amount,
                "max_tokens_in":  carol_allowance.to_string(),
        }),
        0,
    );

    //TDOO println!("carol removes allowance to CLP")

    println!("{} {:?}", &NEP21_ACC, get_pool_info(&ctx.r, &NEP21_ACC));
    println!("{} {:?}", &NEP21_ACC2, get_pool_info(&ctx.r, &NEP21_ACC2));*/
    //TODO check balances*/
}
