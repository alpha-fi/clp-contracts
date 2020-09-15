import React from "react";

import Button from 'react-bootstrap/Button';
import Dropdown from 'react-bootstrap/Dropdown';

import { FaCog } from "react-icons/fa";

export default function SettingsButton() {
  if (!window.walletConnection.isSignedIn()) {
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
            <Dropdown.Item>Connected: {window.accountId}</Dropdown.Item>
            <Dropdown.Divider />
            <Dropdown.Item href="">Another action</Dropdown.Item>
            <Dropdown.Item href="">Something else</Dropdown.Item>
          </Dropdown.Menu>
        </Dropdown>
      </>
  );
}