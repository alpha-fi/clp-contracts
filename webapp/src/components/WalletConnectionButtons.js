import React, { useContext } from "react";

import { login, logout } from '../utils'

import { Web3Context, signInWithWeb3 } from '../contexts/Web3Context';

import Dropdown from 'react-bootstrap/Dropdown';

export default function WalletConnectionButtons() {

  // Web3 state
  const web3State = useContext(Web3Context);
  const { web3Modal, setWeb3Modal, setCurrentUser, currentUser } = web3State;

  // Initial connection buttons
  let nearConnectionBtn, ethConnectionBtn;

  // Set NEAR connection button with correct label and function call
  if (!window.walletConnection.isSignedIn()) {
    nearConnectionBtn = <Dropdown.Item onClick={login} className="h-100 w-100">Connect to NEAR wallet</Dropdown.Item>;
  } else {
    nearConnectionBtn = <Dropdown.Item onClick={logout} className="h-100 w-100">Disconnect NEAR wallet</Dropdown.Item>;
  }

  // Set Ethereum connection button with correct label and function call
  if (!currentUser) {
    ethConnectionBtn = <Dropdown.Item onClick={
      async () => {
        try {
          const w3c = await signInWithWeb3();
          const [account] = await w3c.web3.eth.getAccounts();
          setWeb3Modal(w3c);
          setCurrentUser(account);
        } catch (err) {
          console.log('web3Modal error', err);
        }
      }
    }>Connect to Ethereum wallet</Dropdown.Item>
  } else {
    ethConnectionBtn = <Dropdown.Item onClick={
      async () => {
        try {
          setWeb3Modal("");
          setCurrentUser("");
          await web3Modal.web3Modal.clearCachedProvider();
        } catch (err) {
          console.log('web3Modal error', err);
        }
      }
    }>Disconnect Ethereum wallet</Dropdown.Item>
  }

  return (
    <>
      {nearConnectionBtn}
      {ethConnectionBtn}
    </>
  );
}
