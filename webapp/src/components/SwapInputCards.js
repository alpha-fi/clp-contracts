import React, { useEffect, useState, useContext, useCallback } from "react";

import findCurrencyLogoUrl from "../services/find-currency-logo-url";
import { calcPriceFromIn, calcPriceFromOut, swapFromOut, incAllowance, getAllowance } from "../services/near-nep21-util";
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
  async function checkStatuses() {
    await delay(1200).then(async function() { // delay to wait for balances update
      if (inputs.state.swap.status == "isApproving") {

        // If approval is successful
        if (inputs.state.swap.previous < inputs.state.swap.in.allowance) {
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

        // inputs.state.swap.in.balance has not updated at this point, so we have to pull from tokenListState
        let newBalance = tokenListState.state.tokenList.tokens[inputs.state.swap.in.tokenIndex].balance;

        // If swap is successful
        if (inputs.state.swap.previous > newBalance) {
          // Reset amounts
          clearInputs();
          // Reset needsApproval
          dispatch({ type: 'UPDATE_SWAP_APPROVAL', payload: { 
            needsApproval: (inputs.state.swap.out.type === "NEP-21")
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
            message: "Your swap has failed."
          }});
          // Update status
          dispatch({ type: 'UPDATE_SWAP_STATUS', payload: { 
            status: "readyToSwap"
          }});
        }
      }
    });
  }

  // Updates swap status (usually readyToSwap, notReadyToSwap) and error message. Called by handleOutAmountChange() 
  function updateStatus(fromAmount, outAmount) {

    let newError;
    let newStatus = "notReadyToSwap";

    // Look for errors in order of most critical to least critical
    if (inputs.state.swap.in.tokenIndex === inputs.state.swap.out.tokenIndex) {
      newError = "Cannot swap to same currency.";
    } else if (!window.accountId) {
      newError = "Not signed in.";
    } else if (Number(fromAmount) > Number(inputs.state.swap.in.balance)) {
      newError = "Input exceeds balance.";
    } else if (Number(fromAmount) === 0 && Number(outAmount) !== 0) {
      newError = "Insufficient liquidity for trade."
    } else if (!isNonzeroNumber(outAmount)) {
      newError = "Enter a non-zero number.";
    } else {
      newError = null;
      newStatus = "readyToSwap"
    }

    dispatch({ type: 'UPDATE_SWAP_STATUS', payload: { status: newStatus, error: newError } });
  }

  // Handles updating button view and input information
  async function handleInTokenUpdate() {
    // Update image, symbol, address, tokenIndex, and type of selected currency
    await delay(1000).then(async function() { // delay to wait for balances update
      // Update image, symbol, address, tokenIndex, and type of selected currency
      let newToken = tokenListState.state.tokenList.tokens[inputs.state.swap.in.tokenIndex];
      dispatch({ type: 'UPDATE_IN_SELECTED_CURRENCY', 
        payload: { 
          logoUrl: findCurrencyLogoUrl(inputs.state.swap.in.tokenIndex, tokenListState.state.tokenList),
          symbol: newToken.symbol,
          type: newToken.type,
          tokenIndex: inputs.state.swap.in.tokenIndex,
          address: newToken.address,
          balance: newToken.balance
        }
      });
    })
    .then(function() {
      updateAllowance(inputs.state.swap.in)
    });

  }
  async function handleOutTokenUpdate() {
    // Update image, symbol, address, tokenIndex, and type of selected currency
    await delay(1000).then(async function() { // delay to wait for balances update
      let newToken = tokenListState.state.tokenList.tokens[inputs.state.swap.out.tokenIndex];
      dispatch({ type: 'UPDATE_OUT_SELECTED_CURRENCY', 
        payload: { 
          logoUrl: findCurrencyLogoUrl(inputs.state.swap.out.tokenIndex, tokenListState.state.tokenList),
          symbol: newToken.symbol,
          type: newToken.type,
          tokenIndex: inputs.state.swap.out.tokenIndex,
          address: newToken.address,
          balance: newToken.balance
        }
      });
    })
  }

  // Load icons and symbol for the current selected currency/token
  useEffect(() => {
    handleInTokenUpdate();
    handleOutTokenUpdate();
    checkStatuses();
  }, []);

  // Handle opening modal to select currency
  function handleCurrencySelectionModalFrom() {
    dispatch({ type: 'SET_CURRENCY_SELECTION_INPUT', payload: { input: "in" } });
  }
  function handleCurrencySelectionModalTo() { 
    dispatch({ type: 'SET_CURRENCY_SELECTION_INPUT', payload: { input: "out" } });
  }

  // Handle 'I want' amount changes
  async function handleOutAmountChange(amount) {

    // @TODO: check if amount <= allowance if NEP-21 is selected and temporarily set to false

    // If both inputs are valid non-zero numbers, set status to readyToSwap
    // Otherwise, set to notReadyToSwap
    let newIsValid = isNonzeroNumber(amount);

    // Update inputs state
    dispatch({ type: 'SET_OUT_AMOUNT', payload: {
      amount: amount,
      isValid: newIsValid,
    }});

    // Calculate the value of the other input box (only called when the user types)
<<<<<<< HEAD
    let updatedToken = { ...inputs.state.swap.out, amount: amount };
    let calculatedFromPrice = await calcPriceFromIn(updatedToken, inputs.state.swap.in)
=======
    let updatedToken = { ...inputs.state.swap.to, amount: amount };
    let calculatedFromPrice = await calcPriceFromOut(inputs.state.swap.from, updatedToken)
>>>>>>> master
    .then(function(result) {
      dispatch({ type: 'SET_IN_AMOUNT', payload: { amount: result, isValid: isNonzeroNumber(result) }});
      updateStatus(result, amount); // Update status and/or error message
    })
  }

  async function handleApprovalSubmission() {
    dispatch({ type: 'UPDATE_SWAP_STATUS', payload: { status: "isApproving", error: null, previous: inputs.state.swap.in.allowance } });
    await delay(1000).then(function() {
      dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
    })
    .then(async function() {
      let isApproved = await incAllowance(inputs.state.swap.in, inputs.state.swap.out);
    })
  }

  async function handleSwap() {
    try {
      dispatch({ type: 'UPDATE_SWAP_STATUS', payload: { status: "isSwapping", error: null, previous: inputs.state.swap.in.balance } });
      await delay(1000).then(function() {
        dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
      })
      .then(async function() {
        let swap = await swapFromOut(inputs.state.swap.in, inputs.state.swap.out);
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
  async function updateAllowance(tokenPayload) {
    await delay(500).then(async function() {
      if (tokenPayload.type == "NEP-21") {
        try {
          let allowance = await getAllowance(tokenPayload);
          dispatch({ type: 'UPDATE_IN_ALLOWANCE', payload: { allowance: allowance } });
        } catch (e) {
          console.error(e);
        }
      }
    });
  }

  // Move "To" input and currency to "From" and vice versa
  function switchInputs() {
    let oldInAmount = inputs.state.swap.in.amount;
    let oldOut = inputs.state.swap.out;
    dispatch({type: 'SWITCH_SWAP_INPUTS'});
    handleOutAmountChange(oldInAmount);
    updateAllowance(oldOut);
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
                value={inputs.state.swap.out.amount || ''}
                className="form-control border-0 bg-transparent"
                placeholder="0.0"
                onChange={(e) => handleOutAmountChange(e.target.value)}
              />
            </div>
          </Col>
          <Col xl={2} lg={3} md={4} sm={4} xs={12} className="d-flex flex-row-reverse align-items-center mr-2">
            <div className="text-right">

              {/* Show balance if any balance of selected token exists */}
              {(inputs.state.swap.out.balance && inputs.state.swap.out.balance !== 0)
                && <>
                    <small className="mr-3 text-secondary">
                      Balance: {Number(inputs.state.swap.out.balance).toFixed(2)}
                    </small>
                    <br/>
                  </>
              }

              <Button size="sm" variant="outline-secondary" className="mr-1" style={{'whiteSpace': 'nowrap'}} onClick={handleCurrencySelectionModalTo}>
                <img src={inputs.state.swap.out.logoUrl} width="15px" className="align-middle pb-1" />
                {' '}{inputs.state.swap.out.symbol}
                {' '}
                <BsCaretDownFill/>
              </Button>
              <br/>
              <small className="mr-3 text-secondary" style={{'whiteSpace': 'nowrap', 'fontSize': '60%'}}>
                {inputs.state.swap.out.type === "ERC-20"
                  ? <><FaEthereum/> ERC-20</>
                  : inputs.state.swap.out.type
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
                value={inputs.state.swap.in.amount}
                className="form-control border-0 bg-transparent"
                placeholder="0.0"
                readOnly
              />
            </div>
          </Col>
          <Col xl={2} lg={3} md={4} sm={4} xs={12} className="d-flex flex-row-reverse align-items-center mr-2">
            <div className="text-right">

              {/* Show balance if any balance of selected token exists */}
              {(inputs.state.swap.in.balance && inputs.state.swap.in.balance !== 0)
                && <>
                    <small className="mr-3 text-secondary">
                      Balance: {Number(inputs.state.swap.in.balance).toFixed(2)}
                    </small>
                    <br/>
                  </>
              }

              <Button size="sm" variant="outline-secondary" className="mr-1" style={{'whiteSpace': 'nowrap'}} onClick={handleCurrencySelectionModalFrom}>
                <img src={inputs.state.swap.in.logoUrl} width="15px" className="align-middle pb-1" />
                {' '}{inputs.state.swap.in.symbol}
                {' '}
                <BsCaretDownFill/>
              </Button>
              <br/>
              <small className="mr-3 text-secondary" style={{'whiteSpace': 'nowrap', 'fontSize': '60%'}}>
                {inputs.state.swap.in.type === "ERC-20"
                  ? <><FaEthereum/> ERC-20</>
                  : inputs.state.swap.in.type
                }
              </small>
            </div>
          </Col>
        </Row>
      </Theme>

      {(inputs.state.swap.in.allowance) &&
        <div className="text-right pr-3 text-secondary">
          <small>Current {inputs.state.swap.in.symbol} allowance: {inputs.state.swap.in.allowance}</small>
        </div>}

      <div className="text-right my-2 pr-2">
        {/* Display textual information before user swaps */}
        { ((inputs.state.swap.status === "readyToSwap")) &&
          <small className="text-secondary">
            You'll get at least <b className="text-black">{Number(inputs.state.swap.out.amount)}</b> {inputs.state.swap.out.symbol}{' '}
            for <b className="text-black">{Number(inputs.state.swap.in.amount)}</b> {inputs.state.swap.in.symbol}.
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
                      Approve {inputs.state.swap.in.symbol} allowance {
                        (inputs.state.swap.in.amount && inputs.state.swap.in.amount !== 0)
                          ? <>of {inputs.state.swap.in.amount}</>
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