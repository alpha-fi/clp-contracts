import React, {createContext, useReducer} from 'react';

const initialState = { 
  swap: {
    from: {
      amount: "",     // Amount of tokens
      symbol: "",     // Symbol of token
      type: "",       // Native token (NEAR or ETH), ERC-20, NEP-21, ...
      logoUrl: "",    // Address of token logo image
      tokenIndex: 1,  // Index of token within token list
      address: "",
      isValid: false
    },
    to: {
      amount: "",
      symbol: "",
      type: "",
      logoUrl: "",
      tokenIndex: 2,
      address: "",
      isValid: false
    },
    needsApproval: false,
    status: "notReadyToSwap"    // possible values: notReadyToSwap, readyToSwap, swapping
  },
  pool: {
    input1: {
      amount: "",
      symbol: "",
      type: "",
      logoUrl: "",
      tokenIndex: 2,
      address: "",
      isValid: false
    },
    input2: {
      amount: "",
      symbol: "",
      type: "",
      logoUrl: "",
      tokenIndex: 0,
      address: "",
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
        return { ...state, swap: {
          from: { 
            amount: action.payload.amount,              // UPDATE amount
            symbol: state.swap.from.symbol,             // leave symbol
            type: state.swap.from.type,                 // leave type
            logoUrl: state.swap.from.logoUrl,           // leave logo
            tokenIndex: state.swap.from.tokenIndex,     // leave token index
            address: state.swap.from.address,           // leave address
            isValid: action.payload.isValid,            // UPDATE isValid
          },                                            //
          to: state.swap.to,                            // leave to input
          needsApproval: state.swap.needsApproval,      // leave needsApproval
          status: action.payload.status,                // UPDATE status
        }};
      case 'SET_TO_AMOUNT':
        return { ...state, swap: {
          to: { 
            amount: action.payload.amount,              // UPDATE amount
            symbol: state.swap.to.symbol,               // leave symbol
            type: state.swap.to.type,                   // leave type
            logoUrl: state.swap.to.logoUrl,             // leave logo
            tokenIndex: state.swap.to.tokenIndex,       // leave token index
            address: state.swap.to.address,             // leave address
            isValid: action.payload.isValid,            // UPDATE isValid
          },                                            //
          from: state.swap.from,                        // leave from input
          needsApproval: state.swap.needsApproval,      // leave needsApproval
          status: action.payload.status,                // UPDATE status
        }};
      case 'SET_INPUT1_AMOUNT':
        return { ...state, pool: { input1: { 
          amount: action.payload.amount,
          symbol: state.pool.input1.symbol,
          type: state.pool.input1.type,
          logoUrl: state.pool.input1.logoUrl,
          tokenIndex: state.pool.input1.tokenIndex,
          address: state.pool.input1.address,
          isValid: action.payload.isValid,
        }, input2: state.pool.input2 }};
      case 'SET_INPUT2_AMOUNT':
        return { ...state, pool: { input2: { 
          amount: action.payload.amount,
          symbol: state.pool.input2.symbol,
          type: state.pool.input2.type,
          logoUrl: state.pool.input2.logoUrl,
          tokenIndex: state.pool.input2.tokenIndex,
          address: state.pool.input2.address,
          isValid: action.payload.isValid,
        }, input1: state.pool.input1 }};

      // Updates the currency in the 'From' input card on the swap tab usually when a user chooses from the
      // currency selection modal
      case 'UPDATE_FROM_SELECTED_CURRENCY':
        return {...state, swap: { 
          from: {
            amount: state.swap.from.amount,               // leave amount
            symbol: action.payload.symbol,                // UPDATE symbol
            type: action.payload.type,                    // UPDATE type
            logoUrl: action.payload.logoUrl,              // UPDATE logo URL
            tokenIndex: action.payload.tokenIndex,        // UPDATE token index
            address: action.payload.address,              // UPDATE address
            isValid: state.swap.from.isValid,             // leave isValid alone (which just checks for a non-zero number)
          },                                              //
          to: state.swap.to,                              // leave the other input card alone
          needsApproval:                                  // NEP-21<>NEP-21 requires an extra approval, so check for
            action.payload.type === "NEP-21",             //    it and set it. It can be updated later if
                                                          //    needed by dispatching UPDATE_SWAP_APPROVAL
          status: "notReadyToSwap",                       // RESET status to notReadyToSwap
        }, currencySelectionModal: { isVisible: false }}; // Close the currency selection modal

      // Updates the currency in the 'To' input card on the swap tab usually when a user chooses from the
      // currency selection modal
      case 'UPDATE_TO_SELECTED_CURRENCY':
        return { ...state, swap: { 
          to: {
            amount: state.swap.to.amount,                 // leave amount
            symbol: action.payload.symbol,                // UPDATE symbol
            type: action.payload.type,                    // UPDATE type
            logoUrl: action.payload.logoUrl,              // UPDATE logo URL
            tokenIndex: action.payload.tokenIndex,        // UPDATE token index
            address: action.payload.address,              // UPDATE address
            isValid: state.swap.to.isValid,               // leave isValid alone (which just checks for a non-zero number)
          },                                              //
          from: state.swap.from,                          // leave the other input card alone
          needsApproval:                                  // NEP-21<>NEP-21 requires an extra approval, so check for
            state.swap.from.type === "NEP-21",            //    it and set it. It can be updated later if
                                                          //    needed by dispatching UPDATE_SWAP_APPROVAL
          status: "notReadyToSwap",                       // RESET status to notReadyToSwap
        }, currencySelectionModal: { isVisible: false }}; // close the currency selection modal

      // Updates the currency in the first input card for providing liquidity on the pool tab
      // usually when a user chooses from the currency selection modal
      case 'UPDATE_INPUT1_SELECTED_CURRENCY':
        return { ...state, pool: { input1: {
          amount: state.pool.input1.amount,
          symbol: action.payload.symbol,
          type: action.payload.type,
          logoUrl: action.payload.logoUrl,
          tokenIndex: action.payload.tokenIndex,
          address: action.payload.address,
          isValid: state.pool.input1.isValid,
        }, input2: state.pool.input2 }, currencySelectionModal: { isVisible: false }};

      // Updates the currency in the second 'Input' input card for providing liquidity on the pool tab
      // usually when a user chooses from the currency selection modal
      // (always set to NEAR)
      case 'UPDATE_INPUT2_SELECTED_CURRENCY':
        return { ...state, pool: { input2: {
          amount: state.pool.input2.amount,
          symbol: action.payload.symbol,
          type: action.payload.type,
          logoUrl: action.payload.logoUrl,
          tokenIndex: action.payload.tokenIndex,
          address: action.payload.address,
          isValid: state.pool.input2.isValid,
        }, input1: state.pool.input1 }, currencySelectionModal: { isVisible: false }};

      case 'TOGGLE_CURRENCY_SELECTION_MODAL':
        return { ...state, currencySelectionModal: { 
          isVisible: !state.currencySelectionModal.isVisible, 
          selectedInput: state.currencySelectionModal.selectedInput } };
      case 'UPDATE_SWAP_APPROVAL':
        return { ...state, swap: { 
          needsApproval: action.payload.needsApproval,
          status: state.swap.status,
        to: state.swap.to, from: state.swap.from }}
      case 'SET_CURRENCY_SELECTION_INPUT':
        return { ...state, currencySelectionModal: { selectedInput: action.payload.input, isVisible: !state.isVisible } };
      default:
        throw new Error();
    };
  }, initialState);

  return <Provider value={{ state, dispatch }}>{children}</Provider>;
}

export { InputsContext, InputsProvider };
