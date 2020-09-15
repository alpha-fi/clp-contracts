import React, { useContext } from "react";

import { GlobalContext } from "../contexts/GlobalContext";

import PriceInputCard from "./PriceInputCard"

import Button from 'react-bootstrap/Button';

import { BsPlus } from "react-icons/bs";

export default function PoolTab() {

  const globalState = useContext(GlobalContext);

  return (
    <>
      <PriceInputCard
        label="Input"
        name="input1"
        amount={globalState.state.pool.input1.amount}
        logoUrl={globalState.state.pool.input1.logoUrl}
        symbol={globalState.state.pool.input1.symbol}
        tokenIndex={globalState.state.pool.input1.tokenIndex}
      />
      <div className="text-center my-2">
        <BsPlus/>
      </div>
      <PriceInputCard
        label="Input"
        name="input2"
        amount={globalState.state.pool.input2.amount}
        logoUrl={globalState.state.pool.input2.logoUrl}
        symbol={globalState.state.pool.input2.symbol}
        tokenIndex={globalState.state.pool.input2.tokenIndex}
      />
      <br/>
      <Button variant="warning" block disabled>Add Liquidity</Button>
    </>
  );
}
