const white = "#FFFFFF";
const black = "#000000";
const darkBackground = "#0b0d0f";
const lightGray = "#F8F8F9";
const darkGray = "#495057"
const darkBlue = "#262d35";	
const offWhite = '#dee2e6';
const lightShadow = 'rgba(9,30,66,.25)';
const yellow = '#F0EC74';
const darkerYellow = 'rgba(222,170,12,.5)';
const lightYellow = '#faf9d1';
const seaGreen = '#8FD6BD';
const darkerSeaGreen = '#2f8164';

const themeLight = {
  background: lightGray,
  body: black,
  cardBackground: white,
  cardBorder: offWhite,
  cardShadow: lightShadow,
  buttonColor: yellow,
  textInput: darkGray,
  navTabShadow: yellow,
  navbarToggler: "transparent",
  buttonBorder: darkerYellow,
  hr: "rgba(0,0,0,.1)",
  coloredText: darkerYellow,
  coloredTextShadow: lightYellow,
};

const themeDark = {
  background: darkBackground,
  body: white,
  cardBackground: darkBlue,
  cardBorder: darkBlue,
  cardShadow: black,
  buttonColor: seaGreen,
  textInput: offWhite,
  navTabShadow: darkGray,
  navbarToggler: lightGray,
  buttonBorder: darkerSeaGreen,
  hr: darkGray,
  coloredText: seaGreen,
  coloredTextShadow: black,
};

const theme = mode => (mode === "dark" ? themeDark : themeLight);

export default theme;