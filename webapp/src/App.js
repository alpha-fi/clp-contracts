import 'regenerator-runtime/runtime'
import React from 'react'
import './global.css'

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

import getConfig from './config'
const { networkId } = getConfig(process.env.NODE_ENV || 'development')

import NavigationBar from "./components/NavigationBar";
import SwapTab from "./components/SwapTab";
import PoolTab from "./components/PoolTab";
import CurrencySelectionModal from "./components/CurrencySelectionModal";

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

  // after submitting the form, we want to show Notification
  const [showNotification, setShowNotification] = React.useState(false)

  // The useEffect hook can be used to fire side-effects during render
  // Learn more: https://reactjs.org/docs/hooks-intro.html
  React.useEffect(
    () => {
      // in this case, we only care to query the contract when signed in
      if (window.walletConnection.isSignedIn()) {

        // window.contract is set by initContract in index.js
        window.contract.get_greeting({ account_id: window.accountId })
          .then(greetingFromContract => {
            set_greeting(greetingFromContract)
          })
      }
    },

    // The second argument to useEffect tells React when to re-run the effect
    // Use an empty array to specify "only run on first render"
    // This works because signing into NEAR Wallet reloads the page
    []
  )

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
          <Col md={7} lg={6}>
            <Card className="border-0 bg-transparent">
              <Card.Body>
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
      {showNotification && <Notification />}
    </Wrapper>
  )
}

// this component gets rendered by App after the form is submitted
function Notification() {
  const urlPrefix = `https://explorer.${networkId}.near.org/accounts`
  return (
    <aside>
      <a target="_blank" rel="noreferrer" href={`${urlPrefix}/${window.accountId}`}>
        {window.accountId}
      </a>
      {' '}
      called method: 'set_greeting' in contract:
      {' '}
      <a target="_blank" rel="noreferrer" href={`${urlPrefix}/${window.contract.contractId}`}>
        {window.contract.contractId}
      </a>
      <footer>
        <div>✔ Succeeded</div>
        <div>Just now</div>
      </footer>
    </aside>
  )
}
