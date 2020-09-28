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

  function checkStatuses() {

    if (inputs.state.swap.status == "isApproving") {

      // If approval is successful
      if (true) {
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
      if (true) {
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

  // Updates allowance of from token
  async function initializeFromAllowance() {
    await delay(1000).then(async function() {
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

  // Handles updating button view and input information
  function handleFromTokenUpdate() {
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
  }
  function handleToTokenUpdate() {
    // Update image, symbol, address, tokenIndex, and type of selected currency
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

  // Handle 'From' amount changes
  async function handleFromAmountChange(amount, isShallowUpdate) {

    // If both inputs are valid non-zero numbers, set status to readyToSwap
    // Otherwise, set to notReadyToSwap
    let newIsValid = isNonzeroNumber(amount);
    let newStatus = (newIsValid ? "readyToSwap" : "notReadyToSwap" );

    // Update inputs state
    dispatch({ type: 'SET_FROM_AMOUNT', payload: {
      amount: amount,
      isValid: newIsValid,
      status: newStatus // possible values: notReadyToSwap, readyToSwap
    }});

    // Calculate the value of the other input box (only called when the user types)
    if (!isShallowUpdate) {
      let updatedToken = { ...inputs.state.swap.from, amount: amount };
      let calculatedToPrice = await calcPriceFromIn(updatedToken, inputs.state.swap.to)
      .then(function(result) {
        handleToAmountChange(result, true); // Shallow update the other input box
      });
    }
  }

  // Handle 'To' amount changes
  async function handleToAmountChange(amount, isShallowUpdate) {

    // If both inputs are valid non-zero numbers, set status to readyToSwap
    // Otherwise, set to notReadyToSwap
    let newIsValid = isNonzeroNumber(amount);
    let newStatus = (newIsValid ? "readyToSwap" : "notReadyToSwap" );

    // Update inputs state
    dispatch({ type: 'SET_TO_AMOUNT', payload: {
      amount: amount,
      isValid: newIsValid,
      status: newStatus // possible values: notReadyToSwap, readyToSwap
    }});

    // Calculate the value of the other input box (only called when the user types)
    if (!isShallowUpdate) {
      let updatedToken = { ...inputs.state.swap.to, amount: amount };
      let calculatedToPrice = await calcPriceFromIn(inputs.state.swap.from, updatedToken)
      .then(function(result) {
        handleFromAmountChange(result, true); // Shallow update the other input box
      });
    }
  }

  async function handleApprovalSubmission() {
    dispatch({ type: 'UPDATE_SWAP_STATUS', payload: { status: "isApproving" } });
    await delay(1000).then(function() {
      dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
    })
    .then(async function() {
      let isApproved = await incAllowance(inputs.state.swap.from, inputs.state.swap.to);
    })
  }

  async function handleSwap() {
    try {
      dispatch({ type: 'UPDATE_SWAP_STATUS', payload: { status: "isSwapping" } });
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

  async function updateFromAllowance() {

  }

  // Clear inputs
  function clearInputs() {
    dispatch({type: 'CLEAR_SWAP_INPUTS'});
    dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
  }

  // Move "To" input and currency to "From" and vice versa
  function switchInputs() {
    let oldToAmount = inputs.state.swap.to.amount;
    dispatch({type: 'SWITCH_SWAP_INPUTS'});
    handleFromAmountChange(oldToAmount, false);
    dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
    updateFromAllowance();
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
              {/* FROM INPUT */}
              <input
                type="text"
                value={inputs.state.swap.from.amount || ''}
                className="form-control border-0 bg-transparent"
                placeholder="0.0"
                onChange={(e) => handleFromAmountChange(e.target.value, false)}
              />
            </div>
          </Col>
          <Col xl={2} lg={3} md={4} sm={4} xs={12} className="d-flex flex-row-reverse align-items-center mr-2">
            <div className="text-right">

              {/* Show max balance if any balance of selected token exists */}
              {(inputs.state.swap.from.balance && inputs.state.swap.from.balance !== 0)
                && <>
                    <small className="mr-3 text-secondary">
                      Max:{' '}
                      <u
                        onClick={(e) => handleFromAmountChange(
                          (inputs.state.swap.from.balance - 0.06).toString(), // subtract 0.06 for call
                          false) // not a shallow update
                        }
                        style={{ cursor: 'pointer' }}
                      >
                        {Number(inputs.state.swap.from.balance).toFixed(2)}
                      </u>
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
        <div className="text-right pr-3">
          <small>Current {inputs.state.swap.from.symbol} allowance: {inputs.state.swap.from.allowance}</small>
        </div>}

      <div className="text-center my-2">
        <span onClick={switchInputs} style={{ cursor: 'pointer' }}><BsArrowUpDown/></span>
      </div>

      <Theme className="py-2">
        <label className="ml-4 mb-1 mt-0">
          <small className="text-secondary">To</small>
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
                onChange={(e) => handleToAmountChange(e.target.value, false)}
              />
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

      <div className="text-right my-2 pr-3">
        {/* Display textual information before user swaps */}
        { ((inputs.state.swap.status === "readyToSwap")) &&
          <small className="text-secondary">
            You'll get at least <b className="text-black">{Number(inputs.state.swap.to.amount).toFixed(2)}</b> {inputs.state.swap.to.symbol}{' '}
            for <b className="text-black">{Number(inputs.state.swap.from.amount).toFixed(2)}</b> {inputs.state.swap.from.symbol}.
          </small>
        }
        <small className="mx-2 text-secondary">Slippage: 1%</small>
        {/* Clear button */}
        {((inputs.state.swap.status !== "isApproving") && (inputs.state.swap.status !== "isSwapping"))
          && <Button size="sm" variant="warning" onClick={clearInputs} className="ml-2">Clear</Button>}
      </div>

      {/* Display approve button if NEP-21 -> ____ swap */}
      {(inputs.state.swap.needsApproval)
         && <Button variant="warning" block
              disabled={(inputs.state.swap.status !== "readyToSwap")}
              onClick={handleApprovalSubmission}
            >
              {(inputs.state.swap.status != "isApproving")
                ? <>Approve NEP-21 allowance <small>+0.06 NEAR</small></>
                : "Approving..."
              }
            </Button>}
      
      {/*<Button variant="primary" block
       onClick={handleApprovalSubmission}
     >Approve tokens (test button)</Button>*/}

      {/* Enable submission only if inputs are valid */}
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
  );
}