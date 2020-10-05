import React, { useEffect, useState } from "react";

import { convertToE24Base, browsePools, poolInfo } from "../services/near-nep21-util";

import PoolInputCards from "./PoolInputCards"
import PoolInfoCard from "./PoolInfoCard"

import Button from 'react-bootstrap/Button';

import { BsPlus } from "react-icons/bs";

import styled from "@emotion/styled";
const Hr = styled("hr")`
  border-top: 1px solid ${props => props.theme.hr}
`;

export default function PoolTab() {

  const [pools, setPools] = useState([]);

  async function fetchPools() {
    browsePools()
    .then(function(fetchedPools) {
      fetchedPools.map((fetchedPoolInfo, index) => {
        poolInfo(fetchedPoolInfo)
        .then(function(poolInfo) {
          // Set state to an array of pools and include the name of the pool
          // @TODO: find token within TokenListContext and include images, symbol name. etc.
          setPools(pools => [...pools, {...poolInfo, name: fetchedPools[index]}]);
        });
      });
    });
  }

  //----------------------------
  //----------------------------
  //----------------------------
  useEffect(function() {
     fetchPools();
   }, []);

  return (
    <>
      <p className="text-center my-1 text-secondary" style={{ 'letterSpacing': '3px' }}><small>POOLS</small></p>
      {pools.map((pool, index) => (
        <PoolInfoCard key={index} 
                    ynear={pool.ynear} 
                    reserve={pool.reserve} 
                    total_shares={pool.total_shares} 
                    name={pool.name} 
                    />
      ))}
      <p className="mt-4 text-center text-secondary"><small><i>Don't see a pair you're looking for? Create a new pool below.</i></small></p>
      <Hr className="mt-4"/>
      <p className="text-center my-1 text-secondary" style={{ 'letterSpacing': '3px' }}><small>PROVIDE LIQUIDITY</small></p>
      <PoolInputCards/>
    </>
  );
}
