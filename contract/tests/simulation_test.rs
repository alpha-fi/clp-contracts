mod utils;
use crate::utils::{
    ExternalUser, ALICE_ACCOUNT_ID, CLP_ACCOUNT_ID, FUNGIBLE_TOKEN_ACCOUNT_ID,
    FUN_TOKEN2_ACCOUNT_ID, MAX_GAS,
};
use near_clp::PoolInfo;
use near_primitives::errors::ActionErrorKind;
use near_primitives::errors::TxExecutionError;
use near_primitives::transaction::ExecutionStatus;
use near_runtime_standalone::RuntimeStandalone;
use near_sdk::json_types::{U128, U64};
use serde_json::json;
use utils::{
    deploy_and_init_fungible_token, deploy_clp, near_call, near_view, new_root, ntoy,
    NewClpArgs, NewFungibleTokenArgs,
};

const CANT_FAIL_GAS: u64   = 300_000_000_000_000;

//util fn
fn get_funtok_balance(    
    r: &mut RuntimeStandalone,
    account: &ExternalUser
) -> U128 {

    let result: U128 = near_view(
        &r,
        &FUNGIBLE_TOKEN_ACCOUNT_ID.into(),
        "get_balance",
        &json!({
            "owner_id": &account.account_id()
        })
    );

    return result;

}


#[test]
fn deploy_fungible_mint_for_alice() {
    let (mut r, _, fungible_token, _, _, _, _, _, _) = basic_setup();
    let total_supply = 1_000_000;

    let args = NewFungibleTokenArgs {
        owner_id: FUNGIBLE_TOKEN_ACCOUNT_ID.into(),
        total_supply: U128(total_supply.clone()),
    };

    deploy_and_init_fungible_token(&mut r, &fungible_token, "new", U64(MAX_GAS), &args).unwrap();

    let returned_supply: U128 = near_view(
        &r,
        &FUNGIBLE_TOKEN_ACCOUNT_ID.into(),
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
        &FUNGIBLE_TOKEN_ACCOUNT_ID.into(),
        "get_balance",
        &json!({
            "owner_id": ALICE_ACCOUNT_ID,
        }),
    );
    // Confirm Alice's initial balance is 0
    assert_eq!(alice_balance.0, 0);
    // send some to Alice
    near_call(
        &mut r,
        &fungible_token,
        &FUNGIBLE_TOKEN_ACCOUNT_ID,
        "transfer",
        &serde_json::to_vec(&json!({
            "new_owner_id": ALICE_ACCOUNT_ID,
            "amount": "191919",
        }))
        .unwrap(),
        U64(MAX_GAS),
        36_500_000_000_000_000_000_000,
    )
    .unwrap();

    let alice_balance: U128 = near_view(
        &r,
        &FUNGIBLE_TOKEN_ACCOUNT_ID.into(),
        "get_balance",
        &json!({
            "owner_id": ALICE_ACCOUNT_ID,
        }),
    );
    // Confirm Alice's initial balance has increased to set amount
    assert_eq!(alice_balance.0, 191_919);
}

// utility, get pool info from CLP
fn get_pool_info(r: &RuntimeStandalone, funtok: &str) -> PoolInfo {
    return near_view(
        r,
        &CLP_ACCOUNT_ID.into(),
        "pool_info",
        &json!({ "token": funtok }),
    );
}

