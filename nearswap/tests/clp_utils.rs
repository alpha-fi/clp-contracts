// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

#![allow(unused)]

use near_sdk_sim::account::AccessKey;
use near_sdk_sim::{
    call, deploy, init_simulator, near_crypto::Signer, to_yocto, view, ContractAccount,
    UserAccount, STORAGE_AMOUNT,
};

use near_primitives::types::{AccountId, Balance};
use near_sdk::json_types::{ValidAccountId, U128, U64};
use nearswap::util::*;
use nearswap::{NearCLPContract, PoolInfo};
use nep21_mintable::FungibleTokenContract;
use serde_json::json;
use std::convert::TryInto;

pub const NEARSWAP_CONTRACT_ID: &str = "nearswap";

// Load in contract bytes at runtime. Current directory = closes Cargo.toml file location
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    CLP_WASM_BYTES => "../res/nearswap.wasm"
}

// Deploy NearCLP Contract
pub fn deploy_clp() -> (
    UserAccount,
    ContractAccount<NearCLPContract>,
    UserAccount,
    UserAccount,
    UserAccount,
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

pub fn get_pool_info(clp: &ContractAccount<NearCLPContract>, token: &AccountId) -> PoolInfo {
    let val = view!(clp.pool_info(token));
    let value: PoolInfo = val.unwrap_json();
    return value;
}

pub fn create_pool_add_liquidity(
    clp: &ContractAccount<NearCLPContract>,
    token_contract: &ContractAccount<FungibleTokenContract>,
    owner: &UserAccount,
    token: &UserAccount,
    token_id: AccountId,
    near_amount: u128,
    token_amount: u128,
) {
    println!("{} creates a pool", owner.account_id());
    // Uses default gas amount, `near_sdk_sim::DEFAULT_GAS`
    // Create Pool
    call!(
        owner,
        clp.create_pool(token_id.to_string().try_into().unwrap()),
        deposit = STORAGE_AMOUNT
    )
    .assert_success();

    // Fund owner account with tokens
    call!(
        token,
        token_contract.transfer(owner.account_id(), token_amount.into()),
        deposit = STORAGE_AMOUNT
    )
    .assert_success();

    // increase allowance
    call!(
        owner,
        token_contract.inc_allowance(NEARSWAP_CONTRACT_ID.to_string(), token_amount.into()),
        deposit = 2 * NEP21_STORAGE_DEPOSIT
    )
    .assert_success();

    //add_liquidity
    call!(
        owner,
        clp.add_liquidity(token_id.to_string(), U128(token_amount), U128(near_amount)),
        deposit = near_amount + NEP21_STORAGE_DEPOSIT
    )
    .assert_success();
}
