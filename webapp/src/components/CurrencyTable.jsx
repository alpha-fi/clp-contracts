import React, { useContext, useEffect } from "react";

import findCurrencyLogoUrl from "../services/find-currency-logo-url";
import { getAllowance, convertToE24Base5Dec, getBalanceNEP } from '../services/near-nep21-util'
import { calcPriceFromIn } from "../services/near-nep21-util";
import { delay, isNonzeroNumber } from "../utils"

import { InputsContext } from "../contexts/InputsContext";
import { TokenListContext } from "../contexts/TokenListContext";

import Badge from 'react-bootstrap/Badge';

import { FaEthereum } from "react-icons/fa";

import styled from "@emotion/styled";
const Tr = styled("tr")`
  &:hover {
    cursor: pointer;
  }
`;

//fetch balance for token INDEX
export async function getCurrentBalance(tokenIndex, tokenListState) {
  let yoctos = ""
  let token = tokenListState.state.tokens[tokenIndex]
  if (window.walletConnection.getAccountId() === "") {
    return;
  }
  try {
    if (token.type === "Native token") {
      yoctos = (await window.walletConnection.account().getAccountBalance()).available;
    }
    else {
      yoctos = await getBalanceNEP(token.address);
    }
  }
  catch (ex) {
    console.log(ex)
    yoctos = ex.message;
  }

  return yoctos; //returns balance

}


//set the currency of one of the 4 fields in the swap tab
//copying data from the currencyTable
export function setCurrencyIndex(inputName, newTokenIndex, inputs, tokenListState) {

  // Find URL of token logo, symbol, type, and address
  let newImageUrl = findCurrencyLogoUrl(newTokenIndex, tokenListState.state.tokens);
  let newSymbol = tokenListState.state.tokens[newTokenIndex].symbol;
  let newType = tokenListState.state.tokens[newTokenIndex].type;
  let newAddress = tokenListState.state.tokens[newTokenIndex].address;
  let newBalance = tokenListState.state.tokens[newTokenIndex].balance;
  let newPayload = {
    tokenIndex: newTokenIndex,
    logoUrl: newImageUrl,
    symbol: newSymbol,
    type: newType,
    address: newAddress,
    balance: newBalance,
    allowance: 0,
  };

  // Find correct input to update
  switch (inputName) {

    case 'in':
      inputs.dispatch({ type: 'UPDATE_IN_SELECTED_CURRENCY', payload: newPayload });
      // Calculate the value of the other input box (only called when the user types)
      //let updatedToken = { ...inputs.state.swap.out, amount: outAmount };
      calcPriceFromIn(inputs.state.swap.out, inputs.state.swap.in)
        .then(function (result) {
          inputs.dispatch({ type: 'SET_OUT_AMOUNT', payload: { amount: result, isValid: isNonzeroNumber(result) } });
          //updateStatus(result, outAmount); // Update status and/or error message
        })
      break;

    case 'out':
      inputs.dispatch({ type: 'UPDATE_OUT_SELECTED_CURRENCY', payload: newPayload });

    case 'input1':
      inputs.dispatch({ type: 'UPDATE_INPUT1_SELECTED_CURRENCY', payload: newPayload });
      break;
    case 'input2':
      inputs.dispatch({ type: 'UPDATE_INPUT2_SELECTED_CURRENCY', payload: newPayload });
  }
}

export function saveInputsStateLocalStorage(state) {
  localStorage.setItem("inputs", JSON.stringify(state));
}



