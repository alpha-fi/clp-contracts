use std::convert::TryFrom;

use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::AccountId;
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount};

use nearswap::{NearSwapContract};
use sample_token::ContractContract as SampleToken;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    SAMPLE_TOKEN_WASM_BYTES => "../res/sample_token.wasm",
    NEARSWAP_WASM_BYTES => "../res/nearswap.wasm",
}

pub fn deploy(creator: &str) -> (UserAccount, UserAccount, ContractAccount<NearSwapContract>) {
    let root = init_simulator(None);
    let owner = root.create_user("owner".to_string(), to_yocto("100000"));
    let nearswap = deploy!(
        contract: NearSwapContract,
        contract_id: clp_contract(),
        bytes: &NEARSWAP_WASM_BYTES,
        signer_account: owner,
        init_method: new(to_va(creator.into()))
    );
    return (root, owner, nearswap);
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
        signer_account: creator
    );
    call!(creator, t.new()).assert_success();
    call!(
        creator,
        t.mint(to_va(creator.account_id.clone()), to_yocto("1000").into())
    )
    .assert_success();
    for account_id in accounts_to_register {
        call!(
            creator,
            t.storage_deposit(Some(to_va(account_id)), None),
            deposit = to_yocto("1")
        )
        .assert_success();
    }
    t
}

pub fn mint(
    token: &ContractAccount<SampleToken>, recipient: &UserAccount,
    creator: &UserAccount, amount: u128
) {
    call!(
        creator,
        token.mint(to_va(recipient.account_id.clone()), amount.into())
    )
    .assert_success();
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

pub fn create_pools(
    nearswap: &ContractAccount<NearSwapContract>,
    owner: &UserAccount) {
    call!(
        owner,
        nearswap.create_pool(to_va("dai".into()))
    )
    .assert_success();
    call!(
        owner,
        nearswap.create_pool(to_va("eth".into()))
    )
    .assert_success();
}

pub fn register_deposit_acc(
    nearswap: &ContractAccount<NearSwapContract>,
    owner: &UserAccount, amount: u128) {
    // Register account
    call!(
        owner,
        nearswap.storage_deposit(None, Some(true)),
        deposit = to_yocto("1")
    )
    .assert_success();
    // Deposit more near in account deposit
    call!(
        owner,
        nearswap.storage_deposit(None, None),
        deposit = amount
    )
    .assert_success();
}
