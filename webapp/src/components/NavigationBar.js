import React from "react";

import Navbar from 'react-bootstrap/Navbar';
import Nav from 'react-bootstrap/Nav';
import Container from 'react-bootstrap/Container';

import ThemeSwitcher from "./ThemeSwitcher";
import WalletConnectionButtons from "./WalletConnectionButtons";
import AboutButton from "./AboutButton";
import SettingsButton from "./SettingsButton";

import { FaGithub } from "react-icons/fa";

export default function NavigationBar() {
  return (
    <>
      <Navbar expand="sm" className="py-2">
        <Container>
          <Navbar.Brand href="" className="pr-4 pl-3"><strong>NEARswap</strong></Navbar.Brand>
          <Navbar.Toggle aria-controls="pages" />
          <Navbar.Collapse id="pages">
            <Nav className="mr-auto">
              <Nav.Link><AboutButton/></Nav.Link>
              <Nav.Link><ThemeSwitcher/></Nav.Link>
              <Nav.Link href="https://github.com/robert-zaremba/near-clp"><FaGithub/></Nav.Link>
            </Nav>
            <WalletConnectionButtons/>
            <SettingsButton/>
          </Navbar.Collapse>
        </Container>
      </Navbar>
    </>
  );
}
