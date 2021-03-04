// TOOD: let's add a bit more liquidity
    // println!("Alice increases the liquidity right first top up");

    // add_liquidity(
    //     &mut ctx.r,
    //     &ctx.clp,
    //     &ctx.alice,
    //     &ctx.nep21_1,
    //     3 * NDENOM,
    //     30 * NDENOM + 1,
    // );

    //---------------
#[test]
fn add_liquidity() {
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

    //add_liquidity
    let res1 = call!(
        liquidity_provider,
        clp.add_liquidity(token_id.to_string(), U128(token_amount), U128(near_amount)),
        deposit = near_amount + NEP21_STORAGE_DEPOSIT
    );
    //show_nep21_bal(&token_contract, &"nearswap".to_string());
    // TODO: Add separate test for add liquidity and pool creation
    // make setup function with pool creation and added liquidity

    let after_adding_info = get_pool_info(&clp, &token_id.to_string());
    println!(
        "pool after add liq: {} {:?}",
        &token.account_id(),
        after_adding_info
    );
}