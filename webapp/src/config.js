const CONTRACT_NAME = "beta-1.nearswap.testnet"
const ETH_WALLET_EXPLORER_PREFIX = "https://etherscan.io/address/";
const IPFS_PREFIX = "https://ipfs.infura.io:5001/api/v0/cat/";

export default function getConfig(env) {
  switch (env) {

  case 'production':
  case 'mainnet':
    return {
      networkId: 'mainnet',
      nodeUrl: 'https://rpc.mainnet.near.org',
      contractName: CONTRACT_NAME,
      walletUrl: 'https://wallet.near.org',
      helperUrl: 'https://helper.mainnet.near.org',
      explorerUrl: 'https://explorer.mainnet.near.org',
      addressPrefix: "https://wallet.mainnet.near.org/profile/",
      ethWalletExplorerPrefix: ETH_WALLET_EXPLORER_PREFIX,
      ethChainId: 1,
      ipfsPrefix: IPFS_PREFIX,
      infuraId: "",
    }
  case 'development':
  case 'testnet':
    return {
      networkId: 'testnet',
      nodeUrl: 'https://rpc.testnet.near.org',
      contractName: CONTRACT_NAME,
      walletUrl: 'https://wallet.testnet.near.org',
      helperUrl: 'https://helper.testnet.near.org',
      explorerUrl: 'https://explorer.testnet.near.org',
      addressPrefix: "https://wallet.testnet.near.org/profile/",
      ethWalletExplorerPrefix: ETH_WALLET_EXPLORER_PREFIX,
      ethChainId: 4, // rinkeby
      ipfsPrefix: IPFS_PREFIX,
      infuraId: "",
    }
  case 'betanet':
    return {
      networkId: 'betanet',
      nodeUrl: 'https://rpc.betanet.near.org',
      contractName: CONTRACT_NAME,
      walletUrl: 'https://wallet.betanet.near.org',
      helperUrl: 'https://helper.betanet.near.org',
      explorerUrl: 'https://explorer.betanet.near.org',
      addressPrefix: "https://wallet.betanet.near.org/profile/",
    }
  case 'local':
    return {
      networkId: 'local',
      nodeUrl: 'http://localhost:3030',
      keyPath: `${process.env.HOME}/.near/validator_key.json`,
      walletUrl: 'http://localhost:4000/wallet',
      contractName: CONTRACT_NAME,
    }
  case 'test':
  case 'ci':
    return {
      networkId: 'shared-test',
      nodeUrl: 'https://rpc.ci-testnet.near.org',
      contractName: CONTRACT_NAME,
      masterAccount: 'test.near',
    }
  case 'ci-betanet':
    return {
      networkId: 'shared-test-staging',
      nodeUrl: 'https://rpc.ci-betanet.near.org',
      contractName: CONTRACT_NAME,
      masterAccount: 'test.near',
    }
  default:
    throw Error(`Unconfigured environment '${env}'. Can be configured in src/config.js.`)
  }
}

module.exports = getConfig
