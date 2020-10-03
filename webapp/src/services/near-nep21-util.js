import { Contract} from 'near-api-js'

const e22 = '0'.repeat(22);
const maxGas = '300000000000000';
const attach60NearCents = '6' + e22;
const nep21AllowanceFee = '4' + e22;
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
  const res = await window.nep21.get_balance({ owner_id: window.walletConnection.getAccountId() });
  return convertToE24Base(res);

}

export async function incAllowance( token ) {
  const AMOUNT = normalizeAmount( token.amount )
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
      escrow_account_id: window.config.contractName,
      amount: AMOUNT},
      maxGas,
      nep21AllowanceFee
      );
     console.log("DONE");
    return true;
  } catch(error) {
    return false;
  }

}

export async function getAllowance( token ) {
  const accountId = window.accountId;
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
    escrow_account_id: window.config.contractName });
  console.log('Allowance: ', allowance);
  return convertToE24Base(allowance);
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

export function convertToE24Base( str ) {

  let result = (str +"").padStart(24,"0")
  result = result.slice(0,-24)+"."+result.slice(-24)
  return result
/*
  const append = 25 - str.length;
  if(append > 0) {
    str = '0'.repeat(append) + str;
  }
  const pos = str.length - 24;
  var res = [str.slice(0, pos), '.', str.slice(pos)].join('');
  res = trimZeros(res);

  if(res[0] === '.')
    res = '0' + res;

  if(res[res.length - 1] === '.')
    res = res.slice(0, res.length - 1);
  return res;
  */
}

export function trimZeros( str ) {
  if (typeof str!=="string") return str;
  let start = 0;
  for (; start< str.length; ++start) {
    if (str[start] !== '0')  break;
  }
  let end = str.length - 1;
  for (; end> start; --end) {
    if (str[end] !== '0')  break;
  }
  if(str.includes(".") === false) {
    str = str.slice(start, str.length);
    if(str === "")
      return "0";
    return str;
  }
  var res = str.slice(start,end+1);
  if(res === "" || res === ".")
    return "0";
  return res;
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

function toYoctosString(amount){
  let result = amount + ""
  let pos = result.indexOf(".");
  let decimals=0;
  if (pos>=0){
    decimals = result.length-pos;
  }
  else    {
    decimals = 24;
  }
  result = result + "0".repeat(decimals)
  result = result.replace(".","")
  return result

}

export async function calcPriceFromIn(token1, token2) {
  //const amount1 = normalizeAmount( token1.amount );

  if(token1.amount < 1) {
    return 0;
  }
  if(token1.type === "Native token") {
    // Native to NEP-21
    const price = await window.contract.price_near_to_token_in( {
      token: token2.address,
      ynear_in: toYoctosString(token1.amount)});
      console.log(price);
    return convertToE24Base(price);
  }
  else {
    if(token2.type === "NEP-21") {
      // NEP-21 to NEP-21
      const price = await window.contract.price_token_to_token_in( {
        from: token1.address,
        to: token2.address,
        tokens_in: toYoctosString(token1.amount)});
      return convertToE24Base(price);
    }
    else if(token2.type === "Native token") {
      // NEP-21 to Native
      const price = await window.contract.price_token_to_near_in( {
        token: token1.address,
        tokens_in: toYoctosString(token1.amount)});
      return convertToE24Base(price);
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
      min_tokens: amount2
    },
    maxGas,
    attach60NearCents
    );

  }
  else {
    if(token2.type === "NEP-21") {
      // NEP-21 to NEP-21
      await window.contract.swap_tokens_exact_in( {
        from: token1.address,
        to: token2.address,
        tokens_in: amount1,
        min_tokens_out: amount2 },
        maxGas,
        attach60NearCents
        );

    }
    else if(token2.type === "Native token") {
      // NEP-21 to Native
        await window.contract.swap_token_to_near_exact_in( {
          token: token1.address,
          tokens_paid: amount1,
          min_ynear: amount2 },
          maxGas,
          attach60NearCents
          );

      }
    else {
      console.error("Error: Token type error");
    }
  }
}

export async function calcPriceFromOut( token1, token2) {
  let amount2 = normalizeAmount( token2.amount );
  if(amount2 < 1) {
    return 0;
  }
  if(token1.type === "Native token") {
    // Native to NEP-21
    const price = await window.contract.price_near_to_token_out( {
      token: token2.address,
      tokens_out: amount2});
    console.log("expect_in ", price);
    return convertToE24Base(price);
  }
  else {
    if(token2.type === "NEP-21") {
      // NEP-21 to NEP-21
      const price = await window.contract.price_token_to_token_out( {
        from: token1.address,
        to: token2.address,
        tokens_out: amount2});
        console.log("expect_in ", price);
      return convertToE24Base(price);
    }
    else if(token2.type === "Native token") {
      // NEP-21 to Native
      const price = await window.contract.price_token_to_near_out( {
        token: token1.address,
        ynear_out: amount2});
      return convertToE24Base(price);
    }
    else {
      console.log("Error: Token type error");
    }
  }
}

export async function swapFromOut( tokenIN, tokenOUT ) {
  const amountIN = normalizeAmount( tokenIN.amount );
  const amountOUT = normalizeAmount( tokenOUT.amount );
  if(tokenIN.type === "Native token") {
    // Native to NEP-21
    //NEARs IN / Tokens out
    await window.contract.swap_near_to_token_exact_out( {
      token: tokenOUT.address,
      tokens_out: amountOUT },
      maxGas,
      amountIN //near-in
      );

  }
  else {
    if(tokenOUT.type === "NEP-21") {
      // NEP-21 to NEP-21
      await window.contract.swap_tokens_exact_out( {
        from: tokenIN.address,
        to: tokenOUT.address,
        tokens_out: amountOUT,
        max_tokens_in: amountIN },
        maxGas,
        attach60NearCents
        );

    }
    else if(tokenOUT.type === "Native token") {
      // NEP-21 to Native
        await window.contract.swap_token_to_near_exact_out( {
          token: tokenIN.address,
          ynear_out: amountOUT,
          max_tokens: amountIN },
          maxGas,
          attach60NearCents
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
    attach60NearCents
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
    const pools = await window.contract.list_pools();
    console.log(pools);
    return pools;
  } catch (error) {
    console.error('cannot fetch pool list');
  }
}

export async function poolInfo(pool) {
  try {
    const poolInfo = await window.contract.pool_info({token: pool});
    console.log(poolInfo);
    return poolInfo;
  } catch (error) {
    console.error('cannot fetch pool info');
  }
}

// eturns the owner balance of shares of a pool identified by token.
export async function sharesBalance( token ) {
  const bal = await window.contract.balance_of ({
    token: token.address,
    owner:  window.walletConnection.account()});
  return bal;
}
