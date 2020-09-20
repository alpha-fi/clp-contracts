import React, {createContext, useReducer} from 'react';

const initialState = { 
  swap: {
    from: {
      amount: "",     // Amount of tokens
      symbol: "",     // Symbol of token
      logoUrl: "",    // Address of token logo image
      tokenIndex: 0   // Index of token within token list
    },
    to: {
      amount: "",
      symbol: "",
      logoUrl: "",
      tokenIndex: 1
    }
  },
  pool: {
    input1: {
      amount: "",
      symbol: "",
      logoUrl: "",
      tokenIndex: 0
    },
    input2: {
      amount: "",
      symbol: "NEAR",
      logoUrl: "",
      tokenIndex: 1
    }
  },
  currencySelectionModal: {
    isVisible: false,
    selectedInput: ""
  }
};

const GlobalContext = createContext(initialState);
const { Provider } = GlobalContext;

const GlobalStateProvider = ( { children } ) => {
  const [state, dispatch] = useReducer((state, action) => {
    switch(action.type) {
      case 'SET_FROM_AMOUNT':
        return { ...state, swap: { from: { 
          amount: action.payload.amount,
          symbol: state.swap.from.symbol,
          logoUrl: state.swap.from.logoUrl,
          tokenIndex: state.swap.from.tokenIndex
        }, to: state.swap.to }};
      case 'SET_TO_AMOUNT':
        return { ...state, swap: { to: { 
          amount: action.payload.amount,
          symbol: state.swap.to.symbol,
          logoUrl: state.swap.to.logoUrl,
          tokenIndex: state.swap.to.tokenIndex
        }, from: state.swap.from }};
      case 'SET_INPUT1_AMOUNT':
        return { ...state, pool: { input1: { 
          amount: action.payload.amount,
          symbol: state.pool.input1.symbol,
          logoUrl: state.pool.input1.logoUrl,
          tokenIndex: state.pool.input1.tokenIndex
        }, input2: state.pool.input2 }};
      case 'SET_INPUT2_AMOUNT':
        return { ...state, pool: { input2: { 
          amount: action.payload.amount,
          symbol: state.pool.input2.symbol,
          logoUrl: state.pool.input2.logoUrl,
          tokenIndex: state.pool.input2.tokenIndex
        }, input1: state.pool.input1 }};
      case 'UPDATE_FROM_SELECTED_CURRENCY':
        return { ...state, swap: { from: {
          amount: state.swap.from.amount,
          symbol: action.payload.symbol,
          logoUrl: action.payload.logoUrl,
          tokenIndex: state.swap.from.tokenIndex
        }, to: state.swap.to }, currencySelectionModal: { isVisible: false }};
      case 'UPDATE_TO_SELECTED_CURRENCY':
        return { ...state, swap: { to: {
          amount: state.swap.to.amount,
          symbol: action.payload.symbol,
          logoUrl: action.payload.logoUrl,
          tokenIndex: state.swap.to.tokenIndex
        }, from: state.swap.from }, currencySelectionModal: { isVisible: false }};
      case 'UPDATE_INPUT1_SELECTED_CURRENCY':
        return { ...state, pool: { input1: {
          amount: state.pool.input1.amount,
          symbol: action.payload.symbol,
          logoUrl: action.payload.logoUrl,
          tokenIndex: state.pool.input1.tokenIndex
        }, input2: state.pool.input2 }, currencySelectionModal: { isVisible: false }};
      case 'UPDATE_INPUT2_SELECTED_CURRENCY':
        return { ...state, pool: { input2: {
          amount: state.pool.input2.amount,
          symbol: state.pool.input2.symbol,
          logoUrl: action.payload.logoUrl,
          tokenIndex: state.pool.input2.tokenIndex
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

export { GlobalContext, GlobalStateProvider };
