#![allow(unused)]

mod test_utils;
use crate::test_utils::*;
use near_clp::util::{MAX_GAS, NDENOM, NEP21_STORAGE_DEPOSIT};
use near_clp::PoolInfo;

//use near_primitives::errors::{ActionErrorKind, TxExecutionError};
use near_primitives::transaction::ExecutionStatus;
use near_runtime_standalone::RuntimeStandalone;
use near_sdk::json_types::{U128, U64};
use serde_json::json;

pub const CLP_ACC: &str = "nearclp";
pub const NEP21_ACC: &str = "fungible_token";
pub const ALICE_ACC: &str = "alice";
pub const BOB_ACC: &str = "bob";
pub const CAROL_ACC: &str = "carol";
pub const DAVE_ACC: &str = "dave";
pub const NEP21_ACC2: &str = "fun_token_2";

//#[test]
fn deploy_fungible_mint_for_alice() {
    let mut ctx = Ctx::new();
    let total_supply = 1_000_000 * NDENOM;

    let args = NewFungibleTokenArgs {
        owner_id: NEP21_ACC.into(),
        total_supply: U128(total_supply.clone()),
    };

    deploy_and_init_fungible_token(&mut ctx.r, &ctx.nep21_1, "new", U64(MAX_GAS), &args).unwrap();

    let returned_supply: U128 = near_view(&ctx.r, &NEP21_ACC.into(), "get_total_supply", "");
    assert_eq!(returned_supply.0, total_supply);
    println!("Note that we can use println! instead of env::log in simulation tests.");
    let demo_variable = "-- --nocapture".to_string();
    println!(
        "Just remember to to add this after 'cargo test': '{}'",
        demo_variable
    );

    let alice_balance: U128 = near_view(
        &ctx.r,
        &NEP21_ACC.into(),
        "get_balance",
        &json!({
            "owner_id": ALICE_ACC,
        }),
    );
    // Confirm Alice's initial balance is 0
    assert_eq!(alice_balance.0, 0);
    // send some to Alice
    let _execution_result = near_call(
        &mut ctx.r,
        &ctx.nep21_1,
        &ctx.nep21_1.account_id(),
        "transfer",
        &serde_json::to_vec(&json!({
            "new_owner_id": ALICE_ACC,
            "amount": "191919",
        }))
        .unwrap(),
        U64(MAX_GAS),
        36_500_000_000_000_000_000_000,
    )
    .unwrap();

    let alice_balance: U128 = near_view(
        &ctx.r,
        &NEP21_ACC.into(),
        "get_balance",
        &json!({
            "owner_id": ALICE_ACC,
        }),
    );
    // Confirm Alice's initial balance has increased to set amount
    assert_eq!(alice_balance.0, 191_919);
}

// utility, get pool info from CLP
fn get_pool_info(r: &RuntimeStandalone, token: &str) -> PoolInfo {
    return near_view(r, &CLP_ACC.into(), "pool_info", &json!({ "token": token }));
}

//helper fn
fn show_funtok_bal(r: &mut RuntimeStandalone, token: &ExternalUser, acc: &ExternalUser) -> u128 {
    println!("let's see how many tokens {} has now", acc.account_id());
    let fungt_balance: u128 = get_funtok_balance(r, token, &acc).into();
    println!("{} has {} {} tokens ", acc.account_id(), token.account_id(), fungt_balance);
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
        format!(r#"{{ "token":"{}" }}"#, token.account_id()),
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
        format!(
            r#"{{
            "new_owner_id": "{}",
            "amount": "{}"
        }}"#,
            owner.account_id(),
            token_amount
        ),
        NEP21_STORAGE_DEPOSIT, //refundable, required if the fun-contract needs more storage
    );

    println!("{} adds first liquidity", owner.account_id);

    println!("creating allowance for CLP");
    call(
        r,
        &owner,
        &token,
        "inc_allowance",
        format!(
            r#"{{
            "escrow_account_id": "{}",
            "amount": "{}"
        }}"#,
            CLP_ACC, token_amount
        ),
        NEP21_STORAGE_DEPOSIT, //refundable, required if the fun-contract needs more storage
    );

    show_funtok_bal(r, &token, &owner);

    //add_liquidity
    call(
        r,
        &owner,
        &clp,
        "add_liquidity",
        format!(
            r#"{{
                    "token": "{tok}",
                    "max_tokens": "{mta}",
                    "min_shares": "{msa}"
                }}"#,
            tok = token.account_id,
            mta = token_amount,
            msa = near_amount
        ),
        near_amount.into(), //send NEAR
    );

    let after_adding_info = get_pool_info(&r, &token.account_id());
    println!("pool after add liq: {} {:?}",&token.account_id(), after_adding_info);
}

