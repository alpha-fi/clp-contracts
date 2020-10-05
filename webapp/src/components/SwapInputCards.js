import React, { useEffect, useReducer, useContext, useCallback } from "react";

import { convertToE24Base, convertToE24Base5Dec, getBalanceNEP } from '../services/near-nep21-util'
import { produce } from 'immer';

import {getCurrentBalance, saveInputsStateLocalStorage, setCurrencyIndex} from "./CurrencyTable"
import findCurrencyLogoUrl from "../services/find-currency-logo-url";
import { calcPriceFromIn, calcPriceFromOut, swapFromOut, incAllowance, getAllowance } from "../services/near-nep21-util";
import { isNonzeroNumber, delay } from "../utils"

import { InputsContext } from "../contexts/InputsContext";
import { TokenListContext } from "../contexts/TokenListContext";
import { NotificationContext } from "../contexts/NotificationContext";

import Row from 'react-bootstrap/Row';
import Col from 'react-bootstrap/Col';
import Button from 'react-bootstrap/Button';
import Alert from 'react-bootstrap/Alert';
import Spinner from 'react-bootstrap/Spinner';

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
async function testContractCall(token1, token2) {
  console.log("testContractCall called. [token1.amount, token2.amount]")
  console.log([token1.amount, token2.amount])
  return await window.nep21.get_balance({ owner_id: window.walletConnection.getAccountId() });
}


