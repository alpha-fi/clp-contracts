import React, { useState, useContext } from "react";

import Modal from 'react-bootstrap/Modal';
import Table from 'react-bootstrap/Table';
import Button from 'react-bootstrap/Button';

import { GlobalContext } from "../contexts/GlobalContext";

import { TokenListContext } from "../contexts/TokenListContext";

import styled from "@emotion/styled";
const PointerHover = styled("tr")`
  &:hover {
    cursor: pointer;
  }
`;
const LimitedHeightTable = styled("div")`
  height: 60vh;
  overflow: scroll;
`;

export default function CurrencySelectionModal(props) {

  // Global state
  const globalState = useContext(GlobalContext);
  const { dispatch } = globalState;

  // Token list state
  const tokenListState = useContext(TokenListContext);
  
  const toggleModalVisibility = () => {
    dispatch({ type: 'TOGGLE_CURRENCY_SELECTION_MODAL' });
  };

  function handleCurrencyChange(newTokenIndex) {
    
    let infuraPrefix = "https://ipfs.infura.io:5001/api/v0/cat/";
    let newImageUrl = "";
    let hasImage = tokenListState.state.tokenList.tokens[newTokenIndex].hasOwnProperty("logoURI");
    let newSymbol = tokenListState.state.tokenList.tokens[newTokenIndex].symbol

    // Only display image if it exists
    if (hasImage) {
      newImageUrl = tokenListState.state.tokenList.tokens[newTokenIndex].logoURI;
      // Use Infura prefix if using IPFS
      if (newImageUrl.startsWith("ipfs://")) {
        newImageUrl = infuraPrefix + newImageUrl.substring(7);
      }
    }

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

  // Parses token list to table
  let tokensTable = "";
  if (tokenListState.state.tokenList.tokens[0].logoURI.startsWith('ipfs://')) {
    // Token image is served over IPFS
    tokensTable = tokenListState.state.tokenList.tokens.map((token, index) =>
      <PointerHover key={index} onClick={() => handleCurrencyChange(index)}>
        <td><img src={"https://ipfs.infura.io:5001/api/v0/cat/" + token.logoURI.substring(7)} width="25px" /></td>
        <td>{token.symbol}</td>
        <td>{token.name}</td>
      </PointerHover>
    );
  } else {
    // Token image is served over HTTP/HTTPS
    tokensTable = tokenListState.state.tokenList.tokens.map((token, index) =>
      <PointerHover key={index} onClick={() => handleCurrencyChange(index)}>
        <td><img src={token.logoURI} width="25px" /></td>
        <td>{token.symbol}</td>
        <td>{token.name}</td>
      </PointerHover>
    );
  }

  return (
    <>
      <Modal show={globalState.state.currencySelectionModal.isVisible} onHide={toggleModalVisibility}>
        <Modal.Header closeButton>
          <Modal.Title>Select currency</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          <LimitedHeightTable>
            <Table hover>
              <thead>
                <tr>
                  <th className="border-0"></th>
                  <th className="border-0">Symbol</th>
                 <th className="border-0">Name</th>
                </tr>
              </thead>
              <tbody>
                {tokensTable}
              </tbody>
            </Table>
          </LimitedHeightTable>
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={toggleModalVisibility}>
            Close
          </Button>
        </Modal.Footer>
      </Modal>
    </>
  );
}