#[test]
fn alice_adds_liquidity_carol_swaps() {
    let mut ctx = Ctx::new();

    let args = NewFungibleTokenArgs {
        owner_id: NEP21_ACC.into(),
        total_supply: U128(1_000_000 * NDENOM),
    };
    println!("deploy_and_init_fungible_token");
    deploy_and_init_fungible_token(&mut ctx.r, &ctx.nep21_1, "new", U64(MAX_GAS), &args).unwrap();

    let args2 = NewFungibleTokenArgs {
        owner_id: NEP21_ACC2.into(),
        total_supply: U128(10_000_000 * NDENOM),
    };
    println!("deploy_and_init_fungible_token 2");
    deploy_and_init_fungible_token(&mut ctx.r, &ctx.nep21_2, "new", U64(MAX_GAS), &args2).unwrap();

    println!("deploy_and_init_CLP");
    let args_clp = NewClpArgs {
        owner: ALICE_ACC.into(),
    };
    deploy_clp(&mut ctx.r, &ctx.clp, "new", U64(MAX_GAS), &args_clp).unwrap();

    let near_deposit = 3_000 * NDENOM;
    let token_deposit = 30_000 * NDENOM;

    //---------------
    create_pool_add_liquidity(&mut ctx.r, &ctx.clp, &ctx.alice, &ctx.nep21_1, near_deposit, token_deposit);
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
        format!(
            r#"{{
                "new_owner_id": "{}",
                "amount": "{}"
            }}"#,
            "carol",
            19_000 * NDENOM
        ),
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
        format!(
            r#"{{
                "token": "{tok}",
                "min_tokens": "{min_tok}"
                }}"#,
            tok = NEP21_ACC,
            min_tok = min_token_expected
        ),
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
    create_pool_add_liquidity(&mut ctx.r, &ctx.clp, &ctx.bob, &ctx.nep21_2, 
                1000*NDENOM, 500*NDENOM);
    //---------------


    //carol tries to swap nep1 she owns with nep2 from bob's pool

    //she gives allowance to CLP first
    let carol_tok_from_max_amount=15*NDENOM;

    println!("carol gives allowance to CLP");

    call(
        &mut ctx.r,
        &ctx.carol,
        &ctx.nep21_1,
        "inc_allowance",
        format!(
            r#"{{
            "escrow_account_id": "{}",
            "amount": "{}"
        }}"#,
            CLP_ACC, carol_tok_from_max_amount
        ),
        NEP21_STORAGE_DEPOSIT, //refundable, required if the nep21-contract needs more storage
    );

    println!("nep21-2 {:?}", get_pool_info(&ctx.r, &NEP21_ACC2));

    //-- swap_tokens_exact_out
    call(
        &mut ctx.r,
        &ctx.carol,
        &ctx.clp,
        "swap_tokens_exact_out",
        format!(
            r#"{{
                "from": "{from}",
                "to": "{to}",
                "to_tokens": "{tokto}",
                "max_from_tokens": "{max_tok_from}"
                }}"#,
            from = &ctx.nep21_1.account_id(),
            to = &ctx.nep21_2.account_id(),
            tokto = 200*NDENOM,
            max_tok_from = carol_tok_from_max_amount,
        ),
        0,
    );

    //TDOO println!("carol removes allowance to CLP")

    println!("{} {:?}",&NEP21_ACC, get_pool_info(&ctx.r, &NEP21_ACC) );
    println!("{} {:?}",&NEP21_ACC2, get_pool_info(&ctx.r, &NEP21_ACC2) );
    //panic!("show");
    //TODO check balances

}

