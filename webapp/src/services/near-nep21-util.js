import { Contract} from 'near-api-js'
import getConfig from '../config'

const nearConfig = getConfig(process.env.NODE_ENV || 'development')

export async function getBalanceNEP( contractName ) {

  window.nep21 = await new Contract(
    window.walletConnection.account(),
    contractName ,
    {
      // View methods are read only
      viewMethods: ['get_balance'],
      // Change methods modify state but don't receive updated data
      changeMethods: []
    }
  )

  return await window.nep21.get_balance({ owner_id: window.walletConnection.getAccountId() });

}

export async function incAllowance( allowAmount ) {
  window.nep21 = await new Contract(
    window.walletConnection.account(),
    contractName ,
    {
      // View methods are read only
      viewMethods: [],
      // Change methods modify state but don't receive updated data
      changeMethods: ['inc_allowance']
    }
  )
  
  try {
    await window.nep21.inc_allowance({ 
      escrow_account_id: nearConfig.contractName, 
      amount: allowAmount});
    return true;
  } catch(error) {
    return false;
  }

}

export async function gasCheck() {
  // Set default
  const near_limit = 0.6;
  const bal = (await window.walletConnection.account().getAccountBalance()).available / 1000000000000000000000000;
  if ( bal > near_limit ) {
    return true;
  }
  return false;
}

export async function calcPriceFromIn( token1, token2) {
  if(token1.name === token2.name) {
    return token1.balance;
  }
  if(token1.type === "Native token") {
    // Native to NEP-21
    const price = await window.contract.price_near_to_token_in( {
      token: token2.address, 
      near_in: token1.amount});
    return price;
  }
  else {
    if(token2.type === "NEP-21") {
      // NEP-21 to NEP-21
      const price = await window.contract.price_token_to_token_in( {
        from: token1.address,
        to: token2.address,
        tokens_in: token1.amount});
      return price;
    }
    else {
      // NEP-21 to Native
      const price = await window.contract.price_token_to_near_in( {
        token: token1.address, 
        tokens_in: token1.amount});
      return price;
    }
  } 
}

export async function swapFromIn( token1, token2 ) {
  if(token1.name === token2.name) {
    return false;
  }
  if(token1.type === "Native token") {
    // Native to NEP-21
    const price = await window.contract.swap_near_to_reserve_exact_in( {
      token: token2.address, 
      min_tokens: 2 }); // ?? value
    return price;
  }
  else {
    if(token2.type === "NEP-21") {
      // NEP-21 to NEP-21
      const price = await window.contract.swap_tokens_exact_in( {
        from: token1.address,
        to: token2.address,
        tokens_from: 2, // ??
        min_tokens_to: 2 }); // ??
      return price;
    }
    else {
      // NEP-21 to Native
      const price = await window.contract.swap_reserve_to_near_exact_in( {
        token: token1.address,
        tokens_paid: 2, // ??
        min_near: 2 }); // ??
      return price;
    }
  } 
}