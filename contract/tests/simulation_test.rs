mod utils;
use crate::utils::{
    ExternalUser,
    MAX_GAS,
};
use near_clp::PoolInfo;
//use near_primitives::errors::ActionErrorKind;
//use near_primitives::errors::TxExecutionError;
use near_primitives::{
    transaction::ExecutionStatus,
    types::{AccountId},
};
use near_runtime_standalone::RuntimeStandalone;
use near_sdk::json_types::{U128, U64};
use serde_json::json;
use utils::{
    deploy_and_init_fungible_token, deploy_clp, near_call, near_view, new_root, ntoy,
    NewClpArgs, NewFungibleTokenArgs,
};

pub const CLP_ACCOUNT_NAME: &str = "nearclp";
pub const FUNGIBLE_TOKEN_ACCOUNT_NAME: &str = "fungible_token";
pub const ALICE_ACCOUNT_NAME: &str = "alice";
pub const BOB_ACCOUNT_NAME: &str = "bob";
pub const CAROL_ACCOUNT_NAME: &str = "carol";
pub const DAVE_ACCOUNT_NAME: &str = "dave";
pub const FUN_TOKEN2_ACCOUNT_NAME: &str = "fun_token_2";

const NEP21_STORAGE_DEPOSIT: u128 = 10_000_000_000_000_000_000_000_000;

#[test]
fn deploy_fungible_mint_for_alice() {
    let (mut r, _, fungible_token, _, _, _, _, _, _) = basic_setup();
    let total_supply = 1_000_000;

    let args = NewFungibleTokenArgs {
        owner_id: FUNGIBLE_TOKEN_ACCOUNT_NAME.into(),
        total_supply: U128(total_supply.clone()),
    };

    deploy_and_init_fungible_token(&mut r, &fungible_token, "new", U64(MAX_GAS), &args).unwrap();

    let returned_supply: U128 = near_view(
        &r,
        &FUNGIBLE_TOKEN_ACCOUNT_NAME.into(),
        "get_total_supply",
        "",
    );
    assert_eq!(returned_supply.0, total_supply);
    println!("Note that we can use println! instead of env::log in simulation tests.");
    let demo_variable = "-- --nocapture".to_string();
    println!(
        "Just remember to to add this after 'cargo test': '{}'",
        demo_variable
    );

    let alice_balance: U128 = near_view(
        &r,
        &FUNGIBLE_TOKEN_ACCOUNT_NAME.into(),
        "get_balance",
        &json!({
            "owner_id": ALICE_ACCOUNT_NAME,
        }),
    );
    // Confirm Alice's initial balance is 0
    assert_eq!(alice_balance.0, 0);
    // send some to Alice
    let _execution_result = near_call(
        &mut r,
        &fungible_token,
        &fungible_token.account_id(),
        "transfer",
        &serde_json::to_vec(&json!({
            "new_owner_id": ALICE_ACCOUNT_NAME,
            "amount": "191919",
        }))
        .unwrap(),
        U64(MAX_GAS),
        36_500_000_000_000_000_000_000,
    )
    .unwrap();

    let alice_balance: U128 = near_view(
        &r,
        &FUNGIBLE_TOKEN_ACCOUNT_NAME.into(),
        "get_balance",
        &json!({
            "owner_id": ALICE_ACCOUNT_NAME,
        }),
    );
    // Confirm Alice's initial balance has increased to set amount
    assert_eq!(alice_balance.0, 191_919);
}

// utility, get pool info from CLP
fn get_pool_info(r: &RuntimeStandalone, funtok: &str) -> PoolInfo {
    return near_view(
        r,
        &CLP_ACCOUNT_NAME.into(),
        "pool_info",
        &json!({ "token": funtok }),
    );
}

fn showFunTokBal(r:&mut RuntimeStandalone, acc:&ExternalUser) -> u128 {
    println!("let's see how many tokens {} has now",acc.account_id());
    let funt_balance:u128 = get_funtok_balance(r, &acc).into();
    println!("{} fun tokens {}", acc.account_id(), funt_balance);
    return funt_balance;
}

