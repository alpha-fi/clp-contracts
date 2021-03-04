
#[test]
pub fn create_pool() {
    println!("{} creates a pool", owner.account_id());

    // Uses default gas amount, `near_sdk_sim::DEFAULT_GAS`
    let res = call!(
        owner,
        clp.create_pool(token_id.to_string().try_into().unwrap()),
        deposit = STORAGE_AMOUNT
    );
    println!("{:#?}\n Cost:\n{:#?}", res.status(), res.profile_data());
    assert!(res.is_ok());

    assert_eq!(
        get_pool_info(&clp, &token_id.to_string()),
        PoolInfo {
            ynear: 0.into(),
            reserve: 0.into(),
            total_shares: 0.into()
        },
        "new pool should be empty"
    );
}