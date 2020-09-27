#![allow(unused)]

mod ctrtypes;
use crate::ctrtypes::*;

mod test_utils;
use crate::test_utils::*;

use near_clp::util::*;
use near_clp::PoolInfo;

//use near_primitives::errors::{ActionErrorKind, TxExecutionError};
use near_primitives::transaction::ExecutionStatus;
use near_primitives::types::{AccountId, Balance};
use near_runtime_standalone::{init_runtime_and_signer, RuntimeStandalone};
use near_sdk::json_types::{U128, U64};
use serde_json::json;

//#[test]
fn test_nep21_transer() {
    let mut ctx = Ctx::new();
    println!(
        "Note that we can use println! instead of env::log in simulation tests. To debug add '-- --nocapture' after 'cargo test': "
    );

    ctx.deploy_tokens();
    ctx.deploy_clp();
    println!("tokens deployed");

    check_nep21_bal(&ctx, &NEP21_ACC.into(), &ALICE_ACC.into(), 0);

    // send some to Alice
    let _execution_result = near_call(
        &mut ctx.r,
        &ctx.nep21_1,
        &ctx.nep21_1.account_id(),
        "transfer",
        &json!({
            "new_owner_id": ALICE_ACC,
            "amount": "191919",
        }),
        U64(MAX_GAS),
        36_500_000_000_000_000_000_000,
    )
    .unwrap();

    check_nep21_bal(&ctx, &NEP21_ACC.into(), &ALICE_ACC.into(), 191_919);
}

// utility, get pool info from CLP
fn get_pool_info(r: &RuntimeStandalone, token: &str) -> PoolInfo {
    return near_view(r, &CLP_ACC.into(), "pool_info", &json!({ "token": token }));
}

//helper fn
fn show_funtok_bal(r: &mut RuntimeStandalone, token: &ExternalUser, acc: &ExternalUser) -> u128 {
    println!("let's see how many tokens {} has now", acc.account_id());
    let fungt_balance: u128 = get_funtok_balance(r, token, &acc).into();
    println!(
        "{} has {} {} tokens ",
        acc.account_id(),
        token.account_id(),
        fungt_balance
    );
    return fungt_balance;
}

//-------------------------
fn create_pool_add_liquidity(
    r: &mut RuntimeStandalone,
    clp: &ExternalUser,
    owner: &ExternalUser,
    token: &ExternalUser,
    near_amount: u128,
    token_amount: u128,
) {
    println!("{} creates a pool", owner.account_id());

    call(
        r,
        &owner,
        &clp,
        "create_pool",
        &json!({"token": token.account_id()}),
        0,
    );

    assert_eq!(
        get_pool_info(&r, &token.account_id()),
        PoolInfo {
            ynear: 0,
            reserve: 0,
            total_shares: 0
        },
        "new pool should be empty"
    );

    println!("Making sure owner has token before adding liq");
    call(
        r,
        &token,
        &token,
        "transfer",
        &json!({"new_owner_id": owner.account_id(), "amount": token_amount.to_string()}),
        NEP21_STORAGE_DEPOSIT, //refundable, required if the fun-contract needs more storage
    );

    println!("{} adds first liquidity", owner.account_id);

    println!("creating allowance for CLP");
    call(
        r,
        &owner,
        &token,
        "inc_allowance",
        &json!({"escrow_account_id": CLP_ACC, "amount": token_amount.to_string()}),
        NEP21_STORAGE_DEPOSIT, //refundable, required if the fun-contract needs more storage
    );

    show_funtok_bal(r, &token, &owner);

    //add_liquidity
    call(
        r,
        &owner,
        &clp,
        "add_liquidity",
        &json!({"token": token.account_id,
                "max_tokens": token_amount.to_string(),
                "min_shares": near_amount.to_string()}),
        near_amount.into(), //send NEAR
    );

    let after_adding_info = get_pool_info(&r, &token.account_id());
    println!(
        "pool after add liq: {} {:?}",
        &token.account_id(),
        after_adding_info
    );
}

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
    //---------------

    //get pool state before swap
    let pool_info_pre_swap = get_pool_info(&ctx.r, &NEP21_ACC);
    assert_eq!(
        pool_info_pre_swap,
        PoolInfo {
            ynear: near_deposit.into(),
            reserve: token_deposit.into(),
            total_shares: near_deposit.into()
        },
        "new pool balance should be from first deposit"
    );
    println!("pool_info:{}", pool_info_pre_swap);

    println!("Sending nep21 to Carol");
    call(
        &mut ctx.r,
        &ctx.nep21_1,
        &ctx.nep21_1,
        "transfer",
        &json!({"new_owner_id": "carol", "amount": (19_000 * NDENOM).to_string()}),
        NEP21_STORAGE_DEPOSIT, //refundable, required if the fun-contract needs more storage
    );

    let carol_funt_balance_pre = show_funtok_bal(&mut ctx.r, &ctx.nep21_1, &ctx.carol);

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
    let carol_funt_balance_post = show_funtok_bal(&mut ctx.r, &ctx.nep21_1, &ctx.carol);
    let carol_received = carol_funt_balance_post - carol_funt_balance_pre;
    assert!(
        carol_received >= min_token_expected,
        "carol should have received at least min_token_expected"
    );

    assert_eq!(
        get_pool_info(&ctx.r, &NEP21_ACC),
        PoolInfo {
            ynear: (pool_info_pre_swap.ynear + carol_deposit_yoctos).into(),
            reserve: (pool_info_pre_swap.reserve - carol_received).into(),
            total_shares: pool_info_pre_swap.total_shares,
        },
        "new pool balance after swap"
    );

    println!();
    println!("-----------------------------");
    //bob creates another pool with nep21_2
    //---------------
    create_pool_add_liquidity(
        &mut ctx.r,
        &ctx.clp,
        &ctx.bob,
        &ctx.nep21_2,
        1000 * NDENOM,
        500 * NDENOM,
    );
    //---------------

    //carol tries to swap nep1 she owns with nep2 from bob's pool

    //she gives allowance to CLP first
    let carol_allowance = 15 * NDENOM;

    println!("carol gives allowance to CLP");

    call(
        &mut ctx.r,
        &ctx.carol,
        &ctx.nep21_1,
        "inc_allowance",
        &json!({"escrow_account_id": CLP_ACC, "amount": carol_allowance.to_string()}),
        NEP21_STORAGE_DEPOSIT, //refundable, required if the nep21-contract needs more storage
    );

    println!("nep21-2 {:?}", get_pool_info(&ctx.r, &NEP21_ACC2));

    //-- swap_tokens_exact_out
    call(
        &mut ctx.r,
        &ctx.carol,
        &ctx.clp,
        "swap_tokens_exact_out",
        &json!({"from": ctx.nep21_1.account_id,
                "to":  &ctx.nep21_2.account_id,
                "to_tokens":  (200 * NDENOM).to_string(),
                "max_from_tokens":  carol_allowance.to_string(),
        }),
        0,
    );

    //TDOO println!("carol removes allowance to CLP")

    println!("{} {:?}", &NEP21_ACC, get_pool_info(&ctx.r, &NEP21_ACC));
    println!("{} {:?}", &NEP21_ACC2, get_pool_info(&ctx.r, &NEP21_ACC2));
    //panic!("show");
    //TODO check balances
}

