var HDWalletProvider = require("truffle-hdwallet-provider");
var mnemonic = "spoon flip anxiety bread hint quit analyst hip zebra hint emotion hat";
module.exports = {
  networks: {
    development: {
      host: "localhost",
      port: 9545,
      network_id: "*", // Match any network id
      gas: 5000000
    },
    rinkebyInfura: {
     provider: function() {
      return new HDWalletProvider(mnemonic, "https://rinkeby.infura.io/v3/0a9ed154f20140a8b223c2ca16998d52");
     },
     network_id: 4,
     gas: 5500000,
     gasPrice: 30000000000,
     timeoutBlocks: 50,
     websockets: true,
     from: ""
    }
  },
  compilers: {
    solc: {
   	  version: "0.5.16",
      settings: {
        optimizer: {
          enabled: true, // Default: false
          runs: 200      // Default: 200
        },
      }
    }
  }
};
