use std::convert::TryFrom;

use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::AccountId;
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, view, ContractAccount, UserAccount};

use near_sdk_sim::transaction::ExecutionStatus;
use nearswap::{NearSwapContract, PoolInfo};
use std::collections::HashMap;
use sample_token::ContractContract as SampleToken;

mod simulation_utils;
use simulation_utils::*;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    NEARSWAP_WASM_BYTES => "../res/nearswap.wasm",
}

#[test]
fn add_liquidity() {
    let root = init_simulator(None);
    let owner = root.create_user("owner".to_string(), to_yocto("100000"));
    let nearswap = deploy!(
        contract: NearSwapContract,
        contract_id: clp_contract(),
        bytes: &NEARSWAP_WASM_BYTES,
        signer_account: owner,
        init_method: new(to_va("owner".to_string()))
    );
    let token1 = sample_token(&owner, dai(), vec![clp_contract()]);
    let token2 = sample_token(&owner, eth(), vec![clp_contract()]);
    call!(
        owner,
        nearswap.extend_whitelisted_tokens(vec![to_va(dai()), to_va(eth())])
    );
    call!(
        owner,
        nearswap.create_pool(to_va("dai".into())),
        deposit = to_yocto("1")
    )
    .assert_success();

    let mut res = view!(nearswap.pool_info(&dai())).unwrap_json::<PoolInfo>();
    
    // verify created pool
    assert!(to_u128(res.ynear) == 0, "Near in pool incorrect");
    assert!(to_u128(res.tokens) == 0, "Tokens in pool incorrect");
    assert!(to_u128(res.total_shares) == 0, "Total shares in pool incorrect");

    // Register account
    call!(
        owner,
        nearswap.storage_deposit(None, Some(true)),
        deposit = to_yocto("1")
    )
    .assert_success();

    let minimum = 84_000_000_000_000_000_0000;
    let mut near_dep = view!(nearswap.get_deposit_near("owner".to_string())).unwrap_json::<U128>();
    // Near deposited during registration must be equal to minimum balance
    assert_eq!(near_dep, U128(minimum), "Minium Balance not satisfied");

    // Deposit more near in account deposit
    call!(
        owner,
        nearswap.storage_deposit(None, None),
        deposit = to_yocto("1")
    )
    .assert_success();

    near_dep = view!(nearswap.get_deposit_near("owner".to_string())).unwrap_json::<U128>();
    // One additional near should be deposited
    assert_eq!(near_dep, U128(minimum + to_yocto("1")), "Minium Balance not satisfied");

    call!(
        owner,
        nearswap.add_to_account_whitelist(&vec![to_va(dai()), to_va(eth())])
    )
    .assert_success();
    // Deposit tokens
    call!(
        owner,
        token1.ft_transfer_call(to_va(clp_contract()), to_yocto("105").into(), None, "".to_string()),
        deposit = 1
    )
    .assert_success();

    call!(
        owner,
        nearswap.add_liquidity(dai(), U128(123), U128(to_yocto("105")), U128(0)),
        deposit = 1
    )
    .assert_success();
    
    res = view!(nearswap.pool_info(&dai())).unwrap_json::<PoolInfo>();
    
    // verify created pool after adding liquidity
    assert!(to_u128(res.ynear) == 123, "Near in pool incorrect");
    assert!(to_u128(res.tokens) == to_yocto("105"), "Tokens in pool incorrect");
    assert!(to_u128(res.total_shares) == 123, "Total shares in pool incorrect");
}