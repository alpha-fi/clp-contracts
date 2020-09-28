import React, { useContext } from "react";

import SwapInputCards from "./SwapInputCards";

import Button from 'react-bootstrap/Button';

import { BsArrowUpDown } from "react-icons/bs";

export default function SwapTab() {

  return (
    <>
      <p className="text-center my-1 text-secondary" style={{ 'letterSpacing': '3px' }}><small>SWAP</small></p>
      <SwapInputCards/>
    </>
  );
}
