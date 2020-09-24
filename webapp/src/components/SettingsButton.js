import React, {useContext} from "react";

import { Web3Context } from '../contexts/Web3Context';

import Dropdown from 'react-bootstrap/Dropdown';

import { FaCog } from "react-icons/fa";

export default function SettingsButton() {
  
  // Web3 state
  const web3State = useContext(Web3Context);
  const { currentUser, web3Modal } = web3State;

  // Return disabled settings button if no wallets connected
  if (!window.walletConnection.isSignedIn() && !currentUser) {
    return (
      <>
        <Dropdown alignRight>
          <Dropdown.Toggle disabled variant="warning"className="py-2 mr-1 mb-1">
            <FaCog/>
          </Dropdown.Toggle>
        </Dropdown>
      </>
    )
  }

  // Initialize connection information
  let nearConnected, ethConnected = "";
  if (window.walletConnection.isSignedIn()) {
    nearConnected = <Dropdown.Item href={(process.env.REACT_APP_NEAR_ADDRESS_EXPLORER) + window.accountId}>Connected to NEAR: {window.accountId}</Dropdown.Item>;
  }
  if (currentUser) {
    ethConnected = <Dropdown.Item href={(process.env.REACT_APP_ETH_ADDRESS_EXPLORER) + currentUser}>Connected to Ethereum: {currentUser.substring(0,5)}...{currentUser.substr(currentUser.length-5)}</Dropdown.Item>
  }

  return (
    <>
      <Dropdown alignRight>
        <Dropdown.Toggle variant="warning" className="py-2 mr-1 mb-1">
          <FaCog/>
        </Dropdown.Toggle>
        <Dropdown.Menu className="mt-2">
          {nearConnected}
          {ethConnected}
          {/*<Dropdown.Divider />*/}
        </Dropdown.Menu>
      </Dropdown>
    </>
  );
}