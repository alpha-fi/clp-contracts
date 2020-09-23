import React, { useContext } from "react";

import { login, logout } from '../utils'

import { Web3Context, signInWithWeb3 } from '../contexts/Web3Context';

import Row from 'react-bootstrap/Row';
import Col from 'react-bootstrap/Col';
import Button from 'react-bootstrap/Button';

export default function WalletConnectionButtons() {

  // Web3 state
  const web3State = useContext(Web3Context);
  const { web3Modal, setWeb3Modal, setCurrentUser, currentUser } = web3State;

  // Initial connection buttons
  let nearConnectionBtn, ethConnectionBtn;

  // Set NEAR connection button with correct label and function call
  if (!window.walletConnection.isSignedIn()) {
    nearConnectionBtn = <Button variant="warning" size="sm" onClick={login} className="h-100 w-100">Connect to NEAR wallet</Button>;
  } else {
    nearConnectionBtn = <Button variant="warning" size="sm" onClick={logout} className="h-100 w-100">Disconnect NEAR wallet</Button>;
  }

  // Set Ethereum connection button with correct label and function call
  if (!currentUser) {
    ethConnectionBtn = <Button variant="warning" size="sm" className="h-100 w-100" onClick={
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
    }>Connect to Ethereum wallet</Button>
  } else {
    ethConnectionBtn = <Button variant="warning" size="sm" className="h-100 w-100" onClick={
      async () => {
        try {
          setWeb3Modal("");
          setCurrentUser("");
          await web3Modal.web3Modal.clearCachedProvider();
        } catch (err) {
          console.log('web3Modal error', err);
        }
      }
    }>Disconnect Ethereum wallet</Button>
  }

  return (
    <Row noGutters className="">
      <div className="text-center">
      <Col xs={12} className="mb-1 mr-1">
        {nearConnectionBtn}
      </Col>
      <Col xs={12} className="mb-1 mr-1">
        {ethConnectionBtn}
      </Col>
      </div>
    </Row>
  );
}