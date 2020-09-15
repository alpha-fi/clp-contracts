import React, { useContext } from "react";

import { GlobalContext } from "../contexts/GlobalContext";

import PriceInputCard from "./PriceInputCard";

import Button from 'react-bootstrap/Button';

import { BsArrowUpDown } from "react-icons/bs";

export default function SwapTab() {

  const globalState = useContext(GlobalContext);

  return (
    <>
      <PriceInputCard
        label="From"
        name="from"
        amount={globalState.state.swap.from.amount}
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
        amount={globalState.state.swap.to.amount}
        logoUrl={globalState.state.swap.to.logoUrl}
        symbol={globalState.state.swap.to.symbol}
        tokenIndex={globalState.state.swap.to.tokenIndex}
      />
      <br/>
      <Button variant="warning" block disabled>Swap</Button>
    </>
  );
}