//util fn
fn get_funtok_balance(
    r: &mut RuntimeStandalone,
    token: &ExternalUser,
    account: &ExternalUser,
) -> U128 {
    let result: U128 = near_view(
        &r,
        &token.account_id(),
        "get_balance",
        &json!({
            "owner_id": &account.account_id()
        }),
    );

    return result;
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
        let signer_account: AccountId = "main.testnet".to_string();
        let (mut r, signer) = init_runtime_and_signer(&signer_account);
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

lazy_static::lazy_static! {
    static ref CLP_WASM_BYTES: &'static [u8] = include_bytes!("../target/wasm32-unknown-unknown/release/near_clp.wasm").as_ref();
    static ref FUNGIBLE_TOKEN_BYTES: &'static [u8] = include_bytes!("../../neardev/nep-21/target/wasm32-unknown-unknown/release/nep21_basic.wasm").as_ref();
}

pub fn deploy_nep21(
    runtime: &mut RuntimeStandalone,
    signer: &ExternalUser,
    gas: U64,
    args: &NewNEP21Args,
) -> TxResult {
    let tx = signer
        .new_tx(runtime, &signer.account_id)
        // transfer tokens otherwise "wouldn't have enough balance to cover storage"
        .transfer(ntoy(50))
        .deploy_contract(FUNGIBLE_TOKEN_BYTES.to_vec())
        .function_call(
            "new".to_string(),
            serde_json::to_vec(args).unwrap(),
            gas.into(),
            0,
        )
        .sign(&signer.signer);
    let res = runtime.resolve_tx(tx).unwrap();
    runtime.process_all().unwrap();
    outcome_into_result(res)
}

pub fn deploy_clp(
    runtime: &mut RuntimeStandalone,
    signer: &ExternalUser,
    gas: U64,
    args: &NewClpArgs,
) -> TxResult {
    let tx = signer
        .new_tx(runtime, &signer.account_id)
        .transfer(ntoy(50))
        .deploy_contract(CLP_WASM_BYTES.to_vec())
        .function_call(
            "new".to_string(),
            serde_json::to_vec(args).unwrap(),
            gas.into(),
            0,
        )
        .sign(&signer.signer);
    let res = runtime.resolve_tx(tx).unwrap();
    runtime.process_all().unwrap();
    outcome_into_result(res)
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
}
