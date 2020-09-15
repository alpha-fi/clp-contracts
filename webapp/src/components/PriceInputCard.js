import React from "react";

import Row from 'react-bootstrap/Row';
import Col from 'react-bootstrap/Col';
import Button from 'react-bootstrap/Button';

import { BsCaretDownFill } from "react-icons/bs";

export default function PriceInputCard(props) {
  return (
    <>
      <div className="border py-3 bg-white" style={{ 'borderRadius': '15px', 'boxShadow': '0px 1px 0px 0px rgba(9,30,66,.25)' }}>
        <label className="ml-4 mb-1 mt-0"><small className="text-secondary">{props.label}</small></label>
        <Row className="px-2">
          <Col>
            <div className="input-group-lg mb-1">
              <input type="text" className="form-control border-0" placeholder="0.0"/>
            </div>
          </Col>
          <Col md={4} sm={3} className="d-flex flex-row-reverse align-items-center mr-2">
            <div>
            <Button size="sm" variant="outline-secondary"><span className="align-middle">NEAR{' '}<BsCaretDownFill/></span></Button>
            </div>
          </Col>
        </Row>
      </div>
    </>
  );
}
