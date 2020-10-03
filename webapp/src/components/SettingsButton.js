import React, {useContext} from "react";

import { Web3Context } from '../contexts/Web3Context';

import WalletConnectionButtons from "./WalletConnectionButtons";

import Dropdown from 'react-bootstrap/Dropdown';

import { FaCog } from "react-icons/fa";

export default function SettingsButton() {
  
  // Web3 state
  const web3State = useContext(Web3Context);
  const { currentUser, web3Modal } = web3State;

  // Initialize connection information
  let nearConnected, ethConnected = "";
  if (window.walletConnection.isSignedIn()) {
    nearConnected = <Dropdown.Item href={(window.config.nearAddressPrefix) + window.accountId}>Connected to NEAR: {window.accountId}</Dropdown.Item>;
  }
  if (currentUser) {
    ethConnected = <Dropdown.Item href={(window.config.ethWalletExplorerPrefix) + currentUser}>Connected to Ethereum: {currentUser.substring(0,5)}...{currentUser.substr(currentUser.length-5)}</Dropdown.Item>
  }

  return (
    <>
      <p className="align-middle pr-3 mb-0">
      {window.walletConnection.isSignedIn()
        ? window.accountId
        : "Not connected"}
      </p>
      <Dropdown alignRight>
        <Dropdown.Toggle variant="warning" className="py-2 mr-1 mb-1">
          <FaCog/>
        </Dropdown.Toggle>
        <Dropdown.Menu className="mt-2">
          <WalletConnectionButtons/>
          <Dropdown.Divider />
          {nearConnected}
          {ethConnected}
          <Dropdown.Divider />
          <Dropdown.Item href="https://near-examples.github.io/erc20-to-nep21/">Convert ERC-20 to NEP-21 via Rainbow Bridge</Dropdown.Item>
          <Dropdown.Item className="text-secondary">Contract: {window.contract.contractId}</Dropdown.Item>
        </Dropdown.Menu>
      </Dropdown>
    </>
  );
}