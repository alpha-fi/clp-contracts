import { fill, hide, show } from './domHelpers'

const formatLargeNum = n => n >= 1e5 || (n < 1e-3 && n !== 0)
  ? n.toExponential(2)
  : new Intl.NumberFormat(undefined, { maximumSignificantDigits: 3 }).format(n)

// update the html based on user & data state
export default async function render () {
  // if not signed in, stop here
  if (!window.ethUserAddress) return
  fill('ethUser').with(window.ethUserAddress)
  fill('erc20Symbol').with(window.erc20Symbol)

  hide('signed-out')
  show('signed-in')

  const ethBalance = await window.ethProvider.getBalance(window.ethUserAddress)

  // if 0 ETH, cannot pay transaction fee to mint ERC20
  if (ethBalance.eq(0)) {
    show('eth-balance-zero')
    hide('eth-balance-positive')
    return
  }

  const erc20Balance = (await window.erc20.balanceOf(window.ethUserAddress)).toNumber()
  fill('erc20Balance').with(formatLargeNum(erc20Balance))

  hide('eth-balance-zero')
  show('eth-balance-positive')
}
