import React, { useContext, useEffect } from "react";

import { TokenListContext } from "../contexts/TokenListContext";

import Row from 'react-bootstrap/Row';
import Col from 'react-bootstrap/Col';
import Button from 'react-bootstrap/Button';
import Image from 'react-bootstrap/Image';
import Table from 'react-bootstrap/Table';

import { CgSwap } from "react-icons/cg";

import styled from "@emotion/styled";
const Theme = styled("div")`
  background: ${props => props.theme.cardBackground};
  color: ${props => props.theme.body};
  border: 1px solid ${props => props.theme.cardBorder};
  border-radius: 20px;
  box-shadow: 0px 1px 0px 0px ${props => props.theme.cardShadow};
  .form-control:focus {
    color: ${props => props.theme.textInput};
  }
`;
const ColoredThemeText = styled("span")`
  color: ${props => props.theme.coloredText};
  text-shadow: 0px 1px 2px ${props => props.theme.coloredTextShadow};
`;

export default function PoolInfoCard(props) {

  // Token list state (used to get image)
  const tokenListState = useContext(TokenListContext);

  return (
    <>
      <Theme className="py-2 mb-2 mt-1">
        <label className="ml-4 mb-1 mt-0">
          <small><ColoredThemeText><b>NEAR-{tokenListState.state.tokenList.tokens[props.tokenIndex].symbol}</b></ColoredThemeText></small>
        </label>
        <div className="px-2 ml-3 mb-1">
          <Image rounded src={tokenListState.state.tokenList.tokens[0].logoURI} width="23px" className="mr-1"/>
          <Image rounded src={tokenListState.state.tokenList.tokens[props.tokenIndex].logoURI} width="23px" className="mr-1"/>
          NEAR <CgSwap/> {tokenListState.state.tokenList.tokens[props.tokenIndex].name} Pool
          <br/>
          <div className="my-1">
            <Row noGutters>
              <Col>
                <Table borderless size="sm" className="text-secondary mb-1" style={{ fontSize: '80%' }}>
                  <thead>
                    <tr>
                      <th></th>
                      <th>Total tokens</th>
                      <th>Price</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr>
                      <td>NEAR</td>
                      <td>8,433.2</td>
                      <td>$533.24</td>
                    </tr>
                    <tr>
                      <td><ColoredThemeText>{tokenListState.state.tokenList.tokens[props.tokenIndex].symbol}</ColoredThemeText></td>
                      <td><ColoredThemeText>425.4</ColoredThemeText></td>
                      <td style={{ fontSize: '120%' }}><ColoredThemeText>$225.02</ColoredThemeText></td>
                    </tr>
                  </tbody>
                </Table>
              </Col>
              <Col className="my-auto text-center" xs={12} sm={4}>
                <Button variant="warning" size="sm" className="mr-1 mb-1">Add liquidity</Button>
                <Button variant="warning" size="sm" className="mr-1 mb-1">Swap</Button>
                <Button variant="warning" size="sm" className="mr-1 mb-1" disabled>Details</Button>
              </Col>
            </Row>
          </div>
        </div>
      </Theme>
    </>
  );
}
