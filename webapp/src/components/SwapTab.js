import React, { useContext } from "react";

import { InputsContext } from "../contexts/InputsContext";

import PriceInputCard from "./PriceInputCard";

import Button from 'react-bootstrap/Button';

import { BsArrowUpDown } from "react-icons/bs";

export default function SwapTab() {

  const inputs = useContext(InputsContext);

  return (
    <>
      <p className="text-center my-1 text-secondary" style={{ 'letterSpacing': '3px' }}><small>SWAP</small></p>
      <PriceInputCard
        label="From"
        name="from"
        logoUrl={inputs.state.swap.from.logoUrl}
        symbol={inputs.state.swap.from.symbol}
        type={inputs.state.swap.from.type}
        tokenIndex={inputs.state.swap.from.tokenIndex}
      />
      <div className="text-center my-2">
        <BsArrowUpDown/>
      </div>
      <PriceInputCard
        label="To"
        name="to"
        logoUrl={inputs.state.swap.to.logoUrl}
        symbol={inputs.state.swap.to.symbol}
        type={inputs.state.swap.to.type}
        tokenIndex={inputs.state.swap.to.tokenIndex}
      />
      <br/>
      {/* Enable submission only if inputs are valid */}
      {(inputs.state.swap.from.isValid && inputs.state.swap.to.isValid)
        ? <Button variant="warning" block>Swap</Button>
        : <Button variant="warning" block disabled>Swap</Button>
      }
    </>
  );
}
