import React, {createContext, useReducer} from 'react';

import { default as testTokenList } from '../assets/test-token-list.json';

const initialState = {
  tokenList: testTokenList
}

const TokenListContext = createContext(initialState);
const { Provider } = TokenListContext;

const TokenListProvider = ( { children } ) => {
  const [state, dispatch] = useReducer((state, action) => {
    switch(action.type) {
      default:
        throw new Error();
    };
  }, initialState);

  return <Provider value={{ state, dispatch }}>{children}</Provider>;
}

export { TokenListContext, TokenListProvider };