#[test]
fn alice_is_a_lp() {
    let (mut r, _, fungible_token, fun_token2, clp, alice, _bob, carol, _dave) = basic_setup();

    let args = NewFungibleTokenArgs {
        owner_id: FUNGIBLE_TOKEN_ACCOUNT_ID.into(),
        total_supply: U128(1_000_000),
    };

    println!("deploy_and_init_fungible_token");
    deploy_and_init_fungible_token(&mut r, &fungible_token, "new", U64(MAX_GAS), &args).unwrap();

    let args2 = NewFungibleTokenArgs {
        owner_id: FUN_TOKEN2_ACCOUNT_ID.into(),
        total_supply: U128(10_000_000),
    };

    println!("deploy_and_init_fungible_token 2");
    deploy_and_init_fungible_token(&mut r, &fun_token2, "new", U64(MAX_GAS), &args2).unwrap();

    let args_clp = NewClpArgs {
        owner: ALICE_ACCOUNT_ID.into(),
    };
    println!("deploy_and_init_clp");
    deploy_clp(&mut r, &clp, "new", U64(MAX_GAS), &args_clp).unwrap();

    // alice creates a pool
    println!("about to create alice's pool");
    near_call(
        &mut r,
        &alice,
        &CLP_ACCOUNT_ID,
        "create_pool",
        &serde_json::to_vec(&json!({ "token": FUNGIBLE_TOKEN_ACCOUNT_ID })).unwrap(),
        U64(MAX_GAS),
        0,
    )
    .unwrap();

    assert_eq!(
        get_pool_info(&r, &FUNGIBLE_TOKEN_ACCOUNT_ID),
        PoolInfo {
            near_bal: 0,
            token_bal: 0,
            total_shares: 0
        },
        "new pool should be empty"
    );

    println!("alice adds first liquidity");

    let near_deposit: u128 = ntoy(3_000);
    let token_deposit: u128 = ntoy(3_000_000); // 1/1000 ratio

    let execution_outcome = near_call(
        &mut r,
        &alice,
        &CLP_ACCOUNT_ID,
        "add_liquidity",
        format!(
            r#"{{
                    "token": "{tok}",
                    "max_token_amount": {mta},
                    "min_shares_amont": {msa}
                }}"#,
            tok = FUNGIBLE_TOKEN_ACCOUNT_ID,
            mta = token_deposit,
            msa = near_deposit
        )
        .as_bytes(),
        U64(MAX_GAS),
        near_deposit.into(),
    )
    .unwrap();

    println!("Log(s) {:?}", execution_outcome.logs);

    let pool_info = get_pool_info(&r, &FUNGIBLE_TOKEN_ACCOUNT_ID);
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
    near_call(
        &mut r,
        &fungible_token,
        &FUNGIBLE_TOKEN_ACCOUNT_ID,
        "transfer",
        &serde_json::to_vec(&json!({
            "new_owner_id": "carol",
            "amount": "191919",
        }))
        .unwrap(),
        U64(MAX_GAS),
        36_500_000_000_000_000_000_000,
    )
    .unwrap();

    println!("let's see how many tokens carol has now");
    let carol_funt_balance_pre:u128 = get_funtok_balance(&mut r, &carol).into();
    println!("Carol fun tokens {}", carol_funt_balance_pre);

    
    println!("carol swaps some near for tokens");
    let carol_deposit_yoctos: u128 = ntoy(10);
    let min_token_expected: u128 = ntoy(9900); 

    let execution_outcome = near_call(
        &mut r,
        &carol,
        &CLP_ACCOUNT_ID,
        "swap_near_to_reserve_exact_in",
        format!(
            r#"{{
                "token": "{tok}",
                "min_tokens": {min_tok}
                }}"#,
            tok = FUNGIBLE_TOKEN_ACCOUNT_ID,
            min_tok = min_token_expected
        )
        .as_bytes(),
        U64(MAX_GAS),
        carol_deposit_yoctos.into(),
    )
    .unwrap();
    println!("Log(s) {:?}", execution_outcome.logs);
    println!("receipts(s) {:?}", execution_outcome.receipt_ids);
    println!("Status(s) {:?}", execution_outcome.status);

    println!("let's see how many token carol has after the swap");
    let carol_funt_balance_post:u128 = get_funtok_balance(&mut r, &carol).into();
    println!("Carol tokens after {}", carol_funt_balance_post);

    let carol_received = carol_funt_balance_post - carol_funt_balance_pre;

    assert!(carol_received >= min_token_expected, "carol should have received at least min_token_expected");

    assert_eq!(
        get_pool_info(&r, &FUNGIBLE_TOKEN_ACCOUNT_ID),
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
        &FUN_TOKEN2_ACCOUNT_ID.into(),
        "get_num",
        &json!({
            "account": ALICE_ACCOUNT_ID
        })
    );

    assert_eq!(alice_counter.clone(), 0);

    let mut execution_outcome = near_call(&mut r,
        &alice,
        &FUN_TOKEN2_ACCOUNT_ID,
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
        &FUN_TOKEN2_ACCOUNT_ID.into(),
        "get_num",
        &json!({
            "account": ALICE_ACCOUNT_ID
        })
    );

    assert_eq!(alice_counter.clone(), 1);

    // Now we expect that when we increment again, the number will be two, which will move a fungible token
    // Before we can move over the fungible token, though, we need to

    // Check Alice's fungible token balance before, which should be zero.
    let mut alice_tokens: U128 = near_view(
        &r,
        &FUNGIBLE_TOKEN_ACCOUNT_ID.into(),
        "get_balance",
        &json!({
            "owner_id": ALICE_ACCOUNT_ID
        })
    );

    assert_eq!(alice_tokens.clone().0, 0);

    // Now increment again
    execution_outcome = near_call(&mut r,
        &alice,
        &CLP_ACCOUNT_ID,
        "cross_contract_increment",
        &serde_json::to_vec(&json!({
            "counter_account": FUN_TOKEN2_ACCOUNT_ID,
            "token_account": FUNGIBLE_TOKEN_ACCOUNT_ID,
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
        &FUN_TOKEN2_ACCOUNT_ID.into(),
        "get_num",
        &json!({
            "account": ALICE_ACCOUNT_ID
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
        &FUNGIBLE_TOKEN_ACCOUNT_ID.into(),
        "get_balance",
        &json!({
            "owner_id": ALICE_ACCOUNT_ID
        })
    );

    assert_eq!(alice_new_tokens.clone().0, 1);
    */

    // Below we will demonstrate handling an error and a limitation with capturing errors in simulation testing.
    // The following call will fail because we're trying to transfer fungible tokens from an account to itself.

    let will_error = near_call(&mut r,
        &simulation_example,
        &CLP_ACCOUNT_ID,
        "send_token_if_counter_even",
        &serde_json::to_vec(&json!({
            "new_num": alice_counter.clone(),
            "token_account": FUNGIBLE_TOKEN_ACCOUNT_ID,
            "recipient_account": CLP_ACCOUNT_ID,
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
        &FUNGIBLE_TOKEN_ACCOUNT_ID.into(),
        "get_balance",
        &json!({
            "owner_id": CLP_ACCOUNT_ID
        })
    );

    assert_eq!(fungible_tokens.clone().0, 0);

    // Give 50 fungible tokens to simulation account

    near_call(&mut r,
              &fungible_token,
              &FUNGIBLE_TOKEN_ACCOUNT_ID,
              "transfer",
              &serde_json::to_vec(&json!({
            "new_owner_id": CLP_ACCOUNT_ID,
            "amount": "50",
        }),).unwrap(),
              U64(MAX_GAS),
              36_500_000_000_000_000_000_000
    ).unwrap();

    // Now transfer one of those fungible tokens to Alice

    let will_succeed = near_call(&mut r,
        &simulation_example,
        &CLP_ACCOUNT_ID,
        "send_token_if_counter_even",
        &serde_json::to_vec(&json!({
            "new_num": alice_counter.clone(),
            "token_account": FUNGIBLE_TOKEN_ACCOUNT_ID,
            "recipient_account": ALICE_ACCOUNT_ID,
        }),).unwrap(),
        U64(MAX_GAS),
        0
    ).unwrap();

    println!("Log(s) {:?}", will_succeed.logs);
    // Make sure it was successful
    assert_eq!(will_succeed.status, ExecutionStatus::SuccessValue(vec![]));

    alice_tokens = near_view(
        &r,
        &FUNGIBLE_TOKEN_ACCOUNT_ID.into(),
        "get_balance",
        &json!({
            "owner_id": ALICE_ACCOUNT_ID
        })
    );

    assert_eq!(alice_tokens.clone().0, 1);

    let fungible_tokens: U128 = near_view(
        &r,
        &FUNGIBLE_TOKEN_ACCOUNT_ID.into(),
        "get_balance",
        &json!({
            "owner_id": CLP_ACCOUNT_ID
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
        .create_external(&mut r, FUNGIBLE_TOKEN_ACCOUNT_ID.into(), ntoy(1_000_000))
        .unwrap();

    let funt2 = main
        .create_external(&mut r, FUN_TOKEN2_ACCOUNT_ID.into(), ntoy(1_000_000))
        .unwrap();

    let clp = main
        .create_external(&mut r, CLP_ACCOUNT_ID.into(), ntoy(1_000_000))
        .unwrap();

    let alice = main
        .create_external(&mut r, ALICE_ACCOUNT_ID.into(), ntoy(1_000_000))
        .unwrap();

    let bob = main
        .create_external(&mut r, "bob".into(), ntoy(2_000_000))
        .unwrap();

    let carol = main
        .create_external(&mut r, "carol".into(), ntoy(5_000))
        .unwrap();

    let dave = main
        .create_external(&mut r, "dave".into(), ntoy(3_000))
        .unwrap();

    return (r, main, fungible_token, funt2, clp, alice, bob, carol, dave);
}
