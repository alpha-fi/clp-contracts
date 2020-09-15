import React from "react";
import { useTheme } from "../contexts/ThemeContext";

import { RiContrastFill } from "react-icons/ri";

export default function ThemeSwitcher() {

  const themeState = useTheme();

  return (
    <span onClick={() => themeState.toggle()}><RiContrastFill/></span>
  );
}