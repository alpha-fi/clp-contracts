import React, { useEffect, useState, useContext } from "react";

import findCurrencyLogoUrl from "../services/find-currency-logo-url";

import { GlobalContext } from "../contexts/GlobalContext";
import { TokenListContext } from "../contexts/TokenListContext";

import Row from 'react-bootstrap/Row';
import Col from 'react-bootstrap/Col';
import Button from 'react-bootstrap/Button';

import { BsCaretDownFill } from "react-icons/bs";
import { FaEthereum } from "react-icons/fa"; 

import styled from "@emotion/styled";
const Theme = styled("div")`
  background: ${props => props.theme.cardBackground};
  color: ${props => props.theme.body};
  border: 1px solid ${props => props.theme.cardBorder};
  border-radius: 20px;
  box-shadow: 0px 1px 0px 0px ${props => props.theme.cardShadow};
  .form-control:focus {
    color: ${props => props.theme.textInput};
  }
`;

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

    // Find URL of token logo
    let newImageUrl = findCurrencyLogoUrl(props.tokenIndex, tokenListState.state.tokenList);
    let newSymbol = tokenListState.state.tokenList.tokens[props.tokenIndex].symbol;

    // Update image and symbol of selected currency
    switch (props.name) {
      case 'from':
        dispatch({ type: 'UPDATE_FROM_SELECTED_CURRENCY', 
          payload: { logoUrl: newImageUrl, symbol: newSymbol }
        });
        break;
      case 'to':
        dispatch({ type: 'UPDATE_TO_SELECTED_CURRENCY', 
          payload: { logoUrl: newImageUrl, symbol: newSymbol }
        });
        break;
      case 'input1':
        dispatch({ type: 'UPDATE_INPUT1_SELECTED_CURRENCY', 
          payload: { logoUrl: newImageUrl, symbol: newSymbol }
        });
        break;
      case 'input2':
        // Defaults to NEAR; no need to set new currency
        dispatch({ type: 'UPDATE_INPUT2_SELECTED_CURRENCY', 
          payload: { logoUrl: newImageUrl }
        });
        break;
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
      <Theme className="py-3">
        <label className="ml-4 mb-1 mt-0">
          <small className="text-secondary">{props.label}</small>
        </label>
        <Row className="px-2">
          <Col>
            <div className="input-group-lg mb-1">
              <input type="text" className="form-control border-0 bg-transparent" placeholder="0.0" onChange={handleAmountChange}/>
            </div>
          </Col>
          <Col xl={3} lg={5} sm={6} className="d-flex flex-row-reverse align-items-center mr-2">
            <div className="text-right">
              <Button size="sm" variant="outline-secondary" className="mr-1" style={{'whiteSpace': 'nowrap'}} onClick={handleCurrencySelectionModal}>
                <img src={props.logoUrl} width="15px" className="align-middle pb-1" />
                {' '}{props.symbol}
                {' '}
                <BsCaretDownFill/>
              </Button>
              <Button size="sm" variant="outline-secondary" className="mr-1" style={{'whiteSpace': 'nowrap', 'fontSize': '60%'}}>
                <FaEthereum/>{' '}
                ERC-20
                {' '}
                <BsCaretDownFill/>
              </Button>
            </div>
          </Col>
        </Row>
      </Theme>
    </>
  );
}
