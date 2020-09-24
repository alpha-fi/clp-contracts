import React, { createContext, useContext, useEffect } from 'react';
import { useThunkReducer } from 'react-hook-thunk-reducer';

import { Web3Context } from "../contexts/Web3Context";

import { getERC20Balance } from "../services/web3utils";

import { default as testTokenList } from '../assets/test-token-list.json';

const initialState = {
  tokenList: testTokenList
}

const updateBalances = (tokenList, w3, ethAccount) => {
  let tl = tokenList;
  tl.tokens.map(async (token, index) => {
    if (token.type === "ERC-20" && w3 && ethAccount && token.address !== "") {
      // token.balance = await getERC20Balance(w3, ethAccount, token.address); // FIX
    }
    if (token.type === "Native token") {
      token.balance = (await window.walletConnection.account().getAccountBalance()).available / 1000000000000000000000000 ;
    }
  });
  return tl;
}

const TokenListContext = createContext(initialState);
const { Provider } = TokenListContext;

const TokenListProvider = ( { children } ) => {

  const [state, dispatch] = useThunkReducer((state, action) => {
    switch(action.type) {
      case 'FETCH_BALANCES':
        let updatedTokenList = updateBalances(state.tokenList, action.payload.w3.web3, action.payload.ethAccount);
        return { tokenList: updatedTokenList };
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
    if (window.walletConnection.isSignedIn() || currentUser) {
      try {
        // Inject token balances in token list using wallet information
        dispatch({
          type: 'FETCH_BALANCES',
          payload: {
            w3: web3Modal,
            ethAccount: currentUser
          }
        });
      } catch (e) {
        console.error(`Could not inject balances`);
      }
    }

  }, [currentUser, web3Modal]);

  return <Provider value={{ state, dispatch }}>{children}</Provider>;
}

export { TokenListContext, TokenListProvider };
