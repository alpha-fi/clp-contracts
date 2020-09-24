import React, { useContext } from "react";

import { InputsContext } from "../contexts/InputsContext";

import PriceInputCard from "./PriceInputCard"

import Button from 'react-bootstrap/Button';

import { BsPlus } from "react-icons/bs";

import styled from "@emotion/styled";
const Hr = styled("hr")`
  border-top: 1px solid ${props => props.theme.hr}
`;

export default function PoolTab() {

  const inputs = useContext(InputsContext);

  return (
    <>
      <p className="text-center my-1 text-secondary" style={{ 'letterSpacing': '3px' }}><small>POOL</small></p>
      <PriceInputCard
        label="Input"
        name="input1"
        logoUrl={inputs.state.pool.input1.logoUrl}
        symbol={inputs.state.pool.input1.symbol}
        type={inputs.state.pool.input1.type}
        tokenIndex={inputs.state.pool.input1.tokenIndex}
      />
      <div className="text-center my-2">
        <BsPlus/>
      </div>
      <PriceInputCard
        label="Input"
        name="input2"
        logoUrl={inputs.state.pool.input2.logoUrl}
        symbol={inputs.state.pool.input2.symbol}
        type={inputs.state.pool.input2.type}
        tokenIndex={inputs.state.pool.input2.tokenIndex}
        currencySelectionDisabled
      />
      <br/>
      <Button variant="warning" block disabled>Add Liquidity</Button>
      <Hr/>
      <p className="text-center text-secondary my-1" style={{ 'letterSpacing': '3px' }}><small>MY LIQUIDITY</small></p>
      <p className="text-center text-secondary my-5" style={{ 'fontSize': '70%' }}><i>You are not providing any liquidity.</i></p>
    </>
  );
}
