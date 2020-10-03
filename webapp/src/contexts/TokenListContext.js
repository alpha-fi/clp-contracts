import React, { createContext, useContext, useReducer, useEffect } from 'react';
import { useThunkReducer } from 'react-hook-thunk-reducer';

import { Web3Context } from "../contexts/Web3Context";

import { getERC20Balance } from "../services/web3utils";

import { default as testTokenList } from '../assets/test-token-near.json';

const initialState = {
  tokenList: window.config.defaultTokenList,
}
const TokenListContext = createContext(initialState);

const { Provider } = TokenListContext;


const updateEthBalances = (tokenList, w3, ethAccount) => {
  let tl = tokenList;
  tl.tokens.map(async (token, index) => {
    if (token.type === "ERC-20" && w3 && ethAccount && token.address !== "") {
      token.balance = await getERC20Balance(w3, ethAccount, token.address);
    }
  });
  return tl;
}

function reducer(state, action) {
  let newState = {...state}
  switch (action.type) {
    case 'SET_TOKEN_BALANCE':
      for (let token of newState.tokenList.tokens) {
        if (token.name == action.payload.name) {
          token.balance = action.payload.balance;
          return newState;
        }
      }
      // let updatedNearTokenList = updateNearBalances(state.tokenList);
      // return { tokenList: updatedNearTokenList };
      break;

    // case 'FETCH_NEAR_BALANCES':
    //   let updatedNearTokenList = updateNearBalances(state.tokenList);
    //   return { tokenList: updatedNearTokenList };
    //   break;

    // case 'FETCH_ETH_BALANCES':
    //   let updatedEthTokenList = updateEthBalances(state.tokenList, action.payload.w3.web3, action.payload.ethAccount);
    //   return { tokenList: updatedEthTokenList };
    //   break;

    default:
      throw new Error();
  };
}

const TokenListProvider = ({ children }) => {

  const [state, dispatch] = useReducer(reducer, initialState);

  // const [state, dispatch] = useThunkReducer((state, action) => {
  //   switch (action.type) {
  //     case 'SET_TOKEN_BALANCE':
  //       for (let token of state.tokenList.tokens) {
  //         if (token.name == action.payload.name) {
  //           token.balance = action.payload.balance;
  //           return;
  //         }
  //       }
  //       // let updatedNearTokenList = updateNearBalances(state.tokenList);
  //       // return { tokenList: updatedNearTokenList };
  //       break;

  //     // case 'FETCH_NEAR_BALANCES':
  //     //   let updatedNearTokenList = updateNearBalances(state.tokenList);
  //     //   return { tokenList: updatedNearTokenList };
  //     //   break;

  //     case 'FETCH_ETH_BALANCES':
  //       let updatedEthTokenList = updateEthBalances(state.tokenList, action.payload.w3.web3, action.payload.ethAccount);
  //       return { tokenList: updatedEthTokenList };
  //       break;

  //     default:
  //       throw new Error();
  //   };
  // }, initialState);

  // Web3 state
  const web3State = useContext(Web3Context);
  const { currentUser, web3Modal } = web3State;

  //----------------------
  // useEffect(() => {

  //   // Update balances of NEAR tokens
  //   if (window.walletConnection.isSignedIn()) {
  //     //updateNearBalances(this.context);
  //     // try {
  //     //   // Inject token balances in token list using wallet information
  //     //   dispatch({
  //     //     type: 'FETCH_NEAR_BALANCES',
  //     //     payload: {
  //     //       w3: web3Modal,
  //     //       ethAccount: currentUser
  //     //     }
  //     //   });
  //     // } catch (e) {
  //     //   console.error(`Could not inject NEAR balances`);
  //     // }
  //   }

  //   // Update balances of ETH tokens
  //   if (currentUser) {
  //     try {
  //       // Inject token balances in token list using wallet information
  //       dispatch({
  //         type: 'FETCH_ETH_BALANCES',
  //         payload: {
  //           w3: web3Modal,
  //           ethAccount: currentUser
  //         }
  //       });
  //     } catch (e) {
  //       console.error(`Could not inject ETH balances`);
  //     }
  //   }

  // }, [currentUser, web3Modal]);

  return <Provider value={{ state, dispatch }}>{children}</Provider>;
}

export { TokenListContext, TokenListProvider };