//MAIN COMPONENT
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
      if (inputs.state.swap.previous < inputs.state.swap.in.allowance) {
        // Notify user
        notification.dispatch({
          type: 'SHOW_NOTIFICATION', payload: {
            heading: "Approval complete",
            message: "Your NEP-21 token is now approved to swap."
          }
        });
        // Reset needsApproval
        dispatch({
          type: 'UPDATE_SWAP_APPROVAL', payload: {
            needsApproval: false
          }
        });
        
        // Update status
        setStatus("readyToSwap")
        //dispatch({type: 'UPDATE_SWAP_STATUS', payload: {status: "readyToSwap"}});

      } 
      else {
        // Approval is not successful
        notification.dispatch({
          type: 'SHOW_NOTIFICATION', payload: {
            heading: "Approval unsuccessful",
            message: "Please try again."
          }
        });
      }

    } else if (inputs.state.swap.status == "isSwapping") {

      // If swap is successful
      if (inputs.state.swap.previous < inputs.state.swap.in.balance) {
        // Reset amounts
        clearInputs();
        // Reset needsApproval
        dispatch({
          type: 'UPDATE_SWAP_APPROVAL', payload: {
            needsApproval: (inputs.state.swap.out.type === "NEP-21")
          }
        });
        // Notify user
        notification.dispatch({
          type: 'SHOW_NOTIFICATION', payload: {
            heading: "Swap complete",
            message: "Your swap has been submitted."
          }
        });

        // Update status
        setStatus("notReadyToSwap")
        //dispatch({type: 'UPDATE_SWAP_STATUS', payload: {status: "notReadyToSwap" } });

      } 
      else {
        // Swap is not successful
        notification.dispatch({
          type: 'SHOW_NOTIFICATION', payload: {
            heading: "Swap unsuccessful",
            message: "Your swap has failed."
          }
        });
      }
    }
  }

  // Initializes allowance of from token
  async function initializeFromAllowance() {
    await delay(500).then(async function () {
      if (inputs.state.swap.in.type == "NEP-21") {
        try {
          let allowance = await getAllowance(inputs.state.swap.in);
          dispatch({ type: 'UPDATE_IN_ALLOWANCE', payload: { allowance: allowance } });
        } catch (e) {
          console.error(e);
        }
      }
    });
  }

  function setStatus(newStatus, newError, previous) {
    dispatch({ type: 'UPDATE_SWAP_STATUS', payload: { 
        status: newStatus, 
        error: newError||"",
        previous: previous,
      } });
  }

  // Updates swap status (usually readyToSwap, notReadyToSwap) and error message. Called by handleToAmountChange() 
  function updateStatus(inAmount, outAmount) {

    let newError;
    let newStatus = "notReadyToSwap";

    // Look for errors in order of most critical to least critical
    if (inputs.state.swap.in.tokenIndex === inputs.state.swap.out.tokenIndex) {
      newError = "Cannot swap to same currency.";
    } else if (!window.accountId) {
      newError = "Not signed in.";
    } else if (Number(inAmount) > Number(inputs.state.swap.in.balance)) {
      newError = "Input exceeds balance.";
    } else if (Number(inAmount) === 0 && Number(outAmount) !== 0) {
      newError = "Insufficient liquidity for trade."
    } else if (!isNonzeroNumber(outAmount)) {
      newError = "Enter a non-zero number.";
    } else {
      newError = null;
      newStatus = "readyToSwap"
    }

    setStatus(newStatus,newError );
  }

  // Handles updating button view and input information
  async function handleInTokenUpdate() {
    // Update image, symbol, address, tokenIndex, and type of selected currency
    await delay(1000).then(async function () { // delay to wait for balances update
      // Update image, symbol, address, tokenIndex, and type of selected currency
      let newToken = tokenListState.state.tokens[inputs.state.swap.in.tokenIndex];
      dispatch({
        type: 'UPDATE_IN_SELECTED_CURRENCY',
        payload: {
          logoUrl: findCurrencyLogoUrl(inputs.state.swap.in.tokenIndex, tokenListState.state.tokens),
          symbol: newToken.symbol,
          type: newToken.type,
          tokenIndex: inputs.state.swap.in.tokenIndex,
          address: newToken.address,
          balance: newToken.balance
        }
      });
    })
      .then(async function () {
        if (inputs.state.swap.in.type == "NEP-21") {
          try {
            let allowance = await getAllowance(inputs.state.swap.in);
            dispatch({ type: 'UPDATE_IN_ALLOWANCE', payload: { allowance: allowance } });
          } catch (e) {
            console.error(e);
          }
        }
      });

  }
  async function handleOutTokenUpdate() {
    // Update image, symbol, address, tokenIndex, and type of selected currency
    await delay(1000).then(async function () { // delay to wait for balances update
      let newToken = tokenListState.state.tokens[inputs.state.swap.out.tokenIndex];
      dispatch({
        type: 'UPDATE_OUT_SELECTED_CURRENCY',
        payload: {
          logoUrl: findCurrencyLogoUrl(inputs.state.swap.out.tokenIndex, tokenListState.state.tokens),
          symbol: newToken.symbol,
          type: newToken.type,
          tokenIndex: inputs.state.swap.out.tokenIndex,
          address: newToken.address,
          balance: newToken.balance
        }
      });
    })
  }


  function newUserBalanceReceived(newBalance){
    //check if we're recovering state after a SDE (State Destrcution Event) ;)
    if (inputs.state.swap.previous) {
      if (inputs.state.swap.previous.padStart(50, "0") < newBalance.padStart(50, "0")) {
        //previos && balance increased
        //Let's assume a swap went well
        notification.dispatch({
          type: 'SHOW_NOTIFICATION',
          payload: {
            heading: "Swap was successful",
            message: "your balance went from " + convertToE24Base5Dec(inputs.state.swap.previous) + " to " + convertToE24Base5Dec(newBalance),
            show: true,
          }
        });
      }
      inputs.dispatch({type:'CLEAR_PREVIOUS'}) //clear it
    }
  }

  //fetch balance for a leg of the swap
  async function updateSwapBal(inputs, inOut, tokenListState) {

    let yoctos = await getCurrentBalance(inOut.tokenIndex, tokenListState)

    inputs.dispatch({
      type: 'SET_TOKEN_BALANCE',
      payload: {
        index: inOut.tokenIndex,
        balance: yoctos
      }
    });

    return yoctos; //returns balance

  }

  //fetch balance for both legs, in and out in the swap
  //when the call completes, update the UI via setCurrencyIndex
  function updateSwapBalances(tokenListState, inputs) {

    //first leg
    let Promise1 = updateSwapBal(inputs, inputs.state.swap.out, tokenListState)
      .then((newBalance)=>{
        setCurrencyIndex("out", inputs.state.swap.out.tokenIndex, inputs, tokenListState);
        newUserBalanceReceived(newBalance)
      })

    //second leg
    let Promise2 = updateSwapBal(inputs, inputs.state.swap.in, tokenListState)
      .then(setCurrencyIndex("in", inputs.state.swap.in.tokenIndex, inputs, tokenListState))

    //when both resolve
    Promise.all([Promise1, Promise2])
      .then(()=>{     
        inputs.dispatch({type:'SET_STATUS_READY'})
      })
  }

  //computes "in" from the "out" wanted and the price
  function computeInAmountRequired(outAmount){
    // Calculate the value of the other input box (only called when the user types)
    let updatedToken = { ...inputs.state.swap.out, amount: outAmount };
    return calcPriceFromOut(inputs.state.swap.in, updatedToken)
      .then(function (result) {
        dispatch({ type: 'SET_OUT_AMOUNT', payload: { amount: result, isValid: isNonzeroNumber(result) } });
        updateStatus(result, outAmount); // Update status and/or error message
      })
  }
  
  //---------------------------------------------------------------
  //---------------------------------------------------------------
  // APP after mounted / recovery after SDE -----------------------
  //---------------------------------------------------------------
  // Load icons and symbol for the current selected currency/token
  // TODO - async load info for the token list --------------------
  //---------------------------------------------------------------
  //---------------------------------------------------------------
  useEffect(() => {
    updateSwapBalances(tokenListState, inputs) //get balances from the 2 selected currencies
    updateFromAllowance(inputs.state.swap.in);
    //handleFromTokenUpdate();
    //handleToTokenUpdate();
    //checkStatuses();
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
    dispatch({
      type: 'SET_IN_AMOUNT', payload: {
        amount: amount,
        isValid: newIsValid,
      }
    });

    setStatus("notReadyToSwap"); //can't swap until the data comes back
    return computeInAmountRequired(amount);
  }


  //handleIncAllowance
  function handleApprovalSubmission() {
    setStatus("isApproving" );
    
    saveInputsStateLocalStorage(inputs.state);

    //ok, here it comes another SDE
    //dispatch({ type: 'UPDATE_SWAP_STATUS', payload: { status: "isApproving", error: null, previous: inputs.state.swap.in.allowance } });
    incAllowance(inputs.state.swap.in)
    // await delay(1000).then(function () {
    //   dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
    // })
    //   .then(async function () {
    //     let isApproved = await incAllowance(inputs.state.swap.in);
    //   })
  }

  function setError(error){
    setStatus(inputs.state.swap.status,"Invalid amounts")
  }

  async function handleSwap() {

    try {

      if (!inputs.state.swap.in.isValid || !inputs.state.swap.out.isValid){
        setError("Invalid amounts")
        return
      }

      //save state with new flags because it's gonna be destroyed
      const newState= produce(inputs, draft => {
        draft.state.swap.status="isSwapping";
        draft.state.swap.previous=inputs.state.swap.out.balance; //prev balance
      })
      //status is saved 
      localStorage.setItem("inputs", JSON.stringify(newState.state));

      //call near wallet -- navigates out - state destroyed
      await swapFromOut(inputs.state.swap.in, inputs.state.swap.out);

    } catch (e) {
      console.error(e);
    }
  }

  // Clear inputs
  function clearInputs() {
    dispatch({ type: 'CLEAR_SWAP_INPUTS' });
    dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
  }

  // Updates allowance of from token
  async function updateFromAllowance(token) {
    await delay(500).then(async function () {
      if (token.type == "NEP-21") {
        try {
          let allowance = await getAllowance(token);
          let needsApproval = true;
          try{ needsApproval = inputs.state.swap.in.allowance<inputs.state.swap.in.amount } catch (ex){};
          dispatch({ type: 'UPDATE_IN_ALLOWANCE', payload: { allowance: allowance, needsApproval:needsApproval } });
        } catch (e) {
          console.error(e);
        }
      }
    });
  }

  // Move "To" input and currency to "From" and vice versa
  function switchInputs() {
    let oldFromAmount = inputs.state.swap.in.amount;
    let oldTo = inputs.state.swap.out;
    dispatch({ type: 'SWITCH_SWAP_INPUTS' });
    handleToAmountChange(oldFromAmount);
    updateFromAllowance(oldTo);
  }

  function readyToSwap() { 
    return inputs.state.swap.status == "readyToSwap" 
  }
  function waiting() { 
    return inputs.state.swap.status == "fetchingData" 
      || inputs.state.swap.status == "isApproving" 
      || inputs.state.swap.status == "isSwapping" 
  }

  function statusText() {
    if (!window.walletConnection.isSignedIn()) return "Not Connected";
    switch (inputs.state.swap.status) {
      case "readyToSwap": { return "ready" }
      case "notReadyToSwap": { return "Not ready" }
      case "fetchingData": { return "Retrieving data..." }
      case "isApproving": { return "Approving..." }
      case "isSwapping": { return "Swapping..." }
        deafult: return inputs.state.swap.status;
    }
  }

  function needSpinner() {
    return window.walletConnection.isSignedIn() && inputs.state.swap.status=="fetchingData";
  }

  return (
    <>

      {/* STATUS */}

     {/* <Row className="px-2 status">
        <Alert variant="warning"
          className="status"
          style={{ display: waiting() ? 'block' : 'none' }}
        >
          <Spinner
            style={{ display: needSpinner() ? 'inline-block' : 'none' }}
            as="span"
            className="spinner"
            animation="grow"
            size="sm"
            role="status"
            aria-hidden="true"
          />
          {statusText()}
        </Alert>
      </Row> */}

      <p className="text-center my-1 text-secondary" style={{ 'letterSpacing': '3px' }}>
        <small>SWAP</small>
      </p>

      <Theme className="py-2">
        <label className="ml-4 mb-1 mt-0">
          <small className="text-secondary">I want:</small>
        </label>
        <Row className="px-2">
          <Col>
            <div className="input-group-lg mb-1" disabled={!readyToSwap()}>
              {/* IN / I WANT / TO INPUT */}
              <input
                type="text"
                value={inputs.state.swap.out.amount || ''}
                className="form-control border-0 bg-transparent"
                placeholder="0.0"
                onChange={(e) => handleOutAmountChange(e.target.value)}
              />
            </div>
          </Col>
          <Col xl={3} lg={3} md={4} sm={4} xs={12} className="d-flex flex-row-reverse align-items-center mr-2"  style={{minWidth:"120px"}}>
            <div className="text-right">

              {/* Show balance if any balance of selected token exists */}
              {(inputs.state.swap.out.balance && inputs.state.swap.out.balance !== 0)
                && <>
                  <small className="mr-3 text-secondary">
                  Your Balance:<br />{convertToE24Base5Dec(inputs.state.swap.out.balance)}
                  </small>
                  <br />
                </>
              }

              <Button size="sm" variant="outline-secondary" className="mr-1" style={{ 'whiteSpace': 'nowrap' }} onClick={handleCurrencySelectionModalTo}>
                <img src={inputs.state.swap.out.logoUrl} width="15px" className="align-middle pb-1" />
                {' '}{inputs.state.swap.out.symbol}
                {' '}
                <BsCaretDownFill />
              </Button>
              <br />
              <small className="mr-3 text-secondary" style={{ 'whiteSpace': 'nowrap', 'fontSize': '60%' }}>
                {inputs.state.swap.out.type === "ERC-20"
                  ? <><FaEthereum /> ERC-20</>
                  : inputs.state.swap.out.type
                }
              </small>
            </div>
          </Col>
        </Row>
      </Theme>

      <div className="text-center my-2">
        <span onClick={switchInputs} style={{ cursor: 'pointer' }}><BsArrowUpDown /></span>
      </div>

      <Theme className="py-2">
        <label className="ml-4 mb-1 mt-0">
          <small className="text-secondary">I'll provide:</small>
        </label>
        <Row className="px-2">
          <Col>
            <div className="input-group-lg mb-1" >
              {/* OUT / I'LL PROVIDE / FROM INPUT */}
              <input
                type="text"
                readOnly
                value={convertToE24Base5Dec(inputs.state.swap.in.amount)}
                className="form-control border-0 bg-transparent"
                placeholder="0.0"
              />
            </div>
          </Col>
          <Col xl={3} lg={3} md={4} sm={4} xs={12} className="d-flex flex-row-reverse align-items-center mr-2" style={{minWidth:"120px"}}>
            <div className="text-right">

              {/* Show balance if any balance of selected token exists */}
              {(inputs.state.swap.in.balance && inputs.state.swap.in.balance !== 0)
                && <>
                  <small className="mr-3 text-secondary">
                    Your Balance:<br />{convertToE24Base5Dec(inputs.state.swap.in.balance)}
                  </small>
                  <br />
                </>
              }

              <Button size="sm" variant="outline-secondary" className="mr-1" style={{ 'whiteSpace': 'nowrap' }} onClick={handleCurrencySelectionModalFrom}>
                <img src={inputs.state.swap.in.logoUrl} width="15px" className="align-middle pb-1" />
                {' '}{inputs.state.swap.in.symbol}
                {' '}
                <BsCaretDownFill />
              </Button>
              <br />
              <small className="mr-3 text-secondary" style={{ 'whiteSpace': 'nowrap', 'fontSize': '60%' }}>
                {inputs.state.swap.in.type === "ERC-20"
                  ? <><FaEthereum /> ERC-20</>
                  : inputs.state.swap.in.type
                }
              </small>
            </div>
          </Col>
        </Row>
      </Theme>

      {(inputs.state.swap.in.allowance && inputs.state.swap.in.type==="NEP-21") &&
        <div className="text-right pr-3 text-secondary">
          <small>Current {inputs.state.swap.in.symbol} allowance: {convertToE24Base5Dec(inputs.state.swap.in.allowance)}</small>
        </div>
      }

      <div className="text-right my-2 pr-2">

        <small className="text-danger mr-1 text-center">{inputs.state.swap.error}</small>

        {/* Display textual information before user swaps */}
        {!inputs.state.swap.error &&
          <small className="text-secondary">
            You'll get <b className="text-black">{inputs.state.swap.out.amount}</b> {inputs.state.swap.out.symbol}{' '} 
            for <b className="text-black">{convertToE24Base5Dec(inputs.state.swap.in.amount)}</b> {inputs.state.swap.in.symbol}.
          </small>
        }
        

        {/* Clear button and clippage */}
          <small className="mx-2 text-secondary">Slippage: 1%</small>
          <Button size="sm" variant="warning" 
            disable={readyToSwap()?"true":"false"}
            onClick={clearInputs} className="ml-2">
            Clear
          </Button>

      </div>

      {/* Display approve button if NEP-21 -> ____ swap */}
      {(inputs.state.swap.needsApproval)
        && <>
          <small className="text-secondary">Step 1: </small>
          <Button variant="warning" block
            //disabled={(inputs.state.swap.status !== "readyToSwap" || inputs.state.swap.error)}
            onClick={handleApprovalSubmission}
          >
            {(inputs.state.swap.status !== "isApproving")
              ? <>
                Approve {inputs.state.swap.in.symbol} allowance {
                  (inputs.state.swap.in.amount && inputs.state.swap.in.amount !== 0)
                    ? <>of {convertToE24Base5Dec(inputs.state.swap.in.amount)}</>
                    : ""
                }
              </>
              : "Approving..."
            }
          </Button>
        </>
      }

      {/* Enable submission only if inputs are valid */}
        {(inputs.state.swap.needsApproval == true) && 
          <small className="text-secondary">Step 2: </small>
        }

      {/* SWAP BUTTON */}
      <Button variant="warning" block
        onClick={handleSwap}
        disable={readyToSwap()?"false":"true"}
      >
        Swap
      </Button>

    </>
  );
}