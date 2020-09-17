import React, { useContext } from "react";

import { GlobalContext } from "../contexts/GlobalContext";

import PriceInputCard from "./PriceInputCard";

import Button from 'react-bootstrap/Button';

import { BsArrowUpDown } from "react-icons/bs";

export default function SwapTab() {

  const globalState = useContext(GlobalContext);

  return (
    <>
      <p className="text-center my-1 text-secondary" style={{ 'letterSpacing': '3px' }}><small>SWAP</small></p>
      <PriceInputCard
        label="From"
        name="from"
        logoUrl={globalState.state.swap.from.logoUrl}
        symbol={globalState.state.swap.from.symbol}
        tokenIndex={globalState.state.swap.from.tokenIndex}
      />
      <div className="text-center my-2">
        <BsArrowUpDown/>
      </div>
      <PriceInputCard
        label="To"
        name="to"
        logoUrl={globalState.state.swap.to.logoUrl}
        symbol={globalState.state.swap.to.symbol}
        tokenIndex={globalState.state.swap.to.tokenIndex}
      />
      <br/>
      <Button variant="warning" block disabled>Swap</Button>
    </>
  );
}
