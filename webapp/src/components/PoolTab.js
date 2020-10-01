import React, { useEffect } from "react";

import { browsePools, poolInfo } from "../services/near-nep21-util";

import PoolInputCards from "./PoolInputCards"
import PoolInfoCard from "./PoolInfoCard"

import Button from 'react-bootstrap/Button';

import { BsPlus } from "react-icons/bs";

import styled from "@emotion/styled";
const Hr = styled("hr")`
  border-top: 1px solid ${props => props.theme.hr}
`;

export default function PoolTab() {

  async function fetchPools() {
    browsePools()
    .then(function(pools) {
      pools.map((pool, index) => {
        console.log(poolInfo(pool[index]))
      });
    });
  }

  useEffect(function() {
    fetchPools();
  }, []);

  return (
    <>
      <p className="text-center my-1 text-secondary" style={{ 'letterSpacing': '3px' }}><small>TOP POOLS</small></p>
      <PoolInfoCard tokenIndex="2" />
      <PoolInfoCard tokenIndex="3" hasProvidedLiquidity/>
      <p className="mt-4 text-center text-secondary"><small><i>Don't see a pair you're looking for? Create a new pool below.</i></small></p>
      <Hr className="mt-4"/>
      <p className="text-center my-1 text-secondary" style={{ 'letterSpacing': '3px' }}><small>PROVIDE LIQUIDITY</small></p>
      <PoolInputCards/>
    </>
  );
}
