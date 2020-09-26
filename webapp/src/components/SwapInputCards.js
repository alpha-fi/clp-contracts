import React, { useEffect, useState, useContext } from "react";

import findCurrencyLogoUrl from "../services/find-currency-logo-url";
import { calcPriceFromIn } from "../services/near-nep21-util";

import { InputsContext } from "../contexts/InputsContext";
import { TokenListContext } from "../contexts/TokenListContext";
import { NotificationContext } from "../contexts/NotificationContext";

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

function isNonzeroNumber(num) {
  return (!isNaN(num) && (num > 0));
}
// Test contract call function 
async function testContractCall( token1, token2) {
  return await window.nep21.get_balance({ owner_id: window.walletConnection.getAccountId() });
}

export default function SwapInputCards(props) {

  // Global state
  const inputs = useContext(InputsContext);
  const { dispatch } = inputs;

  // Token list state (used to populate button with token logo and symbol)
  const tokenListState = useContext(TokenListContext);

  // Notification state
  const notification = useContext(NotificationContext);

  // Local state (amount values; needed for updating directly in inputs)
  const [fromAmount, setFromAmount] = useState("");
  const [toAmount, setToAmount] = useState("");
  // const [isReadyToSwap, setIsReadyToSwap] = useState();

  // Handles updating button view and input information
  function handleFromTokenUpdate() {
    // Update image, symbol, address, tokenIndex, and type of selected currency
    dispatch({ type: 'UPDATE_FROM_SELECTED_CURRENCY', 
      payload: { 
        logoUrl: findCurrencyLogoUrl(inputs.state.swap.from.tokenIndex, tokenListState.state.tokenList), 
        symbol: tokenListState.state.tokenList.tokens[inputs.state.swap.from.tokenIndex].symbol, 
        type: tokenListState.state.tokenList.tokens[inputs.state.swap.from.tokenIndex].type, 
        tokenIndex: inputs.state.swap.from.tokenIndex,
        address: tokenListState.state.tokenList.tokens[inputs.state.swap.from.tokenIndex].address }
    });
  }
  function handleToTokenUpdate() {
    // Update image, symbol, address, tokenIndex, and type of selected currency
    dispatch({ type: 'UPDATE_TO_SELECTED_CURRENCY', 
      payload: { 
        logoUrl: findCurrencyLogoUrl(inputs.state.swap.to.tokenIndex, tokenListState.state.tokenList), 
        symbol: tokenListState.state.tokenList.tokens[inputs.state.swap.to.tokenIndex].symbol, 
        type: tokenListState.state.tokenList.tokens[inputs.state.swap.to.tokenIndex].type, 
        tokenIndex: inputs.state.swap.to.tokenIndex,
        address: tokenListState.state.tokenList.tokens[inputs.state.swap.to.tokenIndex].address }
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

  // Handle 'From' amount changes
  async function handleFromAmountChange(event) {
    event.persist();                   // Persist event because this is an async function
    setFromAmount(event.target.value); // Update local state

    // If both inputs are valid non-zero numbers, set status to readyToSwap
    // Otherwise, set to notReadyToSwap
    let newStatus = ((
      isNonzeroNumber(event.target.amount) && inputs.state.swap.to.isValid
    ) ? "readyToSwap" : "notReadyToSwap" );

    // Update inputs state
    dispatch({ type: 'SET_FROM_AMOUNT', payload: {
      amount: event.target.value,
      isValid: isNonzeroNumber(event.target.amount),
      status: newStatus // possible values: notReadyToSwap, readyToSwap, swapping
    }});

    // Calculate the value of the other input box
    let updatedToken = { ...inputs.state.swap.from, amount: event.target.value };
    let calculatedToPrice = await calcPriceFromIn(updatedToken, inputs.state.swap.to)
    .then(function(result) {
      if (Number(result) !== 0)
        setToAmount(result); // Update other input box with calculated price
      // Update inputs state
      dispatch({ type: 'SET_TO_AMOUNT', payload: {
        amount: result,
        isValid: isNonzeroNumber(result),
        status: inputs.state.swap.status
      }});
    });
  }

  // Handle 'To' amount changes
  async function handleToAmountChange(event) {
    event.persist();                 // Persist event because this is an async function
    setToAmount(event.target.value); // Update local state

    // If both inputs are valid non-zero numbers, set status to readyToSwap
    // Otherwise, set to notReadyToSwap
    let newStatus = ((
      isNonzeroNumber(event.target.amount) && inputs.state.swap.from.isValid
    ) ? "readyToSwap" : "notReadyToSwap" );

    // Update inputs state
    dispatch({ type: 'SET_TO_AMOUNT', payload: {
      amount: event.target.value,
      isValid: isNonzeroNumber(event.target.amount),
      status: newStatus // possible values: notReadyToSwap, readyToSwap
    }});

    // Calculate the value of the other input box
    let updatedToken = { ...inputs.state.swap.to, amount: event.target.value };
    let calculatedToPrice = await calcPriceFromIn(inputs.state.swap.from, updatedToken)
    .then(function(result) {
      if (Number(result) !== 0)
        setFromAmount(result); // Update other input box with calculated price
      // Update inputs state
      dispatch({ type: 'SET_FROM_AMOUNT', payload: {
        amount: result,
        isValid: isNonzeroNumber(result),
        status: inputs.state.swap.status
      }});
    });
  }

  async function handleApprovalSubmission() {
    // Approval function here
    let isApproved = true;

    dispatch({ type: 'UPDATE_SWAP_APPROVAL', payload: { needsApproval: false }});
    notification.dispatch({ type: 'SHOW_NOTIFICATION', payload: { 
      heading: "Token swap approved",
      message: "You can now make a swap."
    }});
  }

  async function handleSwap() {
    try {
      let swap = await testContractCall(inputs.state.swap.from, inputs.state.swap.to)
        .then(function(result) {
          // Reset amounts in local state
          setFromAmount("");
          setToAmount("");
          // Reset amounts in input state
          dispatch({ type: 'SET_TO_AMOUNT', payload: { amount: "", isValid: false, status: "notReadyToSwap" }});
          // Reset needsApproval
          dispatch({ type: 'UPDATE_SWAP_APPROVAL', payload: { 
            needsApproval: (inputs.state.swap.to.type === "NEP-21" && inputs.state.swap.from.type === "NEP-21")
          }});
          // Notify user
          notification.dispatch({ type: 'SHOW_NOTIFICATION', payload: { 
            heading: "Swap complete",
            message: "Your swap has been submitted."
          }});
      });
    } catch (e) {
      console.error(e);
    }
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
      <br/>
      {/* Display approve button if NEP-21 <> NEP-21 swap */}
      {(inputs.state.swap.needsApproval)
        && <Button variant="warning" block
             disabled={(inputs.state.swap.status !== "readyToSwap")}
             onClick={handleApprovalSubmission}
           >
             Approve tokens (0.04 NEAR)
           </Button>}

      {/* Enable submission only if inputs are valid */}
      <Button variant="warning" block
        disabled={((inputs.state.swap.status !== "readyToSwap") || inputs.state.swap.needsApproval)}
        onClick={handleSwap}
      >
        Swap
      </Button>
    </>
  );
}