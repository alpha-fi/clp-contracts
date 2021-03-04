// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

#![allow(unused)]

mod ctrtypes;
use crate::ctrtypes::*;

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

#[test]
fn test_nep21_transer() {
    println!(
        "Note that we can use println! instead of env::log in simulation tests. To debug add '-- --nocapture' after 'cargo test': "
    );
    let (master_account, contract, alice) = deploy_nep21(U128(1_000_000 * NDENOM));
    println!("tokens deployed");

    let transfer_amount = to_yocto("100");
    let initial_balance = to_yocto("0");
    // send some to Alice
    let res = call!(
        master_account,
        contract.transfer(alice.account_id(), transfer_amount.into()),
        deposit = STORAGE_AMOUNT
    );
    println!("{:#?}\n Cost:\n{:#?}", res.status(), res.profile_data());
    assert!(res.is_ok());

    let val = view!(contract.get_balance(alice.account_id()));
    let value: U128 = val.unwrap_json();
    assert_eq!(initial_balance + transfer_amount, value.0);
}

// utility, get pool info from CLP
/*
//------------------------

#[test]
fn test_clp_add_liquidity_and_swap() {
    let mut ctx = Ctx::new();

    ctx.deploy_tokens();
    ctx.deploy_clp();
    println!("tokens deployed");

    let near_deposit = 3_000 * NDENOM;
    let token_deposit = 30_000 * NDENOM;

    //---------------
    create_pool_add_liquidity(
        &mut ctx.r,
        &ctx.clp,
        &ctx.alice,
        &ctx.nep21_1,
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
    let pooli_before = get_pool_info(&ctx.r, &NEP21_ACC);
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
    call(
        &mut ctx.r,
        &ctx.nep21_1,
        &ctx.nep21_1,
        "transfer",
        &json!({"new_owner_id": "carol", "amount": (19_000 * NDENOM).to_string()}),
        NEP21_STORAGE_DEPOSIT, //refundable, required if the fun-contract needs more storage
    );

    let carol_t_balance_pre =
        show_nep21_bal(&mut ctx.r, &ctx.nep21_1.account_id, &ctx.carol.account_id);

    println!("carol swaps some near for tokens");
    let carol_deposit_yoctos: u128 = 10 * NDENOM;
    let min_token_expected: u128 = 98 * NDENOM; //1-10 relation near/token
    call(
        &mut ctx.r,
        &ctx.carol,
        &ctx.clp,
        "swap_near_to_token_exact_in",
        &json!({"token": NEP21_ACC,
                "min_tokens": min_token_expected.to_string()}),
        carol_deposit_yoctos.into(),
    );

    println!("let's see how many token carol has after the swap");
    let carol_t_balance_post =
        show_nep21_bal(&mut ctx.r, &ctx.nep21_1.account_id, &ctx.carol.account_id);
    let carol_received = carol_t_balance_post - carol_t_balance_pre;
    assert!(
        carol_received >= min_token_expected,
        "carol should have received at least min_token_expected"
    );

    let pooli_after = get_pool_info(&ctx.r, &NEP21_ACC);
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
    println!("{} {:?}", &NEP21_ACC2, get_pool_info(&ctx.r, &NEP21_ACC2));
    //TODO check balances
}

pub const CLP_ACC: &str = "nearclp";
pub const NEP21_ACC: &str = "fungible_token";
pub const ALICE_ACC: &str = "alice";
pub const BOB_ACC: &str = "bob";
pub const CAROL_ACC: &str = "carol";
pub const DAVE_ACC: &str = "dave";
pub const NEP21_ACC2: &str = "fun_token_2";

/// NEAR to yoctoNEAR
pub fn ntoy(near_amount: Balance) -> Balance {
    near_amount * NDENOM
}

/// Ctx encapsulates common variables for a test.
pub struct Ctx {
    pub r: RuntimeStandalone,
    pub nep21_1: ExternalUser,
    pub nep21_2: ExternalUser,
    pub clp: ExternalUser,
    pub alice: ExternalUser,
    pub bob: ExternalUser,
    pub carol: ExternalUser,
    pub dave: ExternalUser,
}
impl Ctx {
    pub fn new() -> Self {
        let signer_account: AccountId;
        let (mut r, signer, signer_account) = init_runtime(None);
        let signer_u = ExternalUser {
            account_id: signer_account,
            signer: signer,
        };
        let mut create_u = |addr: &str, b: u128| {
            signer_u
                .create_external(&mut r, &addr.into(), ntoy(b))
                .unwrap()
        };

        Self {
            nep21_1: create_u(&NEP21_ACC, 1_000_000),
            nep21_2: create_u(&NEP21_ACC2, 1_000_000),
            clp: create_u(&CLP_ACC, 1_000_000),
            alice: create_u(&ALICE_ACC, 1_000_000),
            bob: create_u(&BOB_ACC, 1_000_000),
            carol: create_u(&CAROL_ACC, 1_000_000),
            dave: create_u(&DAVE_ACC, 1_000_000),
            r: r,
        }
    }

    pub fn deploy_tokens(&mut self) {
        println!("deploy_nep21");
        let mut args = NewNEP21Args {
            owner_id: NEP21_ACC.into(),
            total_supply: U128(1_000_000 * NDENOM),
        };
        deploy_nep21(&mut self.r, &self.nep21_1, U64(MAX_GAS), &args).unwrap();

        println!("deploy_nep21 2");
        args.owner_id = NEP21_ACC2.into();
        deploy_nep21(&mut self.r, &self.nep21_2, U64(MAX_GAS), &args).unwrap();

        let supply: U128 = near_view(&self.r, &NEP21_ACC.into(), "get_total_supply", "");
        assert_eq!(supply.0, args.total_supply.into());
    }

    pub fn deploy_clp(&mut self) {
        println!("deploy_and_init_CLP");
        let args_clp = NewClpArgs {
            owner: ALICE_ACC.into(),
        };
        deploy_clp(&mut self.r, &self.clp, U64(MAX_GAS), &args_clp).unwrap();
    }
}

fn check_nep21_bal(ctx: &Ctx, token: &AccountId, who: &AccountId, expected: Balance) {
    let bal: U128 = near_view(
        &ctx.r,
        &token,
        "get_balance",
        &json!({
            "owner_id": who,
        }),
    );
    assert_eq!(bal.0, expected);
}*/