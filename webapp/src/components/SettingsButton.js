import React, {useContext} from "react";

import { Web3Context, signInWithWeb3 } from '../contexts/Web3Context';

import Button from 'react-bootstrap/Button';
import Dropdown from 'react-bootstrap/Dropdown';

import { FaCog } from "react-icons/fa";

export default function SettingsButton() {

  // Web3 state
  const web3State = useContext(Web3Context);
  const { currentUser } = web3State;

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
  return (
    <>
      <Dropdown alignRight>
        <Dropdown.Toggle variant="warning" className="py-2 mr-1 mb-1">
          <FaCog/>
        </Dropdown.Toggle>
        <Dropdown.Menu className="mt-2">
          {(() => {
            if (window.walletConnection.isSignedIn()) {
              return <Dropdown.Item href={(process.env.REACT_APP_NEAR_ADDRESS_EXPLORER) + window.accountId}>Connected to NEAR: {window.accountId}</Dropdown.Item>;
            }
          })()}
          {(() => {
            if (currentUser) {
              return <Dropdown.Item href={(process.env.REACT_APP_ETH_ADDRESS_EXPLORER) + currentUser}>Connected to Ethereum: {currentUser.substring(0,5)}...{currentUser.substr(currentUser.length-5)}</Dropdown.Item>
            }
          })()}
          <Dropdown.Divider />
        </Dropdown.Menu>
      </Dropdown>
    </>
  );
}