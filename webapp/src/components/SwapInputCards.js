import React, { useEffect, useState, useContext, useCallback } from "react";

import findCurrencyLogoUrl from "../services/find-currency-logo-url";
import { calcPriceFromIn, swapFromOut, incAllowance } from "../services/near-nep21-util";

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

  // Local state (amount values; needed for updating directly in inputs)
  const [fromAmount, setFromAmount] = useState(inputs.state.swap.from.amount);
  const [toAmount, setToAmount] = useState(inputs.state.swap.to.amount);

  let fromBalance = tokenListState.state.tokenList.tokens[inputs.state.swap.from.tokenIndex].balance;

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
    dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
  }
  function handleCurrencySelectionModalTo() { 
    dispatch({ type: 'SET_CURRENCY_SELECTION_INPUT', payload: { input: "to" } });
    dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
  }

  // Handle 'From' amount changes
  async function handleFromAmountChange(amount, isShallowUpdate) {
    console.log("handleFromAmountChange")
    console.log(["amount", amount])
    console.log(["isShallowUpdate", isShallowUpdate])
    if (amount !== 0) {
      setFromAmount(amount); // Update local state
    } else {
      setFromAmount("");
    }

    // If both inputs are valid non-zero numbers, set status to readyToSwap
    // Otherwise, set to notReadyToSwap
    let newIsValid;
    if (!isShallowUpdate) {
      // Not a shallow update
      newIsValid = isNonzeroNumber(amount) && (amount <= tokenListState.state.tokenList.tokens[inputs.state.swap.from.tokenIndex].balance);
    } else {
      // Shallow update (called by the other input)
      newIsValid = isNonzeroNumber(amount);
    }
    let newStatus = (newIsValid ? "readyToSwap" : "notReadyToSwap" );
    // console.log(["newIsValid", newIsValid]);
    // console.log(["newStatus", newStatus]);

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
    if (amount !== 0) {
      setToAmount(amount); // Update local state
    } else {
      setToAmount("");
    }

    // If both inputs are valid non-zero numbers, set status to readyToSwap
    // Otherwise, set to notReadyToSwap
    let newIsValid;
    if (!isShallowUpdate) {
      // Not a shallow update
      newIsValid = isNonzeroNumber(amount) && (amount <= tokenListState.state.tokenList.tokens[inputs.state.swap.to.tokenIndex].balance);
    } else {
      // Shallow update (called by the other input)
      newIsValid = isNonzeroNumber(amount);
    }
    let newStatus = (newIsValid ? "readyToSwap" : "notReadyToSwap" );
    // console.log(["newIsValid", newIsValid]);
    // console.log(["newStatus", newStatus]);

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
    dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
    notification.dispatch({ type: 'SHOW_NOTIFICATION', payload: { 
      heading: "Creating allowance",
      message: "Waiting to receive confirmation..."
    }});
    let isApproved = await incAllowance(inputs.state.swap.from, inputs.state.swap.to)
    // let isApproved = await testContractCall(inputs.state.swap.from, inputs.state.swap.to)
    .then(function(result) {
      alert(result);
      dispatch({ type: 'UPDATE_SWAP_APPROVAL', payload: { needsApproval: false }});
      notification.dispatch({ type: 'SHOW_NOTIFICATION', payload: { 
        heading: "Token swap approved",
        message: "You can now make a swap."
      }});
    })
    .catch(function(error) {
      notification.dispatch({ type: 'SHOW_NOTIFICATION', payload: { 
        heading: "Failed to create allowance.",
        message: error
      }});
    });
  }
  // might need to use useCallback to get result when window changes
  //
  // const handleApprovalSubmission = useCallback(async () => {
  //   dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
  //   dispatch({ type: 'SUBMIT_ALLOWANCE' });
  //   notification.dispatch({ type: 'SHOW_NOTIFICATION', payload: { 
  //     heading: "Creating allowance",
  //     message: "Waiting to receive confirmation..."
  //   }});
  //   let isApproved = await incAllowance(inputs.state.swap.from, inputs.state.swap.to)
  // }, []);

  async function handleSwap() {
    try {
      dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
      let swap = await swapFromOut(inputs.state.swap.from, inputs.state.swap.to)
        .then(function(result) {
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
      });
    } catch (e) {
      console.error(e);
    }
  }

  // Clear inputs
  function clearInputs() {
    setFromAmount("");
    setToAmount("");
    dispatch({type: 'CLEAR_SWAP_INPUTS'});
    dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
  }

  function switchInputs() {
    let oldTo = toAmount;
    setToAmount("")
    handleFromAmountChange(oldTo);
    dispatch({type: 'SWITCH_SWAP_INPUTS'});
    dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
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
                value={fromAmount}
                className="form-control border-0 bg-transparent"
                placeholder="0.0"
                onChange={(e) => handleFromAmountChange(e.target.value, false)}
              />
            </div>
          </Col>
          <Col xl={2} lg={3} md={4} sm={4} xs={12} className="d-flex flex-row-reverse align-items-center mr-2">
            <div className="text-right">

              {/* Show max balance if any balance of selected token exists */}
              {(fromBalance)
                && <small className="mr-3 text-secondary">
                  Max:{' '}
                  <u
                    onClick={(e) => handleFromAmountChange(
                      (fromBalance - 0.04).toString(), // subtract 0.04 for call
                      false) // not a shallow update
                    }
                    style={{ cursor: 'pointer' }}
                  >
                    {Number(fromBalance).toFixed(2)}
                  </u>
                </small>
              }

              <br/>
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
                value={toAmount}
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

      <div className="text-center my-2">
        {/* Display textual information before user swaps */}
        { ((inputs.state.swap.status === "readyToSwap")) &&
          <small className="text-secondary">
            You'll get at least <b className="text-black">{inputs.state.swap.to.amount}</b> {inputs.state.swap.to.symbol}{' '}
            for <b className="text-black">{inputs.state.swap.from.amount}</b> {inputs.state.swap.from.symbol}.
          </small>
        }
        {/* Clear button */}
        <Button size="sm" variant="warning" onClick={clearInputs} className="ml-2">Clear</Button>
      </div>

      {/* Display approve button if NEP-21 -> ____ swap */}
      {(inputs.state.swap.needsApproval)
        && <Button variant="warning" block
             disabled={(inputs.state.swap.status !== "readyToSwap")}
             onClick={handleApprovalSubmission}
           >
             Approve tokens <small>+0.04 NEAR</small>
           </Button>}
      
      {/*<Button variant="primary" block
       onClick={handleApprovalSubmission}
     >Approve tokens (test button)</Button>*/}

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