struct Ctx {
    r: RuntimeStandalone,
    nep21_1: ExternalUser,
    nep21_2: ExternalUser,
    clp: ExternalUser,
    alice: ExternalUser,
    bob: ExternalUser,
    carol: ExternalUser,
    dave: ExternalUser,
}

impl Ctx {
    pub fn new() -> Self {
        let (mut r, main) = new_root("main.testnet".into());
        let mut create_u =
            |addr: &str, b: u128| main.create_external(&mut r, &addr.into(), ntoy(b)).unwrap();

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
}

//util fn
fn get_funtok_balance(r: &mut RuntimeStandalone, token:&ExternalUser, account: &ExternalUser) -> U128 {
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

/**utility fn schedule a call in the simulator, execute it, and all its receipts
 * report errors and lgos from all receipts
 *
 */
pub fn call(
    runtime: &mut RuntimeStandalone,
    sending_account: &ExternalUser,
    contract: &ExternalUser,
    method: &str,
    args: String,
    attached_amount: u128,
) {
    let gas = MAX_GAS;

    let tx = sending_account
        .new_tx(runtime, contract.account_id())
        .function_call(
            method.into(),
            args.as_bytes().to_vec(),
            gas.into(),
            attached_amount,
        )
        .sign(&sending_account.signer);

    let execution_outcome = runtime.resolve_tx(tx).unwrap(); //first TXN - unwraps to ExecutionOutcome
    runtime.process_all().unwrap(); //proces until there's no more generated receipts

    /* THE ABOVE CODE REPLACED THIS: near_call(runtime, //runtime
        sending_account, //sending account
        contract, //contract
        method,
        args.as_bytes(),
        U64(MAX_GAS),
        attached_amount
    )
    .unwrap();
    */

    println!("--------------------------------");
    println!("-- {}.{}() --", contract.account_id(), method);
    println!("execution_outcome.status {:?}", execution_outcome.status);
    println!("execution_outcome {:?}", execution_outcome);
    match execution_outcome.status {
        ExecutionStatus::Failure(msg) => panic!(msg),
        ExecutionStatus::SuccessValue(value) => {
            println!("execution_outcome.status => success {:?}", value)
        }
        ExecutionStatus::SuccessReceiptId(_) => {
            panic!("thre are pending receipts! call runtime.process_all() to complete all txns")
        }
        ExecutionStatus::Unknown => unreachable!(),
    }
    println!(
        "--------- RECEIPTS ({})",
        execution_outcome.receipt_ids.len()
    );
    let mut count_failed = 0;
    let mut inx = 0;
    for elem in execution_outcome.receipt_ids {
        let outcome2 = runtime.outcome(&elem);
        println!("---- Receipt {} outcome: {:?}", inx, outcome2);
        match outcome2 {
            Some(outcome2) => {
                println!("receipt {} logs: {:?}", inx, outcome2.logs);
                match outcome2.status {
                    ExecutionStatus::Failure(txresult) => {
                        println!("receipt {} failure: {:?}", inx, txresult);
                        count_failed+=1;
                    },
                    ExecutionStatus::SuccessValue(value) => println!("receipt {} success {:?}",inx,value),
                    ExecutionStatus::SuccessReceiptId(_) => panic!("there are pending receipts! call runtime.process_all() to complete all txns"),
                    ExecutionStatus::Unknown => unreachable!(),
                }
            }
            None => println!("None"),
        }
        inx += 1;
    }
    if count_failed > 0 {
        panic!(format!("{} RECEIPT(S) FAILED", count_failed));
    }
    println!("--------------------------------");
}