//--------------------------------
// Parses token list to table
export const CurrencyTable = () => {


  const inputs = useContext(InputsContext);
  const { dispatch } = inputs;

  // Token list state
  const tokenListState = useContext(TokenListContext);

  function asyncLoadAllETHBalances(tokenList, w3, ethAccount) {
    let tl = tokenList;
    //ASYNC MAP
    tl.tokens.map(async (token, index) => {
  
      if (token.type === "ERC-20" && w3 && ethAccount && token.address !== "") {
  
        let ERC20Balance = await getERC20Balance(w3, ethAccount, token.address);
  
        tokenListState.dispatch({
          type: 'SET_TOKEN_LIST_BALANCE',
          payload: {
            tokenIndex: index,
            balance: ERC20Balance
          }
        });
  
      }
    });
    return tl;
  }
  
  //----------------------
  //load current balance for each item in the tokenList
  //----------------------
  function asyncLoadAllBalances(tokenListState) {
  
    let countPending = tokenListState.state.tokens.length;
  
    //MAP ASYNC - query balances
    tokenListState.state.tokens.map(async (token, index) => {
  
      try {
  
        let yoctos = await getCurrentBalance(index, tokenListState)
  
        tokenListState.dispatch({
          type: 'SET_TOKEN_LIST_BALANCE',
          payload: {
            tokenIndex: index,
            balance: yoctos
          }
        });
      }
      catch (ex) {
        console.log(ex)
      }
  
      countPending--;
      // if (countPending == 0) {
      //   inputs.dispatch({ type: 'ALL_BALANCES_LOADED' })
      // }
  
    });
  }
  
  //---------------------------------------------------------------
  //  after mounted / recovery after SDE -----------------------
  //---------------------------------------------------------------
  // Load actual balance for each configures symbl (async)
  //---------------------------------------------------------------
  //---------------------------------------------------------------
  useEffect(() => {

    // Update balances of NEAR tokens
    if (window.walletConnection.isSignedIn()) {

      asyncLoadAllBalances(tokenListState);

    }

    // // Update balances of ETH tokens
    // if (currentUser) {
    //   try {
    //     // Inject token balances in token list using wallet information
    //     asyncLoadAllETHBalances(tokenListState);
    //   } catch (e) {
    //     console.error(`Could not inject ETH balances`);
    //   }
    // }

  }, []);
  //}, [currentUser, web3Modal]);


  // Inputs state
  // Updates allowance of from token
  async function updateFromAllowance(token) {
    await delay(500).then(async function () {
      if (token.type == "NEP-21") {
        try {
          let allowance = await getAllowance(token);
          let needsApproval = true;
          try{ needsApproval = inputs.state.swap.in.allowance<inputs.state.swap.in.amount } catch (ex){};
          dispatch({ type: 'UPDATE_IN_ALLOWANCE', payload: { allowance: allowance, needsApproval:needsApproval } });
        } catch (e) {
          console.error(e);
        }
      }
    });
  }


  // Updates selected currency in global state and closes modal
  function handleCurrencyChange(newTokenIndex) {
    //update active input
    const name = inputs.state.currencySelectionModal.selectedInput
    setCurrencyIndex(name, newTokenIndex, inputs, tokenListState)
    if (name == "in") {
      updateFromAllowance(inputs.state.swap.in);
    }

    // Save selection in local storage
    saveInputsStateLocalStorage(inputs.state);
  }


  return (
    <>
      {tokenListState.state.tokens.map((token, index) => (
        <Tr key={index} onClick={() => handleCurrencyChange(index)}>
          <td>
            {/* Determine whether each token logo is served over HTTP/HTTPS or IPFS */}
            {tokenListState.state.tokens[index].logoURI.startsWith('ipfs://')
              ?
              // Token image is served over IPFS
              <img src={process.env.REACT_APP_IPFS_GATEWAY + token.logoURI.substring(7)} width="25px" />
              :
              // Token image is served over HTTP/HTTPS
              <img src={token.logoURI} width="25px" />
            }
          </td>
          <td>
            {token.name} ({token.symbol})
            {' '}
            <Badge variant="secondary" className="ml-1">{
              token.type === "ERC-20"
                ? <><FaEthereum />{' '}ERC-20</>
                : token.type
            }</Badge>
          </td>
          <td className="text-right">
            {token.balance
              ? <code className="text-secondary">{token.type==="ERC-20"?token.balance:convertToE24Base5Dec(token.balance)}</code>
              : <code className="text-secondary">-</code>
            }
          </td>
        </Tr>
      ))}
    </>
  );

}
