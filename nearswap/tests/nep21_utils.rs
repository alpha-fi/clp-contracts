#![allow(unused)]

use near_sdk_sim::{
    call, deploy, init_simulator, near_crypto::Signer, to_yocto, view, ContractAccount,
    UserAccount, STORAGE_AMOUNT,
};

use near_primitives::types::{AccountId, Balance};
use near_sdk::json_types::{U128, U64};
use nearswap::util::*;
use nearswap::{NearSwapContract, PoolInfo};
use nep21_mintable::FungibleTokenContract;
use serde_json::json;
use std::convert::TryInto;

// Load in contract bytes at runtime. Current directory = closes Cargo.toml file location
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    NEP21_BYTES => "../res/nep21_mintable.wasm"
}

// Deploy NEP-21 Contract
pub fn deploy_nep21(
    master_account: &UserAccount,
    contract: AccountId,
    total_supply: U128,
) -> ContractAccount<FungibleTokenContract> {
    println!("deploy_nep21");
    // uses default values for deposit and gas
    let contract_user = deploy!(
        // Contract Proxy
        contract: FungibleTokenContract,
        // Contract account id
        contract_id: contract,
        // Bytes of contract
        bytes: &NEP21_BYTES,
        // User deploying the contract,
        signer_account: master_account,
        // init method
        init_method: new(master_account.account_id(), total_supply, 24)
    );
    contract_user
}

pub fn get_nep21_balance(
    token_contract: &ContractAccount<FungibleTokenContract>,
    account: &AccountId,
) -> U128 {
    //near_view(&r, &token, "get_balance", &json!({ "owner_id": account }));
    let val = view!(token_contract.get_balance(account.to_string()));
    let value: U128 = val.unwrap_json();
    return value;
}

pub fn show_nep21_bal(
    token_contract: &ContractAccount<FungibleTokenContract>,
    account: &AccountId,
) -> u128 {
    let bal: u128 = get_nep21_balance(token_contract, account).into();
    println!("Balance check: {} has {}", account, bal);
    return bal;
}
