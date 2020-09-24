import { human_standard_token_abi } from "./human_standard_token_abi";

const promisify = (inner) =>
  new Promise((resolve, reject) =>
    inner((err, res) => {
      if (err) {
        reject(err);
      } else {
        resolve(res);
      }
    })
  );

const getBalance = async (web3, ethAccount) => {
  let wei, balance;
  wei = promisify(callback => web3.eth.getBalance(ethAccount, callback));
  try {
    balance = await web3.utils.fromWei(await wei, 'ether');
    return balance;
  } catch (e) {
    console.log(`Could not retrieve Ether balance`);
    console.log(e);
  }
}

// This function is being called correctly but never successfully resolves
const getERC20Balance = async (web3, ethAddr, tokenAddr) => {
  let tokenContract, decimals, balance, name, symbol, adjustedBalance;

  tokenContract = new web3.eth.Contract(human_standard_token_abi, tokenAddr);

  // Gives error:
  // Uncaught (in promise) Error: Returned values aren't valid, did it run Out of Gas?
  // You might also see this error if you are not using the correct ABI for the contract
  // you are retrieving data from, requesting data from a block number that does not exist,
  // or querying a node which is not fully synced.
  decimals = promisify(callback => tokenContract.methods.decimals().call({ from: ethAddr }));
  balance = promisify(callback => tokenContract.methods.balanceOf(ethAddr).call({ from: ethAddr }));

  try {
    adjustedBalance = (await balance).balance / Math.pow(10, await decimals);
    return adjustedBalance;
  } catch (error) {
    console.log(`Could not retrieve ERC-20 balance`);
  }
}


export { getERC20Balance };
