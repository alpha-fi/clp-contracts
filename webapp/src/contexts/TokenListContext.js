import React, { createContext, useContext, useEffect } from 'react';
import { useThunkReducer } from 'react-hook-thunk-reducer';

import { Web3Context } from "../contexts/Web3Context";

import { getERC20Balance } from "../services/web3utils";

import { default as testTokenList } from '../assets/test-token-near.json';

import { getBalanceNEP } from '../services/near-nep21-util'

const initialState = {
  tokenList: window.config.defaultTokenList,
}

const updateNearBalances = (tokenList) => {
  let tl = tokenList;
  tl.tokens.map(async (token, index) => {
    if (token.type === "Native token") {
      token.balance = (await window.walletConnection.account().getAccountBalance()).available / 1000000000000000000000000 ;
    }
    if (token.type === "NEP-21") {
      token.balance = await getBalanceNEP( token.address );
    }
  });
  return tl;
}

const updateEthBalances = (tokenList, w3, ethAccount) => {
  let tl = tokenList;
  tl.tokens.map(async (token, index) => {
    if (token.type === "ERC-20" && w3 && ethAccount && token.address !== "") {
      token.balance = await getERC20Balance(w3, ethAccount, token.address);
    }
  });
  return tl;
}

const TokenListContext = createContext(initialState);
const { Provider } = TokenListContext;

const TokenListProvider = ( { children } ) => {

  const [state, dispatch] = useThunkReducer((state, action) => {
    switch(action.type) {
      case 'FETCH_NEAR_BALANCES':
        let updatedNearTokenList = updateNearBalances(state.tokenList);
        return { tokenList: updatedNearTokenList };
        break;
      case 'FETCH_ETH_BALANCES':
        let updatedEthTokenList = updateEthBalances(state.tokenList, action.payload.w3.web3, action.payload.ethAccount);
        return { tokenList: updatedEthTokenList };
        break;
      default:
        throw new Error();
    };
  }, initialState);

  // Web3 state
  const web3State = useContext(Web3Context);
  const { currentUser, web3Modal } = web3State;

  useEffect(() => {

    // Update balances of NEAR tokens
    if (window.walletConnection.isSignedIn()) {
      try {
        // Inject token balances in token list using wallet information
        dispatch({
          type: 'FETCH_NEAR_BALANCES',
          payload: {
            w3: web3Modal,
            ethAccount: currentUser
          }
        });
      } catch (e) {
        console.error(`Could not inject NEAR balances`);
      }
    }

    // Update balances of ETH tokens
    if (currentUser) {
      try {
        // Inject token balances in token list using wallet information
        dispatch({
          type: 'FETCH_ETH_BALANCES',
          payload: {
            w3: web3Modal,
            ethAccount: currentUser
          }
        });
      } catch (e) {
        console.error(`Could not inject ETH balances`);
      }
    }

  }, [currentUser, web3Modal]);

  return <Provider value={{ state, dispatch }}>{children}</Provider>;
}

export { TokenListContext, TokenListProvider };
