import React, { useContext } from "react";

import findCurrencyLogoUrl from "../services/find-currency-logo-url";

import Modal from 'react-bootstrap/Modal';
import Table from 'react-bootstrap/Table';
import Button from 'react-bootstrap/Button';
import InputGroup from 'react-bootstrap/InputGroup';
import FormControl from 'react-bootstrap/FormControl';

import { InputsContext } from "../contexts/InputsContext";

import { CurrencyTable } from "./CurrencyTable";

import styled from "@emotion/styled";
const LimitedHeightTable = styled("div")`
  height: 50vh;
  overflow-y: auto;
  overflow-x: none;
`;

export default function CurrencySelectionModal(props) {

  // Inputs state
  const inputs = useContext(InputsContext);
  const { dispatch } = inputs;

  const toggleModalVisibility = () => {
    dispatch({ type: 'TOGGLE_CURRENCY_SELECTION_MODAL' });
  };

  return (
    <>
      <Modal show={inputs.state.currencySelectionModal.isVisible} onHide={toggleModalVisibility}>
        <Modal.Header closeButton>
          <Modal.Title>Select currency</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          <InputGroup className="mb-3">
            <FormControl
              size="lg"
              className="rounded"
              placeholder="Enter a custom token address..."
            />
            <InputGroup.Append>
              <Button variant="warning" className="ml-2" disabled>Add</Button>
            </InputGroup.Append>
          </InputGroup>
          <LimitedHeightTable>
            <Table hover>
              <thead>
                <tr>
                  <th className="border-0"></th>
                  <th className="border-0">Name</th>
                  <th className="border-0 text-right">Balance</th>
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
