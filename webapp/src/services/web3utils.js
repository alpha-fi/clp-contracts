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

const getERC20Balance = async (web3, ethAddr, tokenAddr) => {
  
  const tokenContract = await new web3.eth.Contract(human_standard_token_abi, tokenAddr);

  const decimals = await tokenContract.methods.decimals().call();
  const balance = await tokenContract.methods.balanceOf(ethAddr).call();

  try {
    const adjustedBalance = balance / Math.pow(10, decimals);
    return adjustedBalance;
  } catch (error) {
    console.log(`Could not retrieve ERC-20 balance`);
  }
}


export { getERC20Balance };
