import React, { useContext, useEffect } from "react";

import findCurrencyLogoUrl from "../services/find-currency-logo-url";

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

// Parses token list to table
export const CurrencyTable = () => {
  
  // Inputs state
  const inputs = useContext(InputsContext);
  const { dispatch } = inputs;

  // Token list state
  const tokenListState = useContext(TokenListContext);

  // Updates selected currency in global state and closes modal
  function handleCurrencyChange(newTokenIndex) {

    // Find URL of token logo, symbol, type, and address
    let newImageUrl = findCurrencyLogoUrl(newTokenIndex, tokenListState.state.tokenList);
    let newSymbol = tokenListState.state.tokenList.tokens[newTokenIndex].symbol;
    let newType = tokenListState.state.tokenList.tokens[newTokenIndex].type;
    let newAddress = tokenListState.state.tokenList.tokens[newTokenIndex].address;

    // Find correct input to update
    switch (inputs.state.currencySelectionModal.selectedInput) {
      case 'from':
        dispatch({ type: 'UPDATE_FROM_SELECTED_CURRENCY',
          payload: { 
            tokenIndex: newTokenIndex, 
            logoUrl: newImageUrl, 
            symbol: newSymbol, 
            type: newType, 
            address: newAddress }
        });
        break;
      case 'to':
        dispatch({ type: 'UPDATE_TO_SELECTED_CURRENCY',
          payload: { 
            tokenIndex: newTokenIndex, 
            logoUrl: newImageUrl, 
            symbol: newSymbol, 
            type: newType, 
            address: newAddress }
        });
        break;
      case 'input1':
        dispatch({ type: 'UPDATE_INPUT1_SELECTED_CURRENCY',
          payload: { 
            tokenIndex: newTokenIndex, 
            logoUrl: newImageUrl, 
            symbol: newSymbol, 
            type: newType, 
            address: newAddress }
        });
        break;
      case 'input2':
        dispatch({ type: 'UPDATE_INPUT2_SELECTED_CURRENCY',
          payload: { 
            tokenIndex: newTokenIndex, 
            logoUrl: newImageUrl, 
            symbol: newSymbol, 
            type: newType, 
            address: newAddress }
        });
    }

    // Save selection in local storage
    dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
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
                ? <><FaEthereum/>{' '}ERC-20</>
                : token.type
            }</Badge>
          </td>
          <td className="text-right">
            {token.balance
              ? <code className="text-secondary">{token.balance}</code>
              : <code className="text-secondary">-</code>
            }
          </td>
        </Tr>
      ))}
    </>
  );
}
