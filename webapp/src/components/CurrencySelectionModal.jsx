import React, { useContext } from "react";

import findCurrencyLogoUrl from "../services/find-currency-logo-url";

import Modal from 'react-bootstrap/Modal';
import Table from 'react-bootstrap/Table';
import Button from 'react-bootstrap/Button';

import { GlobalContext } from "../contexts/GlobalContext";
import { TokenListContext } from "../contexts/TokenListContext";

import { CurrencyTable } from "./CurrencyTable";

import styled from "@emotion/styled";
const LimitedHeightTable = styled("div")`
  height: 50vh;
  overflow-y: scroll;
  overflow-x: none;
`;

export default function CurrencySelectionModal(props) {

  // Global state
  const globalState = useContext(GlobalContext);
  const { dispatch } = globalState;

  // Token list state
  const tokenListState = useContext(TokenListContext);
  
  const toggleModalVisibility = () => {
    dispatch({ type: 'TOGGLE_CURRENCY_SELECTION_MODAL' });
  };

  return (
    <>
      <Modal show={globalState.state.currencySelectionModal.isVisible} onHide={toggleModalVisibility}>
        <Modal.Header closeButton>
          <Modal.Title>Select currency</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          <LimitedHeightTable>
            <Table hover>
              <thead>
                <tr>
                  <th className="border-0"></th>
                  <th className="border-0">Symbol</th>
                 <th className="border-0">Name</th>
                </tr>
              </thead>
              <tbody>
                <CurrencyTable/>
              </tbody>
            </Table>
          </LimitedHeightTable>
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={toggleModalVisibility}>
            Close
          </Button>
        </Modal.Footer>
      </Modal>
    </>
  );
}
