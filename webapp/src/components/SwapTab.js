import React, { useContext } from "react";

import { InputsContext } from "../contexts/InputsContext";

import SwapInputCards from "./SwapInputCards";

import Button from 'react-bootstrap/Button';

import { BsArrowUpDown } from "react-icons/bs";

export default function SwapTab() {

  const inputs = useContext(InputsContext);

  return (
    <>
      <p className="text-center my-1 text-secondary" style={{ 'letterSpacing': '3px' }}><small>SWAP</small></p>
      <SwapInputCards/>
      <br/>
      {/* Enable submission only if inputs are valid */}
      {(inputs.state.swap.from.isValid && inputs.state.swap.to.isValid)
        ? <Button variant="warning" block>Swap</Button>
        : <Button variant="warning" block disabled>Swap</Button>
      }
    </>
  );
}
