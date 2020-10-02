import React, { useEffect, useState, useContext, useCallback } from "react";

import findCurrencyLogoUrl from "../services/find-currency-logo-url";
import { calcPriceFromIn, swapFromOut, incAllowance, getAllowance } from "../services/near-nep21-util";
import { isNonzeroNumber, delay } from "../utils"

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

// Test contract call function 
async function testContractCall( token1, token2) {
  console.log("testContractCall called. [token1.amount, token2.amount]")
  console.log([token1.amount, token2.amount])
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

  // Runs only in useEffect(). Looks for isApproving or isSwapping = true
  // and then updates state and shows notifications based on result
  function checkStatuses() {

    if (inputs.state.swap.status == "isApproving") {

      // If approval is successful
      if (inputs.state.swap.previous < inputs.state.swap.from.allowance) {
        // Notify user
        notification.dispatch({ type: 'SHOW_NOTIFICATION', payload: { 
          heading: "Approval complete",
          message: "Your NEP-21 token is now approved to swap."
        }});
        // Reset needsApproval
        dispatch({ type: 'UPDATE_SWAP_APPROVAL', payload: { 
          needsApproval: false
        }});
        // Update status
        dispatch({ type: 'UPDATE_SWAP_STATUS', payload: { 
          status: "readyToSwap"
        }});
      } else {
        // Approval is not successful
        notification.dispatch({ type: 'SHOW_NOTIFICATION', payload: { 
          heading: "Approval unsuccessful",
          message: "Please try again."
        }});
      }

    } else if (inputs.state.swap.status == "isSwapping") {

      // If swap is successful
      if (inputs.state.swap.previous < inputs.state.swap.from.balance) {
        // Reset amounts
        clearInputs();
        // Reset needsApproval
        dispatch({ type: 'UPDATE_SWAP_APPROVAL', payload: { 
          needsApproval: (inputs.state.swap.to.type === "NEP-21")
        }});
        // Notify user
        notification.dispatch({ type: 'SHOW_NOTIFICATION', payload: { 
          heading: "Swap complete",
          message: "Your swap has been submitted."
        }});
        // Update status
        dispatch({ type: 'UPDATE_SWAP_STATUS', payload: { 
          status: "notReadyToSwap"
        }});
      } else {
        // Swap is not successful
        notification.dispatch({ type: 'SHOW_NOTIFICATION', payload: { 
          heading: "Swap unsuccessful",
          message: "Your swap has been failed."
        }});
      }
    }
  }

  // Initializes allowance of from token
  async function initializeFromAllowance() {
    await delay(500).then(async function() {
      if (inputs.state.swap.from.type == "NEP-21") {
        try {
          let allowance = await getAllowance(inputs.state.swap.from);
          dispatch({ type: 'UPDATE_FROM_ALLOWANCE', payload: { allowance: allowance } });
        } catch (e) {
          console.error(e);
        }
      }
    });
  }

  // Updates swap status (usually readyToSwap, notReadyToSwap) and error message. Called by handleToAmountChange() 
  function updateStatus(fromAmount, toAmount) {

    let newError;
    let newStatus = "notReadyToSwap";

    // Look for errors in order of most critical to least critical
    if (inputs.state.swap.from.tokenIndex === inputs.state.swap.to.tokenIndex) {
      newError = "Cannot swap to same currency.";
    } else if (Number(fromAmount) > Number(inputs.state.swap.from.balance)) {
      newError = "Input exceeds balance.";
    } else if (Number(fromAmount) === 0 && Number(toAmount) !== 0) {
      newError = "Insufficient liquidity for trade."
    } else if (!isNonzeroNumber(toAmount)) {
      newError = "Enter a non-zero number.";
    } else {
      newError = null;
      newStatus = "readyToSwap"
    }

    dispatch({ type: 'UPDATE_SWAP_STATUS', payload: { status: newStatus, error: newError } });
  }

  // Handles updating button view and input information
  async function handleFromTokenUpdate() {
    // Update image, symbol, address, tokenIndex, and type of selected currency
    await delay(1000).then(async function() { // delay to wait for balances update
      // Update image, symbol, address, tokenIndex, and type of selected currency
      let newToken = tokenListState.state.tokenList.tokens[inputs.state.swap.from.tokenIndex];
      dispatch({ type: 'UPDATE_FROM_SELECTED_CURRENCY', 
        payload: { 
          logoUrl: findCurrencyLogoUrl(inputs.state.swap.from.tokenIndex, tokenListState.state.tokenList),
          symbol: newToken.symbol,
          type: newToken.type,
          tokenIndex: inputs.state.swap.from.tokenIndex,
          address: newToken.address,
          balance: newToken.balance
        }
      });
      initializeFromAllowance();
    })
    .then(async function() {
      if (inputs.state.swap.from.type == "NEP-21") {
        try {
          let allowance = await getAllowance(inputs.state.swap.from);
          dispatch({ type: 'UPDATE_FROM_ALLOWANCE', payload: { allowance: allowance } });
        } catch (e) {
          console.error(e);
        }
      }
    });

  }
  async function handleToTokenUpdate() {
    // Update image, symbol, address, tokenIndex, and type of selected currency
    await delay(1000).then(async function() { // delay to wait for balances update
      let newToken = tokenListState.state.tokenList.tokens[inputs.state.swap.to.tokenIndex];
      dispatch({ type: 'UPDATE_TO_SELECTED_CURRENCY', 
        payload: { 
          logoUrl: findCurrencyLogoUrl(inputs.state.swap.to.tokenIndex, tokenListState.state.tokenList),
          symbol: newToken.symbol,
          type: newToken.type,
          tokenIndex: inputs.state.swap.to.tokenIndex,
          address: newToken.address,
          balance: newToken.balance
        }
      });
    });
  }

  // Load icons and symbol for the current selected currency/token
  useEffect(() => {
    handleFromTokenUpdate();
    handleToTokenUpdate();
    checkStatuses();
  }, []);

  // Handle opening modal to select currency
  function handleCurrencySelectionModalFrom() {
    dispatch({ type: 'SET_CURRENCY_SELECTION_INPUT', payload: { input: "from" } });
  }
  function handleCurrencySelectionModalTo() { 
    dispatch({ type: 'SET_CURRENCY_SELECTION_INPUT', payload: { input: "to" } });
  }

  // Handle 'I want' amount changes
  async function handleToAmountChange(amount) {

    // If both inputs are valid non-zero numbers, set status to readyToSwap
    // Otherwise, set to notReadyToSwap
    let newIsValid = isNonzeroNumber(amount);

    // Update inputs state
    dispatch({ type: 'SET_TO_AMOUNT', payload: {
      amount: amount,
      isValid: newIsValid,
    }});

    // Calculate the value of the other input box (only called when the user types)
    let updatedToken = { ...inputs.state.swap.to, amount: amount };
    let calculatedFromPrice = await calcPriceFromIn(updatedToken, inputs.state.swap.from)
    .then(function(result) {
      dispatch({ type: 'SET_FROM_AMOUNT', payload: { amount: result, isValid: isNonzeroNumber(result) }});
      updateStatus(result, amount); // Update status and/or error message
    })
  }

  async function handleApprovalSubmission() {
    dispatch({ type: 'UPDATE_SWAP_STATUS', payload: { status: "isApproving", error: null, previous: inputs.state.swap.from.allowance } });
    await delay(1000).then(function() {
      dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
    })
    .then(async function() {
      let isApproved = await incAllowance(inputs.state.swap.from, inputs.state.swap.to);
    })
  }

  async function handleSwap() {
    try {
      dispatch({ type: 'UPDATE_SWAP_STATUS', payload: { status: "isSwapping", error: null, previous: inputs.state.swap.from.balance } });
      await delay(1000).then(function() {
        dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
      })
      .then(async function() {
        let swap = await swapFromOut(inputs.state.swap.from, inputs.state.swap.to);
      });
    } catch (e) {
      console.error(e);
    }
  }

  // Clear inputs
  function clearInputs() {
    dispatch({type: 'CLEAR_SWAP_INPUTS'});
    dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
  }

  // Updates allowance of from token
  async function updateFromAllowance(token) {
    await delay(500).then(async function() {
      if (token.type == "NEP-21") {
        try {
          let allowance = await getAllowance(token);
          dispatch({ type: 'UPDATE_FROM_ALLOWANCE', payload: { allowance: allowance } });
        } catch (e) {
          console.error(e);
        }
      }
    });
  }

  // Move "To" input and currency to "From" and vice versa
  function switchInputs() {
    let oldFromAmount = inputs.state.swap.from.amount;
    let oldTo = inputs.state.swap.to;
    dispatch({type: 'SWITCH_SWAP_INPUTS'});
    handleToAmountChange(oldFromAmount);
    updateFromAllowance(oldTo);
  }

  return (
    <>
      
      <Theme className="py-2">
        <label className="ml-4 mb-1 mt-0">
          <small className="text-secondary">I want:</small>
        </label>
        <Row className="px-2">
          <Col>
            <div className="input-group-lg mb-1">
              {/* TO INPUT */}
              <input
                type="text"
                value={inputs.state.swap.to.amount || ''}
                className="form-control border-0 bg-transparent"
                placeholder="0.0"
                onChange={(e) => handleToAmountChange(e.target.value)}
              />
            </div>
          </Col>
          <Col xl={2} lg={3} md={4} sm={4} xs={12} className="d-flex flex-row-reverse align-items-center mr-2">
            <div className="text-right">

              {/* Show balance if any balance of selected token exists */}
              {(inputs.state.swap.to.balance && inputs.state.swap.to.balance !== 0)
                && <>
                    <small className="mr-3 text-secondary">
                      Balance: {Number(inputs.state.swap.to.balance).toFixed(2)}
                    </small>
                    <br/>
                  </>
              }

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

      

      <div className="text-center my-2">
        <span onClick={switchInputs} style={{ cursor: 'pointer' }}><BsArrowUpDown/></span>
      </div>

      <Theme className="py-2">
        <label className="ml-4 mb-1 mt-0">
          <small className="text-secondary">I'll provide:</small>
        </label>
        <Row className="px-2">
          <Col>
            <div className="input-group-lg mb-1">
              {/* FROM INPUT */}
              <input
                type="text"
                value={inputs.state.swap.from.amount}
                className="form-control border-0 bg-transparent"
                placeholder="0.0"
                readOnly
              />
            </div>
          </Col>
          <Col xl={2} lg={3} md={4} sm={4} xs={12} className="d-flex flex-row-reverse align-items-center mr-2">
            <div className="text-right">

              {/* Show balance if any balance of selected token exists */}
              {(inputs.state.swap.from.balance && inputs.state.swap.from.balance !== 0)
                && <>
                    <small className="mr-3 text-secondary">
                      Balance: {Number(inputs.state.swap.from.balance).toFixed(2)}
                    </small>
                    <br/>
                  </>
              }

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

      {(inputs.state.swap.from.allowance) &&
        <div className="text-right pr-3 text-secondary">
          <small>Current {inputs.state.swap.from.symbol} allowance: {inputs.state.swap.from.allowance}</small>
        </div>}

      <div className="text-right my-2 pr-2">
        {/* Display textual information before user swaps */}
        { ((inputs.state.swap.status === "readyToSwap")) &&
          <small className="text-secondary">
            You'll get at least <b className="text-black">{Number(inputs.state.swap.to.amount)}</b> {inputs.state.swap.to.symbol}{' '}
            for <b className="text-black">{Number(inputs.state.swap.from.amount)}</b> {inputs.state.swap.from.symbol}.
          </small>
        }
        
        <small className="text-danger mr-1">{inputs.state.swap.error}</small>

        {/* Clear button and clippage */}
        {((inputs.state.swap.status !== "isApproving") && (inputs.state.swap.status !== "isSwapping"))
          &&  <>
                <small className="mx-2 text-secondary">Slippage: 1%</small>
                <Button size="sm" variant="warning" onClick={clearInputs} className="ml-2">Clear</Button>
              </>
        }
      </div>

      {/* Display approve button if NEP-21 -> ____ swap */}
      {(inputs.state.swap.needsApproval)
         && <>
              <small className="text-secondary">Step 1: </small>
              <Button variant="warning" block
                disabled={(inputs.state.swap.status !== "readyToSwap" || inputs.state.swap.error)}
                onClick={handleApprovalSubmission}
              >
                {(inputs.state.swap.status !== "isApproving")
                  ? <>
                      Approve {inputs.state.swap.from.symbol} allowance {
                        (inputs.state.swap.from.amount && inputs.state.swap.from.amount !== 0)
                          ? <>of {inputs.state.swap.from.amount}</>
                          : ""
                        }
                    </>
                  : "Approving..."
                }
              </Button>
            </>}

      {/* Enable submission only if inputs are valid */}
      {(inputs.state.swap.status !== "isApproving") &&
        <>
          {(inputs.state.swap.needsApproval == true) && <small className="text-secondary">Step 2: </small>}
          <Button variant="warning" block
            disabled={((inputs.state.swap.status !== "readyToSwap") || inputs.state.swap.needsApproval)}
            onClick={handleSwap}
          >
            {(inputs.state.swap.status !== "isSwapping")
              ? "Swap"
              : "Swapping..."
            }
          </Button>
        </>
      }
    </>
  );
}