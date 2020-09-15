import React from "react";

import PriceInputCard from "./PriceInputCard"

import Button from 'react-bootstrap/Button';

import { BsPlus } from "react-icons/bs";

export default function PoolTab() {
  return (
    <>
      <PriceInputCard label="Input"/>
      <div className="text-center my-2">
        <BsPlus/>
      </div>
      <PriceInputCard label="Input"/>
      <br/>
      <Button variant="warning" block disabled>Add Liquidity</Button>
    </>
  );
}
