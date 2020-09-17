import React, { useContext } from "react";

import findCurrencyLogoUrl from "../services/find-currency-logo-url";

import { GlobalContext } from "../contexts/GlobalContext";
import { TokenListContext } from "../contexts/TokenListContext";

import styled from "@emotion/styled";
const Tr = styled("tr")`
  &:hover {
    cursor: pointer;
  }
`;

// Parses token list to table
export const CurrencyTable = () => {
  
  // Global state
  const globalState = useContext(GlobalContext);
  const { dispatch } = globalState;

  // Token list state
  const tokenListState = useContext(TokenListContext);

  // Updates selected currency in global state and closes modal
  function handleCurrencyChange(newTokenIndex) {

    // Find URL of token logo
    let newImageUrl = findCurrencyLogoUrl(newTokenIndex, tokenListState.state.tokenList);
    let newSymbol = tokenListState.state.tokenList.tokens[newTokenIndex].symbol;

    // Find correct input to update
    switch (globalState.state.currencySelectionModal.selectedInput) {
      case 'from':
        dispatch({ type: 'UPDATE_FROM_SELECTED_CURRENCY',
          payload: { tokenIndex: newTokenIndex, logoUrl: newImageUrl, symbol: newSymbol }
        });
        break;
      case 'to':
        dispatch({ type: 'UPDATE_TO_SELECTED_CURRENCY',
          payload: { tokenIndex: newTokenIndex, logoUrl: newImageUrl, symbol: newSymbol }
        });
        break;
      case 'input1':
        dispatch({ type: 'UPDATE_INPUT1_SELECTED_CURRENCY',
          payload: { tokenIndex: newTokenIndex, logoUrl: newImageUrl, symbol: newSymbol }
        });
        break;
      case 'input2':
        dispatch({ type: 'UPDATE_INPUT2_SELECTED_CURRENCY',
          payload: { tokenIndex: newTokenIndex, logoUrl: newImageUrl, symbol: newSymbol }
        });
    } 
  }

  return (
    <>
      {tokenListState.state.tokenList.tokens.map((token, index) => (
        <Tr key={index} onClick={() => handleCurrencyChange(index)}>
          <td>
            {(() => {
              // Determine whether each token logo is served over HTTP/HTTPS or IPFS
              if (tokenListState.state.tokenList.tokens[index].logoURI.startsWith('ipfs://')) {
                // Token image is served over IPFS
                return <img src={process.env.REACT_APP_IPFS_GATEWAY + token.logoURI.substring(7)} width="25px" />
              } else {
                // Token image is served over HTTP/HTTPS
                return <img src={token.logoURI} width="25px" />
              }
            })()}
          </td>
          <td>{token.symbol}</td>
          <td>{token.name}</td>
        </Tr>
      ))}
    </>
  );
}
