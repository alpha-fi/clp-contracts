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
  // TO-DO
}