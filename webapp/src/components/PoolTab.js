import React, { useContext } from "react";

import { InputsContext } from "../contexts/InputsContext";

import PriceInputCard from "./PriceInputCard"
import PoolInfoCard from "./PoolInfoCard"

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
      <p className="text-center my-1 text-secondary" style={{ 'letterSpacing': '3px' }}><small>TOP POOLS</small></p>
      <PoolInfoCard tokenIndex="2" />
      <PoolInfoCard tokenIndex="3" hasProvidedLiquidity/>
      <p className="mt-4 text-center text-secondary"><small><i>Don't see a pair you're looking for? Create a new pool below.</i></small></p>
      <Hr className="mt-4"/>
      <p className="text-center my-1 text-secondary" style={{ 'letterSpacing': '3px' }}><small>PROVIDE LIQUIDITY</small></p>
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
      {/* Enable submission only if inputs are valid */}
      {(inputs.state.pool.input1.isValid && inputs.state.pool.input2.isValid)
        ? <Button variant="warning" block disabled>Add Liquidity</Button>
        : <Button variant="warning" block disabled>Add Liquidity</Button>
      }
    </>
  );
}
