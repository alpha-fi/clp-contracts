import React from "react";

import PriceInputCard from "./PriceInputCard"

import Button from 'react-bootstrap/Button';

import { BsArrowUpDown } from "react-icons/bs";

export default function SwapTab() {
  return (
    <>
      <PriceInputCard label="From"/>
      <div className="text-center my-2">
        <BsArrowUpDown/>
      </div>
      <PriceInputCard label="To"/>
      <br/>
      <Button variant="warning" block disabled>Swap</Button>
    </>
  );
}
