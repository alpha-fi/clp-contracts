import React, { createContext, useContext, useReducer, useEffect } from 'react';
import { useThunkReducer } from 'react-hook-thunk-reducer';

import { Web3Context } from "../contexts/Web3Context";

import { getERC20Balance } from "../services/web3utils";

import { default as testTokenList } from '../assets/test-token-near.json';
import produce from 'immer';

const initialState =
//{tokenList: window.config.defaultTokenList}
{
  "name": "Default List",
  "tokens": [
    {
      "name": "NEAR",
      "type": "Native token",
      "address": "",
      "symbol": "NEAR",
      "decimals": 24,
      "logoURI": "https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/near/info/logo.png"
    },
    {
      "name": "GOLD",
      "type": "NEP-21",
      "address": "gold.nearswap.testnet",
      "symbol": "GOLD",
      "decimals": 24,
      "logoURI": "https://user-images.githubusercontent.com/26249903/94925431-d3f07700-04dc-11eb-8189-68fbd65e7738.png"
    },
    // {
    //   "name": "Near DAI",
    //   "type": "NEP-21",
    //   "address": "ndai.nearswap.testnet",
    //   "symbol": "nDAI",
    //   "decimals": 24,
    //   "logoURI": "https://user-images.githubusercontent.com/26249903/94925506-f3879f80-04dc-11eb-83cc-d480ef4b91cf.png"
    // },
    {
      "name": "Basic Attention Token",
      "type": "NEP-21",
      "address": "bat.nearswap.testnet",
      "symbol": "nBAT",
      "decimals": 24,
      "logoURI": "https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/ethereum/assets/0x0D8775F648430679A709E98d2b0Cb6250d2887EF/logo.png"
    },
    {
      "name": "USD24 Token",
      "type": "NEP-21",
      "address": "usd24.nearswap.testnet",
      "symbol": "USD24",
      "decimals": 24,
      "logoURI": "https://user-images.githubusercontent.com/26249903/94925506-f3879f80-04dc-11eb-83cc-d480ef4b91cf.png"
    },
    {
      "name": "Tether Token",
      "type": "NEP-21",
      "address": "tether29",
      "symbol": "nUSDT",
      "decimals": 24,
      "logoURI": "https://user-images.githubusercontent.com/26249903/94925313-a0155180-04dc-11eb-9064-cd2f27cd18f8.png"
    },
    {
      "name": "Wrapped ETH Token",
      "type": "NEP-21",
      "address": "weth",
      "symbol": "nwETH",
      "decimals": 24,
      "logoURI": "https://ethereum.org/static/6b935ac0e6194247347855dc3d328e83/31987/eth-diamond-black.png"
    },
    {
      "name": "Abundance Token",
      "type": "NEP-21",
      "address": "mint_with_json-workaround-for-mintablefuntoken.chad",
      "symbol": "nABND",
      "decimals": 24,
      "logoURI": ""
    }
  ]
}

const TokenListContext = createContext(initialState);

const { Provider } = TokenListContext;



//-------------------------------------------
//-------------------------------------------
//-------------------------------------------
const TokenListProvider = ({ children }) => {

  function reducer(state, action) {

    switch (action.type) {

      case 'SET_TOKEN_LIST_BALANCE':
        return produce(state, draft => {
          draft.tokens[action.payload.tokenIndex].balance = action.payload.balance;
        })

      // case 'FETCH_NEAR_BALANCES':
      //   let updatedNearTokenList = updateNearBalances(state.tokens);
      //   return { tokenList: updatedNearTokenList };
      //   break;

      // case 'FETCH_ETH_BALANCES':
      //   let updatedEthTokenList = updateEthBalances(state.tokens, action.payload.w3.web3, action.payload.ethAccount);
      //   return { tokenList: updatedEthTokenList };
      //   break;

      default:
        throw new Error();
    };
  }

  const [state, dispatch] = useReducer(reducer, initialState);

  // const [state, dispatch] = useThunkReducer((state, action) => {
  //   switch (action.type) {
  //     case 'SET_TOKEN_BALANCE':
  //       for (let token of state.tokens) {
  //         if (token.name == action.payload.name) {
  //           token.balance = action.payload.balance;
  //           return;
  //         }
  //       }
  //       // let updatedNearTokenList = updateNearBalances(state.tokens);
  //       // return { tokenList: updatedNearTokenList };
  //       break;

  //     // case 'FETCH_NEAR_BALANCES':
  //     //   let updatedNearTokenList = updateNearBalances(state.tokens);
  //     //   return { tokenList: updatedNearTokenList };
  //     //   break;

  //     case 'FETCH_ETH_BALANCES':
  //       let updatedEthTokenList = updateEthBalances(state.tokens, action.payload.w3.web3, action.payload.ethAccount);
  //       return { tokenList: updatedEthTokenList };
  //       break;

  //     default:
  //       throw new Error();
  //   };
  // }, initialState);

  // Web3 state
  const web3State = useContext(Web3Context);
  const { currentUser, web3Modal } = web3State;

  // Token list state
  const tokenListState = useContext(TokenListContext);

  return <Provider value={{ state, dispatch }}>{children}</Provider>;
}

export { TokenListContext, TokenListProvider };
