export default function findCurrencyLogoUrl(newTokenIndex, tokens) {
  let hasImage = tokens[newTokenIndex].hasOwnProperty("logoURI");

  // Only display image on button if it exists
  if (hasImage) {
    if (tokens[newTokenIndex].logoURI.startsWith("ipfs://")) {
      return (window.config.ipfsPrefix + tokens[newTokenIndex].logoURI.substring(7));
    } else {
      return tokens[newTokenIndex].logoURI;
    }
  }

  return "";
}