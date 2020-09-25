import React, {createContext, useReducer} from 'react';

const initialState = { 
  swap: {
    from: {
      amount: "",     // Amount of tokens
      symbol: "",     // Symbol of token
      type: "",       // Native token (NEAR or ETH), ERC-20, NEP-21, ...
      logoUrl: "",    // Address of token logo image
      tokenIndex: 1,  // Index of token within token list
      isValid: false
    },
    to: {
      amount: "",
      symbol: "",
      type: "",
      logoUrl: "",
      tokenIndex: 0,
      isValid: false
    }
  },
  pool: {
    input1: {
      amount: "",
      symbol: "",
      type: "",
      logoUrl: "",
      tokenIndex: 2,
      isValid: false
    },
    input2: {
      amount: "",
      symbol: "",
      type: "",
      logoUrl: "",
      tokenIndex: 0,
      isValid: false
    }
  },
  currencySelectionModal: {
    isVisible: false,
    selectedInput: ""
  }
};

const InputsContext = createContext(initialState);
const { Provider } = InputsContext;

const InputsProvider = ( { children } ) => {
  const [state, dispatch] = useReducer((state, action) => {
    switch(action.type) {
      case 'SET_FROM_AMOUNT':
        return { ...state, swap: { from: { 
          amount: action.payload.amount,
          symbol: state.swap.from.symbol,
          type: state.swap.from.type,
          logoUrl: state.swap.from.logoUrl,
          tokenIndex: state.swap.from.tokenIndex,
          isValid: state.swap.from.isValid,
        }, to: state.swap.to }};
      case 'SET_TO_AMOUNT':
        return { ...state, swap: { to: { 
          amount: action.payload.amount,
          symbol: state.swap.to.symbol,
          type: state.swap.to.type,
          logoUrl: state.swap.to.logoUrl,
          tokenIndex: state.swap.to.tokenIndex,
          isValid: state.swap.to.isValid,
        }, from: state.swap.from }};
      case 'SET_INPUT1_AMOUNT':
        return { ...state, pool: { input1: { 
          amount: action.payload.amount,
          symbol: state.pool.input1.symbol,
          type: state.pool.input1.type,
          logoUrl: state.pool.input1.logoUrl,
          tokenIndex: state.pool.input1.tokenIndex,
          isValid: state.pool.input1.isValid,
        }, input2: state.pool.input2 }};
      case 'SET_INPUT2_AMOUNT':
        return { ...state, pool: { input2: { 
          amount: action.payload.amount,
          symbol: state.pool.input2.symbol,
          type: state.pool.input2.type,
          logoUrl: state.pool.input2.logoUrl,
          tokenIndex: state.pool.input2.tokenIndex,
          isValid: state.pool.input2.isValid,
        }, input1: state.pool.input1 }};
      case 'UPDATE_FROM_SELECTED_CURRENCY':
        return { ...state, swap: { from: {
          amount: state.swap.from.amount,
          symbol: action.payload.symbol,
          type: action.payload.type,
          logoUrl: action.payload.logoUrl,
          tokenIndex: state.swap.from.tokenIndex,
          isValid: action.payload.isValid,
        }, to: state.swap.to }, currencySelectionModal: { isVisible: false }};
      case 'UPDATE_TO_SELECTED_CURRENCY':
        return { ...state, swap: { to: {
          amount: state.swap.to.amount,
          symbol: action.payload.symbol,
          type: action.payload.type,
          logoUrl: action.payload.logoUrl,
          tokenIndex: state.swap.to.tokenIndex,
          isValid: action.payload.isValid,
        }, from: state.swap.from }, currencySelectionModal: { isVisible: false }};
      case 'UPDATE_INPUT1_SELECTED_CURRENCY':
        return { ...state, pool: { input1: {
          amount: state.pool.input1.amount,
          symbol: action.payload.symbol,
          type: action.payload.type,
          logoUrl: action.payload.logoUrl,
          tokenIndex: state.pool.input1.tokenIndex,
          isValid: action.payload.isValid,
        }, input2: state.pool.input2 }, currencySelectionModal: { isVisible: false }};
      case 'UPDATE_INPUT2_SELECTED_CURRENCY':
        return { ...state, pool: { input2: {
          amount: state.pool.input2.amount,
          symbol: action.payload.symbol,
          type: action.payload.type,
          logoUrl: action.payload.logoUrl,
          tokenIndex: state.pool.input2.tokenIndex,
          isValid: action.payload.isValid,
        }, input1: state.pool.input1 }, currencySelectionModal: { isVisible: false }};
      case 'TOGGLE_CURRENCY_SELECTION_MODAL':
        return { ...state, currencySelectionModal: { isVisible: !state.currencySelectionModal.isVisible, selectedInput: state.currencySelectionModal.selectedInput } };
      case 'SET_CURRENCY_SELECTION_INPUT':
        return { ...state, currencySelectionModal: { selectedInput: action.payload.input, isVisible: !state.isVisible } };
      default:
        throw new Error();
    };
  }, initialState);

  return <Provider value={{ state, dispatch }}>{children}</Provider>;
}

export { InputsContext, InputsProvider };
