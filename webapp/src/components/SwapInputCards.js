import React, { useEffect, useState, useContext } from "react";

import findCurrencyLogoUrl from "../services/find-currency-logo-url";
import { calcPriceFromIn } from "../services/near-nep21-util";

import { InputsContext } from "../contexts/InputsContext";
import { TokenListContext } from "../contexts/TokenListContext";

import Row from 'react-bootstrap/Row';
import Col from 'react-bootstrap/Col';
import Button from 'react-bootstrap/Button';

import { BsCaretDownFill } from "react-icons/bs";
import { FaEthereum } from "react-icons/fa";
import { BsArrowUpDown } from "react-icons/bs";

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

export default function SwapInputCards(props) {

  // Global state
  const inputs = useContext(InputsContext);
  const { dispatch } = inputs;

  // Token list state (used to populate button with token logo and symbol)
  const tokenListState = useContext(TokenListContext);

  // Local state (amount values; needed for updating directly in inputs)
  const [fromAmount, setFromAmount] = useState("");
  const [toAmount, setToAmount] = useState("");

  // Handles updating button view and input information
  function handleFromTokenUpdate() {
    // Find URL of token logo, symbol, type, and address of FROM input
    let fromImageUrl = findCurrencyLogoUrl(inputs.state.swap.from.tokenIndex, tokenListState.state.tokenList);
    let fromSymbol = tokenListState.state.tokenList.tokens[inputs.state.swap.from.tokenIndex].symbol;
    let fromType = tokenListState.state.tokenList.tokens[inputs.state.swap.from.tokenIndex].type;
    let fromAddress = tokenListState.state.tokenList.tokens[inputs.state.swap.from.tokenIndex].address;
    // Update image, symbol, and type of selected currency
    dispatch({ type: 'UPDATE_FROM_SELECTED_CURRENCY', 
      payload: { logoUrl: fromImageUrl, symbol: fromSymbol, type: fromType, isValid: false, address: fromAddress }
    });
  }
  function handleToTokenUpdate() {
    // Find URL of token logo, symbol, type, and address of TO input
    let toImageUrl = findCurrencyLogoUrl(inputs.state.swap.to.tokenIndex, tokenListState.state.tokenList);
    let toSymbol = tokenListState.state.tokenList.tokens[inputs.state.swap.to.tokenIndex].symbol;
    let toType = tokenListState.state.tokenList.tokens[inputs.state.swap.to.tokenIndex].type;
    let toAddress = tokenListState.state.tokenList.tokens[inputs.state.swap.to.tokenIndex].address;
    // Update image, symbol, and type of selected currency
    dispatch({ type: 'UPDATE_TO_SELECTED_CURRENCY', 
      payload: { logoUrl: toImageUrl, symbol: toSymbol, type: toType, isValid: false, address: toAddress }
    });
  }

  // Load icons and symbol for the current selected currency/token
  useEffect(() => {
    handleFromTokenUpdate();
    handleToTokenUpdate();
  }, []);

  // Handle opening modal to select currency
  function handleCurrencySelectionModalFrom() {
    dispatch({ type: 'SET_CURRENCY_SELECTION_INPUT', payload: { input: "from" } });
  }
  function handleCurrencySelectionModalTo() { 
    dispatch({ type: 'SET_CURRENCY_SELECTION_INPUT', payload: { input: "to" } });
  }

  // Handle amount changes
  async function handleFromAmountChange(event) {
    event.persist();
    setFromAmount(event.target.value);
    
    // Logs result of calcPriceFromIn()
    const res = await calcPriceFromIn(inputs.state.swap.from, inputs.state.swap.to);
    console.log(res);

    dispatch({ type: 'SET_FROM_AMOUNT', payload: { amount: event.target.value, isValid: true } });
  }
  function handleToAmountChange(event) {
    setToAmount(event.target.value);
    dispatch({ type: 'SET_TO_AMOUNT', payload: { amount: event.target.value, isValid: true } });
  }

  return (
    <>
      <Theme className="py-2">
        <label className="ml-4 mb-1 mt-0">
          <small className="text-secondary">From</small>
        </label>
        <Row className="px-2">
          <Col>
            <div className="input-group-lg mb-1">
              <input type="text" value={fromAmount} className="form-control border-0 bg-transparent" placeholder="0.0" onChange={handleFromAmountChange}/>
            </div>
          </Col>
          <Col xl={2} lg={3} md={4} sm={4} xs={12} className="d-flex flex-row-reverse align-items-center mr-2">
            <div className="text-right">
              <Button size="sm" variant="outline-secondary" className="mr-1" style={{'whiteSpace': 'nowrap'}} onClick={handleCurrencySelectionModalFrom}>
                <img src={inputs.state.swap.from.logoUrl} width="15px" className="align-middle pb-1" />
                {' '}{inputs.state.swap.from.symbol}
                {' '}
                <BsCaretDownFill/>
              </Button>
              <br/>
              <small className="mr-3 text-secondary" style={{'whiteSpace': 'nowrap', 'fontSize': '60%'}}>
                {inputs.state.swap.from.type === "ERC-20"
                  ? <><FaEthereum/> ERC-20</>
                  : inputs.state.swap.from.type
                }
              </small>
            </div>
          </Col>
        </Row>
      </Theme>
      <div className="text-center my-2">
        <BsArrowUpDown/>
      </div>
      <Theme className="py-2">
        <label className="ml-4 mb-1 mt-0">
          <small className="text-secondary">To</small>
        </label>
        <Row className="px-2">
          <Col>
            <div className="input-group-lg mb-1">
              <input type="text" value={toAmount} className="form-control border-0 bg-transparent" placeholder="0.0" onChange={handleToAmountChange}/>
            </div>
          </Col>
          <Col xl={2} lg={3} md={4} sm={4} xs={12} className="d-flex flex-row-reverse align-items-center mr-2">
            <div className="text-right">
              <Button size="sm" variant="outline-secondary" className="mr-1" style={{'whiteSpace': 'nowrap'}} onClick={handleCurrencySelectionModalTo}>
                <img src={inputs.state.swap.to.logoUrl} width="15px" className="align-middle pb-1" />
                {' '}{inputs.state.swap.to.symbol}
                {' '}
                <BsCaretDownFill/>
              </Button>
              <br/>
              <small className="mr-3 text-secondary" style={{'whiteSpace': 'nowrap', 'fontSize': '60%'}}>
                {inputs.state.swap.to.type === "ERC-20"
                  ? <><FaEthereum/> ERC-20</>
                  : inputs.state.swap.to.type
                }
              </small>
            </div>
          </Col>
        </Row>
      </Theme>
    </>
  );
}
