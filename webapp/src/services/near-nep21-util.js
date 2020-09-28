import { Contract} from 'near-api-js'
import getConfig from '../config'

const nearConfig = getConfig(process.env.NODE_ENV || 'development')
const maxGas = '300000000000000';
const attachedNear = '60000000000000000000000';
const NDENOM = 1e24;

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

export async function incAllowance( token ) {
  window.nep21 = await new Contract(
    window.walletConnection.account(),
    token.address ,
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
      amount: token.amount},
      maxGas,
      attachedNear
      );
     console.log("DONE"); 
    return true;
  } catch(error) {
    return false;
  }

}

export async function getAllowance( token ) {
  const accountId = window.accountId;
  console.log(accountId);
  window.nep21 = await new Contract(
    window.walletConnection.account(),
    token.address ,
    {
      // View methods are read only
      viewMethods: ['get_allowance'],
      // Change methods modify state but don't receive updated data
      changeMethods: []
    }
  )
  const allowance = await window.nep21.get_allowance({
    owner_id: accountId, 
    escrow_account_id: nearConfig.contractName });
  console.log('Allowance: ', allowance);
  return allowance;  
}

export async function gasCheck() {
  // Set default
  const near_limit = 0.6;
  const bal = (await window.walletConnection.account().getAccountBalance()).available / NDENOM;
  if ( bal > near_limit ) {
    return true;
  }
  return false;
}

export function trimZeros( str ) {
  let start = 0;
  for (; start< str.length; ++start) {
    if (str[start] !== '0')  break;
  }
  let end = str.length - 1;
  for (; end> start; --end) {
    if (str[end] !== '0')  break;
  }
  if(str.includes(".") === false) {
    return str.slice(start, str.length);
  }
  return str.slice(start,end+1)
}

export function normalizeAmount( value ) {
  let ok = false;
  let val = 24;
  let res = "";
  const amount = trimZeros( value );
  for(var x of amount) {
    if(x === '.') {
      ok = true;
    }
    else if (x >= '0' && x <= '9') {
      if(ok)
        val--;
      res += x;
    } else {
      console.error("Error: Wrong Input");
      return -1;
    }
  }
  res = res + '0'.repeat(val);
  return res;
}

export async function calcPriceFromIn( token1, token2) {
  const amount1 = normalizeAmount( token1.amount );
  
  if(amount1 < 1) {
    return 0;
  }
  if(token1.type === "Native token") {
    // Native to NEP-21
    console.log("AMM ", amount1);
    const price = await window.contract.price_near_to_token_in( {
      token: token2.address, 
      ynear_in: amount1});
      console.log(price);
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
    else if(token2.type === "Native token") {
      // NEP-21 to Native
      const price = await window.contract.price_token_to_near_in( {
        token: token1.address, 
        tokens_in: token1.amount});
      return price;
    }
    else {
      console.log("Error: Token type error");
    }
  } 
}

export async function swapFromIn( token1, token2 ) {
  const amount1 = normalizeAmount( token1.amount );
  const amount2 = normalizeAmount( token2.amount );
  if(token1.type === "Native token") {
    // Native to NEP-21
    await window.contract.swap_near_to_token_exact_in( {
      token: token2.address, 
      min_tokens: token2.amount 
    },
    maxGas,
    attachedNear
    );
    
  }
  else {
    if(token2.type === "NEP-21") {
      // NEP-21 to NEP-21
      await window.contract.swap_tokens_exact_in( {
        from: token1.address,
        to: token2.address,
        from_tokens: token1.amount, 
        min_to_tokens: token2.amount },
        maxGas,
        attachedNear
        ); 
    
    }
    else if(token2.type === "Native token") {
      // NEP-21 to Native
        await window.contract.swap_token_to_near_exact_in( {
          token: token1.address,
          tokens_paid: token1.amount,
          min_ynear: amount2 },
          maxGas,
          attachedNear
          );
    
      }
    else {
      console.error("Error: Token type error");
    }
  } 
}

export async function calcPriceFromOut( token1, token2) {
  let amount2 = normalizeAmount( token2.amount );
  //console.log("amount_out ", amount2);
  if(amount2 < 1) {
    return 0;
  }
  if(token1.type === "Native token") {
    // Native to NEP-21
    const price = await window.contract.price_near_to_token_out( {
      token: token2.address, 
      tokens_out: token2.amount});
    console.log("expect_in ", price);
    return price;
  }
  else {
    if(token2.type === "NEP-21") {
      // NEP-21 to NEP-21
      const price = await window.contract.price_token_to_token_out( {
        from: token1.address,
        to: token2.address,
        tokens_out: token2.amount});
        console.log("expect_in ", price);
      return price;
    }
    else if(token2.type === "Native token") {
      // NEP-21 to Native
      const price = await window.contract.price_token_to_near_out( {
        token: token1.address, 
        ynear_out: amount2});
      return price;
    }
    else {
      console.log("Error: Token type error");
    }
  } 
}

export async function swapFromOut( token1, token2 ) {
  const amount1 = normalizeAmount( token1.amount );
  const amount2 = normalizeAmount( token2.amount );
  if(token1.type === "Native token") {
    // Native to NEP-21
    console.log("SWAP: amt", token2.amount);
    await window.contract.swap_near_to_token_exact_out( {
      token: token2.address, 
      tokens_out: token2.amount },
      maxGas,
      attachedNear
      ); 
  
  }
  else {
    if(token2.type === "NEP-21") {
      // NEP-21 to NEP-21
      await window.contract.swap_tokens_exact_out( {
        from: token1.address,
        to: token2.address,
        to_tokens: token2.amount, 
        max_from_tokens: token1.amount },
        maxGas,
        attachedNear
        ); 
      
    }
    else if(token2.type === "Native token") {
      // NEP-21 to Native
        await window.contract.swap_token_to_near_exact_out( {
          token: token1.address,
          ynear_out: amount2,
          max_tokens: token1.amount },
          maxGas,
          attachedNear
          );
      
      }
    else {
      console.error("Error: Token type error");
    }
  } 
}

export async function addLiquiduty( tokenDetails, maxTokenAmount, minSharesAmount ) {
  await window.contract.add_liquidity( { token: tokenDetails.address, 
    max_tokens: maxTokenAmount,
    min_shares: minSharesAmount},
    maxGas,
    attachedNear 
    );
}

// returns true if pool already exists
export async function createPool( tokenDetails, maxTokenAmount, minSharesAmount ) {
  const info = await window.contract.pool_info( { token: tokenDetails.address} );
  
  if(info !== null) {
    // Pool already exists. 
    return true;
  }
  await window.contract.create_pool( { token: tokenDetails.address });

  await addLiquiduty( tokenDetails.address, maxTokenAmount, minSharesAmount);

  return false;
}

export async function browsePools() {
  try {
    const poolInfo = await window.contract.list_pools();
    console.log(poolInfo);
    return poolInfo;
  } catch (error) {
    console.error('cannot fetch pool list');
  }
}

// eturns the owner balance of shares of a pool identified by token.
export async function sharesBalance( token ) {
  const bal = await window.contract.multi_balance_of ({ 
    token: token.address, 
    owner:  window.walletConnection.account() });
  return bal;
}
