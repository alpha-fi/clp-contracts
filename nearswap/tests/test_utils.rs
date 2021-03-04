// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

#![allow(unused)]

use near_sdk_sim::{
    call, deploy, init_simulator, near_crypto::Signer, to_yocto, view, ContractAccount,
    UserAccount, STORAGE_AMOUNT,
};
use near_sdk_sim::account::AccessKey;

use nearswap::util::*;
use nearswap::PoolInfo;
use nearswap::NearCLPContract;
use nep21_mintable::FungibleTokenContract;
use near_primitives::transaction::ExecutionStatus;
use near_primitives::types::{AccountId, Balance};
use near_sdk_sim::runtime::{init_runtime, RuntimeStandalone};
use near_sdk::json_types::{U128, U64};
use serde_json::json;
use std::convert::TryInto;

pub const TOKEN_CONTRACT_ID: &str = "token";
pub const NEARSWAP_CONTRACT_ID: &str = "nearswap";

/// Load in contract bytes
near_sdk_sim::lazy_static! {
    static ref CLP_WASM_BYTES: &'static [u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/nearswap.wasm").as_ref();
    static ref FUNGIBLE_TOKEN_BYTES: &'static [u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/nep21_mintable.wasm").as_ref();
}

// Deploy NearCLP Contract
pub fn deploy_clp() -> (
    UserAccount, ContractAccount<NearCLPContract>, UserAccount, UserAccount, UserAccount
) {
    let master_account = init_simulator(None);
    println!("deploy_and_init_CLP");
    // uses default values for deposit and gas
    let contract_user = deploy!(
        // Contract Proxy
        contract: NearCLPContract,
        // Contract account id
        contract_id: NEARSWAP_CONTRACT_ID,
        // Bytes of contract
        bytes: &CLP_WASM_BYTES,
        // User deploying the contract,
        signer_account: master_account,
        // init method
        init_method: new(master_account.account_id().try_into().unwrap())
    );
    let token = master_account.create_user("nep_21_token".to_string(), to_yocto("1000000"));
    let alice = master_account.create_user("alice".to_string(), to_yocto("1000000"));
    let carol = master_account.create_user("carol".to_string(), to_yocto("1000000"));
    (master_account, contract_user, token, alice, carol)
}

// Deploy NEP-21 Contract
pub fn deploy_nep21(
    master_account: &UserAccount, Contract_Id: AccountId, total_supply: U128
) -> ContractAccount<FungibleTokenContract> {
    println!("deploy_nep21");
    // uses default values for deposit and gas
    let contract_user = deploy!(
        // Contract Proxy
        contract: FungibleTokenContract,
        // Contract account id
        contract_id: Contract_Id,
        // Bytes of contract
        bytes: &FUNGIBLE_TOKEN_BYTES,
        // User deploying the contract,
        signer_account: master_account,
        // init method
        init_method: new(master_account.account_id(), total_supply, 24)
    );
    contract_user
}

pub fn get_pool_info(clp: &ContractAccount<NearCLPContract>, token: &AccountId) -> PoolInfo {
    let val = view!(clp.pool_info(token));
    let value: PoolInfo = val.unwrap_json();
    return value;
}

pub fn get_nep21_balance(
    token_contract: &ContractAccount<FungibleTokenContract>, account: &AccountId
) -> U128 {
    //near_view(&r, &token, "get_balance", &json!({ "owner_id": account }));
    let val = view!(token_contract.get_balance(account.to_string()));
    let value: U128 = val.unwrap_json();
    return value;
}

pub fn show_nep21_bal(
    token_contract: &ContractAccount<FungibleTokenContract>, account: &AccountId
) -> u128 {
    let bal: u128 = get_nep21_balance(token_contract, account).into();
    println!("Balance check: {} has {}", account, bal);
    return bal;
}

pub fn create_pool_add_liquidity(
    clp: &ContractAccount<NearCLPContract>,
    token_contract: &ContractAccount<FungibleTokenContract>,
    owner: &UserAccount,
    token: &UserAccount,
    near_amount: u128,
    token_amount: u128,
) {
    println!("{} creates a pool", owner.account_id());

    // Uses default gas amount, `near_sdk_sim::DEFAULT_GAS`
    let res = call!(
        owner,
        clp.create_pool(TOKEN_CONTRACT_ID.to_string().try_into().unwrap()),
        deposit = STORAGE_AMOUNT
    );
    println!("{:#?}\n Cost:\n{:#?}", res.status(), res.profile_data());
    assert!(res.is_ok());

    assert_eq!(
        get_pool_info(&clp, &TOKEN_CONTRACT_ID.to_string()),
        PoolInfo {
            ynear: 0.into(),
            reserve: 0.into(),
            total_shares: 0.into()
        },
        "new pool should be empty"
    );

    println!("Making sure owner has token before adding liq");
    let res1 = call!(
        token,
        token_contract.transfer(owner.account_id(), token_amount.into()),
        deposit = STORAGE_AMOUNT
    );
    println!("{:#?}\n Cost:\n{:#?}", res1.status(), res1.profile_data());
    assert!(res1.is_ok());

    add_liquidity(clp, token_contract, owner, token, near_amount, token_amount);
}

fn add_liquidity(
    clp: &ContractAccount<NearCLPContract>,
    token_contract: &ContractAccount<FungibleTokenContract>,
    liquidity_provider: &UserAccount,
    token: &UserAccount,
    near_amount: u128,
    token_amount: u128,
) {
    println!(
        "{} adds liquidity to {}",
        liquidity_provider.account_id(), token.account_id()
    );
    println!("creating allowance for CLP");
    let res = call!(
        liquidity_provider,
        token_contract.inc_allowance(NEARSWAP_CONTRACT_ID.to_string(), token_amount.into()),
        deposit = 2 * NEP21_STORAGE_DEPOSIT
    );
    println!("{:#?}\n Cost:\n{:#?}", res.status(), res.profile_data());
    assert!(res.is_ok());
    let val = view!(token_contract.get_allowance(
        liquidity_provider.account_id(), NEARSWAP_CONTRACT_ID.to_string())
    );
    let value: U128 = val.unwrap_json();
    println!("{:?} jsjdjks", value);

    //add_liquidity
    let res1 = call!(
        liquidity_provider,
        clp.add_liquidity(TOKEN_CONTRACT_ID.to_string(), U128(token_amount), U128(near_amount)),
        deposit = near_amount + NEP21_STORAGE_DEPOSIT
    );
    show_nep21_bal(&token_contract, &"nearswap".to_string());
    // TODO: Add separate test for add liquidity and pool creation
    // make setup function with pool creation and added liquidity

    let after_adding_info = get_pool_info(&clp, &"token".to_string());
    println!(
        "pool after add liq: {} {:?}",
        &token.account_id(),
        after_adding_info
    );
}
