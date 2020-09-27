import React from "react";

import Navbar from 'react-bootstrap/Navbar';
import Nav from 'react-bootstrap/Nav';
import Container from 'react-bootstrap/Container';

import ThemeSwitcher from "./ThemeSwitcher";
import AboutButton from "./AboutButton";
import SettingsButton from "./SettingsButton";

import { FaGithub } from "react-icons/fa";

export default function NavigationBar() {
  return (
    <>
      <Navbar className="py-2">
        <Container>
          <Navbar.Brand href="" className="pl-3"><strong>NEARswap</strong></Navbar.Brand>
          <Nav className="mr-auto">
            <Nav.Link><AboutButton/></Nav.Link>
            <Nav.Link><ThemeSwitcher/></Nav.Link>
            <Nav.Link href="https://github.com/robert-zaremba/near-clp"><FaGithub/></Nav.Link>
          </Nav>
          <SettingsButton/>
        </Container>
      </Navbar>
    </>
  );
}
