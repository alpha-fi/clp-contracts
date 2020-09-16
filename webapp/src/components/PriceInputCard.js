import React, { useEffect, useState, useContext } from "react";

import { GlobalContext } from "../contexts/GlobalContext";
import { TokenListContext } from "../contexts/TokenListContext";

import Row from 'react-bootstrap/Row';
import Col from 'react-bootstrap/Col';
import Button from 'react-bootstrap/Button';

import { BsCaretDownFill } from "react-icons/bs";

// props.name must be one of:
// (Swap): from, to
// (Pool): input1, input2
export default function PriceInputCard(props) {

  // Global state
  const globalState = useContext(GlobalContext);
  const { dispatch } = globalState;

  // Token list state (used to populate button with token logo and symbol)
  const tokenListState = useContext(TokenListContext);

  // Handles updating button view and global state
  function handleTokenButtonUpdate() {

    let infuraPrefix = "https://ipfs.infura.io:5001/api/v0/cat/";
    let imageUrl = "";
    let hasImage = tokenListState.state.tokenList.tokens[props.tokenIndex].hasOwnProperty("logoURI");

    // Only load image URL if it exists
    if (hasImage) {
      imageUrl = tokenListState.state.tokenList.tokens[props.tokenIndex].logoURI;
      // Use Infura prefix if using IPFS
      if (imageUrl.startsWith("ipfs://")) {
        imageUrl = infuraPrefix + imageUrl.substring(7);
      }
    }

    // Update image and symbol of selected currency
    switch (props.name) {
      case 'from':
        dispatch({ type: 'UPDATE_FROM_SELECTED_CURRENCY', 
          payload: { logoUrl: imageUrl, symbol: tokenListState.state.tokenList.tokens[props.tokenIndex].symbol }
        });
        break;
      case 'to':
        dispatch({ type: 'UPDATE_TO_SELECTED_CURRENCY', 
          payload: { logoUrl: imageUrl, symbol: tokenListState.state.tokenList.tokens[props.tokenIndex].symbol }
        });
        break;
      case 'input1':
        dispatch({ type: 'UPDATE_INPUT1_SELECTED_CURRENCY', 
          payload: { logoUrl: imageUrl, symbol: tokenListState.state.tokenList.tokens[props.tokenIndex].symbol }
        });
        break;
      case 'input2':
        dispatch({ type: 'UPDATE_INPUT2_SELECTED_CURRENCY', 
          payload: { logoUrl: imageUrl, symbol: tokenListState.state.tokenList.tokens[props.tokenIndex].symbol }
        });
    }
  }

  // Load icons and symbol for the current selected currency/token
  useEffect(() => {
    handleTokenButtonUpdate();
  }, []);

  // Handle opening modal to select currency
  function handleCurrencySelectionModal() { 
    dispatch({ type: 'SET_CURRENCY_SELECTION_INPUT', payload: { input: props.name } });
  }

  function handleAmountChange(event) {
    switch (props.name) {
      case 'from':
        dispatch({ type: 'SET_FROM_AMOUNT', payload: { amount: event.target.value } });
        break;
      case 'to':
        dispatch({ type: 'SET_TO_AMOUNT', payload: { amount: event.target.value } });
        break;
      case 'input1':
        dispatch({ type: 'SET_INPUT1_AMOUNT', payload: { amount: event.target.value } });
        break;
      case 'input2':
        dispatch({ type: 'SET_INPUT2_AMOUNT', payload: { amount: event.target.value } });
    }
  }

  return (
    <>
      <div className="border py-3 bg-white" style={{ 'borderRadius': '15px', 'boxShadow': '0px 1px 0px 0px rgba(9,30,66,.25)' }}>
        <label className="ml-4 mb-1 mt-0">
          <small className="text-secondary">{props.label}</small>
        </label>
        <Row className="px-2">
          <Col>
            <div className="input-group-lg mb-1">
              <input type="text" className="form-control border-0" placeholder="0.0" onChange={handleAmountChange}/>
            </div>
          </Col>
          <Col xl={3} lg={5} sm={6} className="d-flex flex-row-reverse align-items-center mr-2">
            <Button size="sm" variant="outline-secondary" onClick={handleCurrencySelectionModal}>
              <span className="align-middle">
                <img src={props.logoUrl} width="15px" className="align-middle pb-1" />
                {' '}{props.symbol}
                {' '}
                <BsCaretDownFill/>
              </span>
            </Button>
          </Col>
        </Row>
      </div>
    </>
  );
}
