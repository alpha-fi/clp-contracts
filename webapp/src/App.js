import 'regenerator-runtime/runtime'
import React from 'react'
import './global.css'

import { Web3Context } from './contexts/Web3Context';

import Container from 'react-bootstrap/Container';
import Nav from 'react-bootstrap/Nav';
import Tabs from 'react-bootstrap/Tabs';
import Tab from 'react-bootstrap/Tab';
import Button from 'react-bootstrap/Button';
import Card from 'react-bootstrap/Card';
import Row from 'react-bootstrap/Row';
import Col from 'react-bootstrap/Col';

import { BsArrowLeftRight } from "react-icons/bs";
import { BsDropletFill } from "react-icons/bs";

import NavigationBar from "./components/NavigationBar";
import SwapTab from "./components/SwapTab";
import PoolTab from "./components/PoolTab";
import CurrencySelectionModal from "./components/CurrencySelectionModal";
import Notification from "./components/Notification";

import styled from "@emotion/styled";
const Wrapper = styled("div")`
  height: 100vh;
  overflow-y: auto;
  background: ${props => props.theme.background};
  color: ${props => props.theme.body};
  .navbar-brand, .nav-link {
    color: ${props => props.theme.body} !important;
  }
  .btn-warning {
    background-color: ${props => props.theme.buttonColor} !important;
  }
  .nav-link.active {
    text-shadow: 1px 1px ${props => props.theme.navTabShadow};
  }
  .navbar-toggler {
    background-color: ${props => props.theme.navbarToggler};
  }
  .btn-warning:focus {
    box-shadow: 0 0 0 .2rem ${props => props.theme.buttonBorder} !important;
  }
`;

export default function App() {

  return (
    <Wrapper>
      <link
        rel="stylesheet"
        href="https://maxcdn.bootstrapcdn.com/bootstrap/4.5.2/css/bootstrap.min.css"
        integrity="sha384-JcKb8q3iqJ61gNV9KGb8thSsNjpSL0n8PARn9HuZOnIxN0hoP+VmmDGMN5t9UJ0Z"
        crossOrigin="anonymous"
      />
      <NavigationBar/>
      <Container className="pb-2">
        <Row className="d-flex justify-content-center">
          <Col md={8} lg={6}>
            <Card className="border-0 bg-transparent mb-4">
              <Card.Body>
                <Notification/>
                <Tab.Container defaultActiveKey="swap">
                  <Nav justify className="border-0 mb-3">
                    <Nav.Link eventKey="swap"><BsArrowLeftRight/>{' '}Swap</Nav.Link>
                    <Nav.Link eventKey="pool"><BsDropletFill/>{' '}Pool</Nav.Link>
                  </Nav>
                  <Tab.Content animation="true">
                    <Tab.Pane eventKey="swap"><SwapTab/></Tab.Pane>
                    <Tab.Pane eventKey="pool"><PoolTab/></Tab.Pane>
                  </Tab.Content>
                </Tab.Container>
                <br/>
              </Card.Body>
            </Card>
          </Col>
        </Row>
      </Container>
      <CurrencySelectionModal/>
    </Wrapper>
  )
}
