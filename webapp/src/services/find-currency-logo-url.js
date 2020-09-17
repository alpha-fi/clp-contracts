export default function findCurrencyLogoUrl(newTokenIndex, tokenList) {
  let hasImage = tokenList.tokens[newTokenIndex].hasOwnProperty("logoURI");

  // Only display image on button if it exists
  if (hasImage) {
    if (tokenList.tokens[newTokenIndex].logoURI.startsWith("ipfs://")) {
      return (process.env.REACT_APP_IPFS_GATEWAY + tokenList.tokens[newTokenIndex].logoURI.substring(7));
    } else {
      return tokenList.tokens[newTokenIndex].logoURI;
    }
  }

  return "";
}