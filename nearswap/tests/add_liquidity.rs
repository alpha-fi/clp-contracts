use std::convert::TryFrom;

use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::AccountId;
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, view, ContractAccount, UserAccount};

use near_sdk_sim::transaction::ExecutionStatus;
use nearswap::{NearSwapContract, PoolInfo};
use std::collections::HashMap;
use sample_token::ContractContract as SampleToken;

mod nep141_utils;
use nep141_utils::*;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    NEARSWAP_WASM_BYTES => "../res/nearswap.wasm",
}

#[test]
fn add_liquidity() {
    let root = init_simulator(None);
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let nearswap = deploy!(
        contract: NearSwapContract,
        contract_id: clp_contract(),
        bytes: &NEARSWAP_WASM_BYTES,
        signer_account: root,
        init_method: new(to_va("owner".to_string()))
    );
    let token1 = sample_token(&root, dai(), vec![clp_contract()]);
    let token2 = sample_token(&root, eth(), vec![clp_contract()]);
    call!(
        owner,
        nearswap.extend_whitelisted_tokens(vec![to_va(dai()), to_va(eth())])
    );
    call!(
        root,
        nearswap.create_pool(to_va("dai".into())),
        deposit = to_yocto("1")
    )
    .assert_success();

    // Register account
    call!(
        root,
        nearswap.storage_deposit(None, Some(true)),
        deposit = to_yocto("1")
    )
    .assert_success();

    // Deposit more near in account deposit
    call!(
        root,
        nearswap.storage_deposit(None, None),
        deposit = to_yocto("1")
    )
    .assert_success();

    // Deposit tokens
    call!(
        root,
        token1.ft_transfer_call(to_va(clp_contract()), to_yocto("105").into(), None, "".to_string()),
        deposit = 1
    )
    .assert_success();

    call!(
        root,
        nearswap.add_liquidity(dai(), U128(to_yocto("5")), U128(to_yocto("105")), U128(0)),
        deposit = 1
    )
    .assert_success();
}