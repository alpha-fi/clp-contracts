import React, { useContext } from "react";

import { GlobalContext } from "../contexts/GlobalContext";

import PriceInputCard from "./PriceInputCard"

import Button from 'react-bootstrap/Button';

import { BsPlus } from "react-icons/bs";

import styled from "@emotion/styled";
const Hr = styled("hr")`
  border-top: 1px solid ${props => props.theme.hr}
`;

export default function PoolTab() {

  const globalState = useContext(GlobalContext);

  return (
    <>
      <p className="text-center my-1 text-secondary" style={{ 'letterSpacing': '3px' }}><small>POOL</small></p>
      <PriceInputCard
        label="Token amount"
        name="input1"
        logoUrl={globalState.state.pool.input1.logoUrl}
        symbol={globalState.state.pool.input1.symbol}
        tokenIndex={globalState.state.pool.input1.tokenIndex}
      />
      <div className="text-center my-2">
        <BsPlus/>
      </div>
      <PriceInputCard
        label="NEAR amount"
        name="input2"
        logoUrl={globalState.state.pool.input2.logoUrl}
        symbol={globalState.state.pool.input2.symbol}
        tokenIndex={globalState.state.pool.input2.tokenIndex}
      />
      <br/>
      <Button variant="warning" block disabled>Add Liquidity</Button>
      <Hr/>
      <p className="text-center text-secondary my-1" style={{ 'letterSpacing': '3px' }}><small>MY LIQUIDITY</small></p>
      <p className="text-center text-secondary my-5" style={{ 'fontSize': '70%' }}><i>You are not providing any liquidity.</i></p>
    </>
  );
}