#[test]
fn alice_is_a_lp() {
    let (mut r, _, fungible_token, _fun_token2, clp, alice, _bob, carol, _dave) = basic_setup();

    let args = NewFungibleTokenArgs {
        owner_id: FUNGIBLE_TOKEN_ACCOUNT_NAME.into(),
        total_supply: U128(1_000_000),
    };

    println!("deploy_and_init_fungible_token");
    deploy_and_init_fungible_token(&mut r, &fungible_token, "new", U64(MAX_GAS), &args).unwrap();

    // let args2 = NewFungibleTokenArgs {
    //     owner_id: FUN_TOKEN2_ACCOUNT_NAME.into(),
    //     total_supply: U128(10_000_000),
    // };

    //println!("deploy_and_init_fungible_token 2");
    //deploy_and_init_fungible_token(&mut r, &fun_token2, "new", U64(MAX_GAS), &args2).unwrap();

    let args_clp = NewClpArgs {
        owner: ALICE_ACCOUNT_NAME.into(),
    };
    println!("deploy_and_init_clp");
    deploy_clp(&mut r, &clp, "new", U64(MAX_GAS), &args_clp).unwrap();

    // alice creates a pool
    println!("about to create alice's pool");
    call(
        &mut r,
        &alice,
        &clp.account_id(),
        "create_pool",
        format!(r#"{{ "token":"{}" }}"#,FUNGIBLE_TOKEN_ACCOUNT_NAME),
        0,
    );

    assert_eq!(
        get_pool_info(&r, &FUNGIBLE_TOKEN_ACCOUNT_NAME),
        PoolInfo {
            near_bal: 0,
            token_bal: 0,
            total_shares: 0
        },
        "new pool should be empty"
    );


    // send som token to alice
    println!("send some funtok to alice");
    call(
        &mut r,
        &fungible_token,
        &fungible_token.account_id(),
        "transfer",
        format!(r#"{{
            "new_owner_id": "{}",
            "amount": "202020"
        }}"#, ALICE_ACCOUNT_NAME),
        NEP21_STORAGE_DEPOSIT //refundable, required if the fun-contract needs more storage
    );

    showFunTokBal(&mut r,&alice);

    println!("alice adds first liquidity");
    let near_deposit: u128 = ntoy(3_000);
    let token_deposit: u128 = ntoy(3_000_000); // 1/1000 ratio

    call(
        &mut r,
        &alice,
        &clp.account_id(),
        "add_liquidity",
        format!(
            r#"{{
                    "token": "{tok}",
                    "max_token_amount": {mta},
                    "min_shares_amont": {msa}
                }}"#,
            tok = FUNGIBLE_TOKEN_ACCOUNT_NAME,
            mta = token_deposit,
            msa = near_deposit
        ),
        near_deposit.into(),
    );

    let pool_info = get_pool_info(&r, &FUNGIBLE_TOKEN_ACCOUNT_NAME);
    assert_eq!(
        pool_info,
        PoolInfo {
            near_bal: near_deposit.into(),
            token_bal: token_deposit.into(),
            total_shares: near_deposit.into()
        },
        "new pool balance should be from first deposit"
    );

    println!("pool_info:{}",pool_info);
    let prev_pool_near_blance = pool_info.near_bal;

    // Check Carols's fungible token balance before
    println!("send some funtok to carol");
    call(
        &mut r,
        &fungible_token,
        &fungible_token.account_id(),
        "transfer",
        format!(r#"{{
            "new_owner_id": {},
            "amount": "191919",
        }}"#,"carol"),
        NEP21_STORAGE_DEPOSIT, //refundable, required if the fun-contract needs more storage
    );

    let carol_funt_balance_pre = showFunTokBal(&mut r, &carol);

    println!("carol swaps some near for tokens");
    let carol_deposit_yoctos: u128 = ntoy(10);
    let min_token_expected: u128 = ntoy(9900); 

    call(
        &mut r,
        &carol,
        &clp.account_id(),
        "swap_near_to_reserve_exact_in",
        format!(
            r#"{{
                "token": "{tok}",
                "min_tokens": {min_tok}
                }}"#,
            tok = FUNGIBLE_TOKEN_ACCOUNT_NAME,
            min_tok = min_token_expected
        ),
        carol_deposit_yoctos.into(),
    );


    println!("let's see how many token carol has after the swap");
    let carol_funt_balance_post = showFunTokBal(&mut r, &carol);

    let carol_received = carol_funt_balance_post - carol_funt_balance_pre;

    assert!(carol_received >= min_token_expected, "carol should have received at least min_token_expected");

    assert_eq!(
        get_pool_info(&r, &FUNGIBLE_TOKEN_ACCOUNT_NAME),
        PoolInfo {
            near_bal: (prev_pool_near_blance + carol_deposit_yoctos).into(),
            token_bal: (token_deposit - carol_received).into(),
            total_shares: (prev_pool_near_blance + carol_deposit_yoctos).into()
        },
        "new pool balance after swap"
    );

    /*
    let mut alice_counter: u8 = near_view(
        &r,
        &FUN_TOKEN2_ACCOUNT_NAME.into(),
        "get_num",
        &json!({
            "account": ALICE_ACCOUNT_NAME
        })
    );

    assert_eq!(alice_counter.clone(), 0);

    let mut execution_outcome = near_call(&mut r,
        &alice,
        &FUN_TOKEN2_ACCOUNT_NAME,
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
        &FUN_TOKEN2_ACCOUNT_NAME.into(),
        "get_num",
        &json!({
            "account": ALICE_ACCOUNT_NAME
        })
    );

    assert_eq!(alice_counter.clone(), 1);

    // Now we expect that when we increment again, the number will be two, which will move a fungible token
    // Before we can move over the fungible token, though, we need to

    // Check Alice's fungible token balance before, which should be zero.
    let mut alice_tokens: U128 = near_view(
        &r,
        &FUNGIBLE_TOKEN_ACCOUNT_NAME.into(),
        "get_balance",
        &json!({
            "owner_id": ALICE_ACCOUNT_NAME
        })
    );

    assert_eq!(alice_tokens.clone().0, 0);

    // Now increment again
    execution_outcome = near_call(&mut r,
        &alice,
        &CLP_ACCOUNT_NAME,
        "cross_contract_increment",
        &serde_json::to_vec(&json!({
            "counter_account": FUN_TOKEN2_ACCOUNT_NAME,
            "token_account": FUNGIBLE_TOKEN_ACCOUNT_NAME,
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
        &FUN_TOKEN2_ACCOUNT_NAME.into(),
        "get_num",
        &json!({
            "account": ALICE_ACCOUNT_NAME
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
        &FUNGIBLE_TOKEN_ACCOUNT_NAME.into(),
        "get_balance",
        &json!({
            "owner_id": ALICE_ACCOUNT_NAME
        })
    );

    assert_eq!(alice_new_tokens.clone().0, 1);
    */

    // Below we will demonstrate handling an error and a limitation with capturing errors in simulation testing.
    // The following call will fail because we're trying to transfer fungible tokens from an account to itself.

    let will_error = near_call(&mut r,
        &simulation_example,
        &CLP_ACCOUNT_NAME,
        "send_token_if_counter_even",
        &serde_json::to_vec(&json!({
            "new_num": alice_counter.clone(),
            "token_account": FUNGIBLE_TOKEN_ACCOUNT_NAME,
            "recipient_account": CLP_ACCOUNT_NAME,
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
        &FUNGIBLE_TOKEN_ACCOUNT_NAME.into(),
        "get_balance",
        &json!({
            "owner_id": CLP_ACCOUNT_NAME
        })
    );

    assert_eq!(fungible_tokens.clone().0, 0);

    // Give 50 fungible tokens to simulation account

    near_call(&mut r,
              &fungible_token,
              &FUNGIBLE_TOKEN_ACCOUNT_NAME,
              "transfer",
              &serde_json::to_vec(&json!({
            "new_owner_id": CLP_ACCOUNT_NAME,
            "amount": "50",
        }),).unwrap(),
              U64(MAX_GAS),
              36_500_000_000_000_000_000_000
    ).unwrap();

    // Now transfer one of those fungible tokens to Alice

    let will_succeed = near_call(&mut r,
        &simulation_example,
        &CLP_ACCOUNT_NAME,
        "send_token_if_counter_even",
        &serde_json::to_vec(&json!({
            "new_num": alice_counter.clone(),
            "token_account": FUNGIBLE_TOKEN_ACCOUNT_NAME,
            "recipient_account": ALICE_ACCOUNT_NAME,
        }),).unwrap(),
        U64(MAX_GAS),
        0
    ).unwrap();

    println!("Log(s) {:?}", will_succeed.logs);
    // Make sure it was successful
    assert_eq!(will_succeed.status, ExecutionStatus::SuccessValue(vec![]));

    alice_tokens = near_view(
        &r,
        &FUNGIBLE_TOKEN_ACCOUNT_NAME.into(),
        "get_balance",
        &json!({
            "owner_id": ALICE_ACCOUNT_NAME
        })
    );

    assert_eq!(alice_tokens.clone().0, 1);

    let fungible_tokens: U128 = near_view(
        &r,
        &FUNGIBLE_TOKEN_ACCOUNT_NAME.into(),
        "get_balance",
        &json!({
            "owner_id": CLP_ACCOUNT_NAME
        })
    );

    assert_eq!(fungible_tokens.clone().0, 49);
    */
}

fn basic_setup() -> (
    RuntimeStandalone,
    ExternalUser,
    ExternalUser,
    ExternalUser,
    ExternalUser,
    ExternalUser,
    ExternalUser,
    ExternalUser,
    ExternalUser,
) {
    let (mut r, main) = new_root("main.testnet".into());

    let fungible_token = main
        .create_external(&mut r, &FUNGIBLE_TOKEN_ACCOUNT_NAME.into(), ntoy(1_000_000))
        .unwrap();

    let funt2 = main
        .create_external(&mut r, &FUN_TOKEN2_ACCOUNT_NAME.into(), ntoy(1_000_000))
        .unwrap();

    let clp = main
        .create_external(&mut r, &CLP_ACCOUNT_NAME.into(), ntoy(1_000_000))
        .unwrap();

    let alice = main
        .create_external(&mut r, &ALICE_ACCOUNT_NAME.into(), ntoy(1_000_000))
        .unwrap();

    let bob = main
        .create_external(&mut r, &BOB_ACCOUNT_NAME.into(), ntoy(2_000_000))
        .unwrap();

    let carol = main
        .create_external(&mut r, &CAROL_ACCOUNT_NAME.into(), ntoy(5_000))
        .unwrap();

    let dave = main
        .create_external(&mut r, &DAVE_ACCOUNT_NAME.into(), ntoy(3_000))
        .unwrap();

    return (r, main, fungible_token, funt2, clp, alice, bob, carol, dave);
}

//util fn
fn get_funtok_balance(    
    r: &mut RuntimeStandalone,
    account: &ExternalUser
) -> U128 {

    let result: U128 = near_view(
        &r,
        &FUNGIBLE_TOKEN_ACCOUNT_NAME.into(),
        "get_balance",
        &json!({
            "owner_id": &account.account_id()
        })
    );

    return result;

}

pub fn call(    
    runtime: &mut RuntimeStandalone,
    sending_account: &ExternalUser,
    contract: &AccountId,
    method: &str,
    args: String,
    attached_amount: u128
) {

    let gas = MAX_GAS;

    let tx = sending_account
        .new_tx(runtime, contract)
        .function_call(method.into(), args.as_bytes().to_vec(), gas.into(), attached_amount)
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
    println!("-- {}.{}() --", contract,method);
    println!("execution_outcome.status {:?}", execution_outcome.status);
    println!("execution_outcome {:?}", execution_outcome);
    match execution_outcome.status {
        ExecutionStatus::Failure(msg) => panic!(msg),
        ExecutionStatus::SuccessValue(value) => println!("execution_outcome.status => success {:?}",value),
        ExecutionStatus::SuccessReceiptId(_) => panic!("thre are pending receipts! call runtime.process_all() to complete all txns"),
        ExecutionStatus::Unknown => unreachable!(),
    }
    println!("-- RECEIPTS ({}) --", execution_outcome.receipt_ids.len());
    let mut count_failed=0;
    for elem in execution_outcome.receipt_ids {
        let outcome2 = runtime.outcome(&elem);
        println!("receipt outcome: {:?}", outcome2); 
        match outcome2 { 
            Some(outcome2) =>{
                println!("receipt logs: {:?}", outcome2.logs);
                match outcome2.status {
                    ExecutionStatus::Failure(txresult) => {
                        println!("receipt failure: {:?}", txresult);
                        count_failed+=1;
                    },
                    ExecutionStatus::SuccessValue(value) => println!("receipt success {:?}",value),
                    ExecutionStatus::SuccessReceiptId(_) => panic!("there are pending receipts! call runtime.process_all() to complete all txns"),
                    ExecutionStatus::Unknown => unreachable!(),
                }
            },
            None =>println!("None")
        }
    }
    if count_failed>0 {
        panic!(format!("{} RECEIPT(S) FAILED",count_failed));
    }
    println!("--------------------------------");
}

