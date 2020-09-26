mod test_utils;
use crate::test_utils::*;
use near_clp::util::{MAX_GAS, NDENOM, NEP21_STORAGE_DEPOSIT};
use near_clp::PoolInfo;

//use near_primitives::errors::ActionErrorKind;
//use near_primitives::errors::TxExecutionError;
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

#[test]
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
fn get_pool_info(r: &RuntimeStandalone, funtok: &str) -> PoolInfo {
    return near_view(r, &CLP_ACC.into(), "pool_info", &json!({ "token": funtok }));
}

//helper fn
fn show_funtok_bal(r: &mut RuntimeStandalone, acc: &ExternalUser) -> u128 {
    println!("let's see how many tokens {} has now", acc.account_id());
    let funt_balance: u128 = get_funtok_balance(r, &acc).into();
    println!("{} fun tokens {}", acc.account_id(), funt_balance);
    return funt_balance;
}

#[test]
fn alice_adds_liquidity_carol_swaps() {
    // let (mut r, _, fungible_token, _fun_token2, clp, alice, _bob, carol, _dave) = basic_setup();
    let mut ctx = Ctx::new();
    let args = NewFungibleTokenArgs {
        owner_id: NEP21_ACC.into(),
        total_supply: U128(1_000_000 * NDENOM),
    };

    println!("deploy_and_init_fungible_token");
    deploy_and_init_fungible_token(&mut ctx.r, &ctx.nep21_1, "new", U64(MAX_GAS), &args).unwrap();

    // let args2 = NewFungibleTokenArgs {
    //     owner_id: NEP21_ACC2.into(),
    //     total_supply: U128(10_000_000),
    // };

    //println!("deploy_and_init_fungible_token 2");
    //deploy_and_init_fungible_token(&mut r, &fun_token2, "new", U64(MAX_GAS), &args2).unwrap();

    let args_clp = NewClpArgs {
        owner: ALICE_ACC.into(),
    };
    deploy_clp(&mut ctx.r, &ctx.clp, "new", U64(MAX_GAS), &args_clp).unwrap();

    println!("alice creates a pool");
    call(
        &mut ctx.r,
        &ctx.alice,
        &ctx.clp,
        "create_pool",
        format!(r#"{{ "token":"{}" }}"#, NEP21_ACC),
        0,
    );

    assert_eq!(
        get_pool_info(&ctx.r, &NEP21_ACC),
        PoolInfo {
            near_bal: 0,
            token_bal: 0,
            total_shares: 0
        },
        "new pool should be empty"
    );

    println!("Sending nep21_1 to alice");
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
            ALICE_ACC,
            202_020 * NDENOM
        ),
        NEP21_STORAGE_DEPOSIT, //refundable, required if the fun-contract needs more storage
    );

    println!("alice adds first liquidity");
    let near_deposit: u128 = ntoy(3_000);
    let token_deposit: u128 = ntoy(30_000); // 1/10 ratio

    println!("Alice creates allowance for CLP");
    call(
        &mut ctx.r,
        &ctx.alice,
        &ctx.nep21_1,
        "inc_allowance",
        format!(
            r#"{{
            "escrow_account_id": "{}",
            "amount": "{}"
        }}"#,
            CLP_ACC, token_deposit
        ),
        NEP21_STORAGE_DEPOSIT, //refundable, required if the fun-contract needs more storage
    );

    show_funtok_bal(&mut ctx.r, &ctx.alice);

    //add_liquidity
    call(
        &mut ctx.r,
        &ctx.alice,
        &ctx.clp,
        "add_liquidity",
        format!(
            r#"{{
                    "token": "{tok}",
                    "max_token_amount": {mta},
                    "min_shares_amount": {msa}
                }}"#,
            tok = NEP21_ACC,
            mta = token_deposit,
            msa = near_deposit
        ),
        near_deposit.into(),
    );

    //get pool state before swap
    let pool_info_pre_swap = get_pool_info(&ctx.r, &NEP21_ACC);
    assert_eq!(
        pool_info_pre_swap,
        PoolInfo {
            near_bal: near_deposit.into(),
            token_bal: token_deposit.into(),
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

    let carol_funt_balance_pre = show_funtok_bal(&mut ctx.r, &ctx.carol);

    println!("carol swaps some near for tokens");
    let carol_deposit_yoctos: u128 = 10 * NDENOM;
    let min_token_expected: u128 = 98 * NDENOM; //1-10 relation near/token

    call(
        &mut ctx.r,
        &ctx.carol,
        &ctx.clp,
        "swap_near_to_reserve_exact_in",
        format!(
            r#"{{
                "token": "{tok}",
                "min_tokens": {min_tok}
                }}"#,
            tok = NEP21_ACC,
            min_tok = min_token_expected
        ),
        carol_deposit_yoctos.into(),
    );

    println!("let's see how many token carol has after the swap");
    let carol_funt_balance_post = show_funtok_bal(&mut ctx.r, &ctx.carol);

    let carol_received = carol_funt_balance_post - carol_funt_balance_pre;

    assert!(
        carol_received >= min_token_expected,
        "carol should have received at least min_token_expected"
    );

    assert_eq!(
        get_pool_info(&ctx.r, &NEP21_ACC),
        PoolInfo {
            near_bal: (pool_info_pre_swap.near_bal + carol_deposit_yoctos).into(),
            token_bal: (pool_info_pre_swap.token_bal - carol_received).into(),
            total_shares: pool_info_pre_swap.total_shares,
        },
        "new pool balance after swap"
    );

    /*
    let mut alice_counter: u8 = near_view(
        &r,
        &NEP21_ACC2.into(),
        "get_num",
        &json!({
            "account": ALICE_ACC
        })
    );

    assert_eq!(alice_counter.clone(), 0);

    let mut execution_outcome = near_call(&mut r,
        &alice,
        &NEP21_ACC2,
        "increment",
        &[],
        U64(MAX_GAS),
        0
    ).unwrap();

    println!("Log(s) {:?}", execution_outcome.logs);

    // Make sure it was successful
    assert_eq!(execution_outcome.status, ExecutionStatus::SuccessValue(vec![]));

    alice_counter = near_view(
        &r,
        &NEP21_ACC2.into(),
        "get_num",
        &json!({
            "account": ALICE_ACC
        })
    );

    assert_eq!(alice_counter.clone(), 1);

    // Now we expect that when we increment again, the number will be two, which will move a fungible token
    // Before we can move over the fungible token, though, we need to

    // Check Alice's fungible token balance before, which should be zero.
    let mut alice_tokens: U128 = near_view(
        &r,
        &NEP21_ACC.into(),
        "get_balance",
        &json!({
            "owner_id": ALICE_ACC
        })
    );

    assert_eq!(alice_tokens.clone().0, 0);

    // Now increment again
    execution_outcome = near_call(&mut r,
        &alice,
        &CLP_ACC,
        "cross_contract_increment",
        &serde_json::to_vec(&json!({
            "counter_account": NEP21_ACC2,
            "token_account": NEP21_ACC,
        }),).unwrap(),
        U64(MAX_GAS),
        0
    ).unwrap();

    println!("Log(s) {:?}", execution_outcome.logs);
    // Make sure it was successful
    assert_eq!(execution_outcome.status, ExecutionStatus::SuccessValue(vec![]));

    // Check that the number has increased to 2
    alice_counter = near_view(
        &r,
        &NEP21_ACC2.into(),
        "get_num",
        &json!({
            "account": ALICE_ACC
        })
    );

    assert_eq!(alice_counter.clone(), 2);

    // Cross-contract call within a callback (see README for more details)
    // Check that the fungible token has been given to Alice since 2 is an even number
    // Note: this is a current limitation with simulation tests.
    // At this time you cannot send more cross-contract calls inside of a cross-contract callback
    // Intentionally commented out the final assertion that would reasonably succeed
    /*
    let alice_new_tokens: U128 = near_view(
        &r,
        &NEP21_ACC.into(),
        "get_balance",
        &json!({
            "owner_id": ALICE_ACC
        })
    );

    assert_eq!(alice_new_tokens.clone().0, 1);
    */

    // Below we will demonstrate handling an error and a limitation with capturing errors in simulation testing.
    // The following call will fail because we're trying to transfer fungible tokens from an account to itself.

    let will_error = near_call(&mut r,
        &simulation_example,
        &CLP_ACC,
        "send_token_if_counter_even",
        &serde_json::to_vec(&json!({
            "new_num": alice_counter.clone(),
            "token_account": NEP21_ACC,
            "recipient_account": CLP_ACC,
        }),).unwrap(),
        U64(MAX_GAS),
        0
    );
    if will_error.is_err() {
        let execution_status  = will_error.clone().unwrap_err().status;

        #[allow(unused_variables)]
        if let ExecutionStatus::Failure(TxExecutionError::ActionError(near_primitives::errors::ActionError { index, kind })) = execution_status {
            if let ActionErrorKind::FunctionCallError(near_vm_errors::FunctionCallError::HostError(near_vm_errors::HostError::GuestPanic { panic_msg })) = kind {
                assert_eq!(panic_msg, "(post_transfer) The promise failed. See receipt failures.".to_string());

                // Uncomment the below line if the ".then" is removed at the bottom of send_token_if_counter_even in src/lib.rs
                // assert!(panic_msg.contains("The new owner should be different from the current owner"));
            }
        }
    }

    // Error messages early in promise execution (see README for more details)
    // Note that above, the error we received is the error set up in src/lib.rs and not the error returned from the fungible token contract.
    // (At the time of this writing, the error message for an account attempting to transfer tokens to itself would be:
    // "The new owner should be different from the current owner"
    // This demonstrates a limitation in simulation testing at the moment. Please see the README for more information on practical debugging steps.

    // Now that we've finished demonstrating that limitation, we'll make the call with the correct

    // Confirm that the simulation account has zero fungible tokens
    let fungible_tokens: U128 = near_view(
        &r,
        &NEP21_ACC.into(),
        "get_balance",
        &json!({
            "owner_id": CLP_ACC
        })
    );

    assert_eq!(fungible_tokens.clone().0, 0);

    // Give 50 fungible tokens to simulation account

    near_call(&mut r,
              &fungible_token,
              &NEP21_ACC,
              "transfer",
              &serde_json::to_vec(&json!({
            "new_owner_id": CLP_ACC,
            "amount": "50",
        }),).unwrap(),
              U64(MAX_GAS),
              36_500_000_000_000_000_000_000
    ).unwrap();

    // Now transfer one of those fungible tokens to Alice

    let will_succeed = near_call(&mut r,
        &simulation_example,
        &CLP_ACC,
        "send_token_if_counter_even",
        &serde_json::to_vec(&json!({
            "new_num": alice_counter.clone(),
            "token_account": NEP21_ACC,
            "recipient_account": ALICE_ACC,
        }),).unwrap(),
        U64(MAX_GAS),
        0
    ).unwrap();

    println!("Log(s) {:?}", will_succeed.logs);
    // Make sure it was successful
    assert_eq!(will_succeed.status, ExecutionStatus::SuccessValue(vec![]));

    alice_tokens = near_view(
        &r,
        &NEP21_ACC.into(),
        "get_balance",
        &json!({
            "owner_id": ALICE_ACC
        })
    );

    assert_eq!(alice_tokens.clone().0, 1);

    let fungible_tokens: U128 = near_view(
        &r,
        &NEP21_ACC.into(),
        "get_balance",
        &json!({
            "owner_id": CLP_ACC
        })
    );

    assert_eq!(fungible_tokens.clone().0, 49);
    */
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
fn get_funtok_balance(r: &mut RuntimeStandalone, account: &ExternalUser) -> U128 {
    let result: U128 = near_view(
        &r,
        &NEP21_ACC.into(),
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
