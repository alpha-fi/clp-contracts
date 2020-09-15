import React from "react";

import { login, logout } from '../utils'

import Button from 'react-bootstrap/Button';

export default function WalletConnectionButtons() {
  if (!window.walletConnection.isSignedIn()) {
    return (
      <Button variant="warning" onClick={login} className="py-2 mr-1 mb-1">Connect to NEAR wallet</Button>
  	)
  }
  return (
    <Button variant="warning" onClick={logout} className="py-2 mr-1 mb-1">Log out</Button>
  );
}