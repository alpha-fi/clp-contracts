use std::convert::TryFrom;

use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::AccountId;
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, view, ContractAccount, UserAccount};

use near_sdk_sim::transaction::ExecutionStatus;
use nearswap::{NearSwapContract, PoolInfo};
use std::collections::HashMap;
use sample_token::ContractContract as SampleToken;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    SAMPLE_TOKEN_WASM_BYTES => "../res/sample_token.wasm",
}

pub fn sample_token(
    creator: &UserAccount,
    token_id: AccountId,
    accounts_to_register: Vec<AccountId>,
) -> ContractAccount<SampleToken> {
    let t = deploy!(
        contract: SampleToken,
        contract_id: token_id,
        bytes: &SAMPLE_TOKEN_WASM_BYTES,
        signer_account: root
    );
    call!(root, t.new()).assert_success();
    call!(
        root,
        t.mint(to_va(root.account_id.clone()), to_yocto("1000").into())
    )
    .assert_success();
    for account_id in accounts_to_register {
        call!(
            root,
            t.storage_deposit(Some(to_va(account_id)), None),
            deposit = to_yocto("1")
        )
        .assert_success();
    }
    t
}

pub fn dai() -> AccountId {
    "dai".to_string()
}

pub fn eth() -> AccountId {
    "eth".to_string()
}

pub fn clp_contract() -> AccountId {
    "clp_contract".to_string()
}

pub fn to_va(a: AccountId) -> ValidAccountId {
    ValidAccountId::try_from(a).unwrap()
}

pub fn to_u128(a: U128) -> u128 {
    return u128::try_from(a).unwrap();
}
