import { connect, Contract, keyStores, WalletConnection } from 'near-api-js'
import getConfig from './config'
import { view_methods_lib_contract, call_methods_lib_contract } from './assets/contractMethods'

const nearConfig = getConfig(process.env.NODE_ENV || 'development')

// Initialize contract & set global variables
export async function initContract() {
  // Initialize connection to the NEAR testnet
  const near = await connect(Object.assign({ deps: { keyStore: new keyStores.BrowserLocalStorageKeyStore() } }, nearConfig))

  // Initializing Wallet based Account. It can work with NEAR testnet wallet that
  // is hosted at https://wallet.testnet.near.org
  window.walletConnection = new WalletConnection(near)

  // Getting the Account ID. If still unauthorized, it's just empty string
  window.accountId = window.walletConnection.getAccountId()

  // Initializing our contract APIs by contract name and configuration
  window.contract = await new Contract(window.walletConnection.account(), window.config.contractName, {
    // View methods are read only. They don't modify the state, but usually return some value.
    viewMethods: view_methods_lib_contract,
    // Change methods can modify the state. But you don't receive the returned value when called.
    changeMethods: call_methods_lib_contract
  })
}

export function logout() {
  window.walletConnection.signOut()
  // reload page
  window.location.replace(window.location.origin + window.location.pathname)
}

export function login() {
  // Allow the current app to make calls to the specified contract on the
  // user's behalf.
  // This works by creating a new access key for the user's account and storing
  // the private key in localStorage.
  window.walletConnection.requestSignIn(window.config.contractName)
}

export function isNonzeroNumber(num) {
  return (!isNaN(num) && (num > 0));
}

export function delay(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}
