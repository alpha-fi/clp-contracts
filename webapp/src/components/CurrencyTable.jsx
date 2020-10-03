import React, { useContext, useEffect } from "react";

import { getBalanceNEP, convertToE24Base5Dec, getAllowance } from '../services/near-nep21-util'

import findCurrencyLogoUrl from "../services/find-currency-logo-url";
import { delay } from "../utils"

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


//set the currency of one of the 4 fields
export function setCurrencyIndex(inputName, newTokenIndex, inputs, tokenListState ) {

  // Find URL of token logo, symbol, type, and address
  let newImageUrl = findCurrencyLogoUrl(newTokenIndex, tokenListState.state.tokenList);
  let newSymbol = tokenListState.state.tokenList.tokens[newTokenIndex].symbol;
  let newType = tokenListState.state.tokenList.tokens[newTokenIndex].type;
  let newAddress = tokenListState.state.tokenList.tokens[newTokenIndex].address;
//  let newBalance = tokenListState.state.tokenList.tokens[newTokenIndex].balance;
  let newPayload = {
    tokenIndex: newTokenIndex,
    logoUrl: newImageUrl,
    symbol: newSymbol,
    type: newType,
    address: newAddress,
//    balance: newBalance,
  };

  // Find correct input to update
  switch (inputName) {
    case 'in':
      inputs.dispatch({ type: 'UPDATE_IN_SELECTED_CURRENCY', payload: newPayload });
      break;
    case 'out':
      inputs.dispatch({ type: 'UPDATE_OUT_SELECTED_CURRENCY', payload: newPayload });
      break;
    case 'input1':
      inputs.dispatch({ type: 'UPDATE_INPUT1_SELECTED_CURRENCY', payload: newPayload });
      break;
    case 'input2':
      inputs.dispatch({ type: 'UPDATE_INPUT2_SELECTED_CURRENCY', payload: newPayload });
  }
}

export function saveInputsLocalStorage(inputs) {
  localStorage.setItem("inputs", JSON.stringify(inputs.state));
}

async function updateSwapBal(token, tokenName, tokenListState) {
  let yoctos = ""
  try {
    if (token.type === "Native token") {
      yoctos = (await window.walletConnection.account().getAccountBalance()).available;
    }
    else {
      yoctos = await getBalanceNEP(token.address);
    }
  } catch (ex) {
    console.log(ex)
    yoctos = ex.message;
  }

  tokenListState.dispatch({
    type: 'SET_TOKEN_BALANCE',
    payload: {
      name: tokenName,
      balance: yoctos
    }
})

}

// Parses token list to table
export const CurrencyTable = () => {

  const inputs = useContext(InputsContext);
  const { dispatch } = inputs;
  
  // Token list state
  const tokenListState = useContext(TokenListContext);

  useEffect(() => {
    tokenListState.state.tokenList.tokens.map((token, index) => {
      updateSwapBal(token, token.name, tokenListState);
    })
  }, []);

  // Inputs state
  // Updates allowance of from token
  async function updateInAllowance(token) {
    await delay(500).then(async function () {
      if (token.type == "NEP-21") {
        try {
          let allowance = await getAllowance(token);
          dispatch({ type: 'UPDATE_IN_ALLOWANCE', payload: { allowance: allowance } });
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
    let newPayload = tokenListState.state.tokenList.tokens[newTokenIndex];
    setCurrencyIndex(name, newTokenIndex, inputs, tokenListState)
    if (name=="in") updateInAllowance(newPayload);

    // Save selection in local storage
    saveInputsLocalStorage(inputs);
  }

  return (
    <>
      {tokenListState.state.tokenList.tokens.map((token, index) => (
        <Tr key={index} onClick={() => handleCurrencyChange(index)}>
          <td>
            {/* Determine whether each token logo is served over HTTP/HTTPS or IPFS */}
            {tokenListState.state.tokenList.tokens[index].logoURI.startsWith('ipfs://')
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
              ? <code className="text-secondary">{convertToE24Base5Dec(token.balance)}</code>
              : <code className="text-secondary">-</code>
            }
          </td>
        </Tr>
      ))}
    </>
  );
}
