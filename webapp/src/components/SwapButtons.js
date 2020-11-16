import React, { useContext, useEffect } from "react";

import { InputsContext } from "../contexts/InputsContext";

import { convertToE24Base5Dec } from '../services/near-nep21-util';

import Dropdown from 'react-bootstrap/Dropdown';
import Button from 'react-bootstrap/Button';

export default function SwapButtons(props) {

  // Global state
  const inputs = useContext(InputsContext);
  const { dispatch } = inputs;

  function readyToSwap() { 
    return inputs.state.swap.status == "readyToSwap" 
  }

  // Clear inputs
  function clearInputs() {
    dispatch({ type: 'CLEAR_SWAP_INPUTS' });
    dispatch({ type: 'SAVE_INPUTS_TO_LOCAL_STORAGE' });
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

  function setSlippage(percent) {
    inputs.dispatch({
      type: 'SET_SLIPPAGE',
      payload: {
        slippage: percent,
      }
    });
  }

  // useEffect(() => {
    
  // }, [inputs.state.swap.slippage]);

  return (
    <>
      {(inputs.state.swap.in.allowance && inputs.state.swap.in.type==="NEP-21") &&
        <div className="text-right pr-3 text-secondary">
          <small>Current {inputs.state.swap.in.symbol} allowance: {convertToE24Base5Dec(inputs.state.swap.in.allowance)}</small>
        </div>
      }


      <div className="text-right mt-2 mb-1 pr-2">
        {/* Clear button and slippage */}
        <div className="btn-group">
          <Dropdown>
            <Dropdown.Toggle variant="warning" size="sm">
              Slippage: {inputs.state.swap.slippage}%
            </Dropdown.Toggle>
            <Dropdown.Menu className="mt-2">
              <Dropdown.Item onClick={e => setSlippage(0.5)}>&#177;0.5%</Dropdown.Item>
              <Dropdown.Item onClick={e => setSlippage(1)}>&#177;1%</Dropdown.Item>
              <Dropdown.Item onClick={e => setSlippage(2)}>&#177;2%</Dropdown.Item>
            </Dropdown.Menu>
          </Dropdown>
          <Button size="sm" variant="warning" 
            disable={readyToSwap()?"true":"false"}
            onClick={clearInputs} className="ml-2">
            Clear
          </Button>
        </div>
      </div>

      <div className="text-center mb-2">
        <small className="text-danger mr-1 text-center">{inputs.state.swap.error}</small>
        {/* Display textual information before user swaps */}
        {!inputs.state.swap.error &&
          <small className="text-secondary">
            You'll get <b className="text-black">{inputs.state.swap.out.amount}</b> {inputs.state.swap.out.symbol}{' '} 
            for <b className="text-black">{convertToE24Base5Dec(inputs.state.swap.in.amount)}</b> {inputs.state.swap.in.symbol}.
          </small>
        }
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
