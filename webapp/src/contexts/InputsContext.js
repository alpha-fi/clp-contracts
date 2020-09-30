import React, {createContext, useReducer} from 'react';
import { produce } from 'immer';

const initialInput = {
  amount: "",       // Amount of tokens
  symbol: "",       // Symbol of token
  type: "",         // Native token (NEAR), ERC-20, NEP-21, ...
  logoUrl: "",      // Address of token logo image
  tokenIndex: 0,    // Index of currency within token list
  address: "",      // Token address of selected currency
  isValid: false,   // True if non-zero number, false otherwise
  allowance: null,  // Required for NEP-21 swaps (null otherwise)
  balance: "",      // Balance of selected currency
}

let initialState = { 
  swap: {
    from: produce(initialInput, draft => {
      draft.tokenIndex = 0;
    }),
    to: produce(initialInput, draft => {
      draft.tokenIndex = 1;
    }),
    error: null,               // error string if error, null otherwise
    needsApproval: false,      // required for NEP-21 swaps
    status: "notReadyToSwap",  // possible values: notReadyToSwap, readyToSwap, isApproving, isSwapping
    previous: null             // when isSwapping or isApproving, we need to store a value to compare
                               // to verify if the swap was successful
  },
  pool: {
    input1: produce(initialInput, draft => {
      draft.tokenIndex = 1;
    }),
    input2: produce(initialInput, draft => {
      draft.tokenIndex = 2;
    }),
  },
  currencySelectionModal: {
    isVisible: false,
    selectedInput: ""
  }
};

// Initialize with previous input state if found in local storage
let savedInputs = localStorage.getItem("inputs");
if (savedInputs) {
  // initialState = JSON.parse(savedInputs);
  initialState = produce(JSON.parse(savedInputs), draft => {
    draft.swap.error = null;
  });
}

const InputsContext = createContext(initialState);
const { Provider } = InputsContext;

const InputsProvider = ( { children } ) => {

  const [state, dispatch] = useReducer((state, action) => {
    switch(action.type) {
      case 'SET_FROM_AMOUNT':
        return produce(state, draft => {
          draft.swap.from.amount = action.payload.amount;
          draft.swap.from.isValid = action.payload.isValid;
        });
      case 'SET_TO_AMOUNT':
        return produce(state, draft => {
          draft.swap.to.amount = action.payload.amount;
          draft.swap.to.isValid = action.payload.isValid;
        });
      case 'SET_INPUT1_AMOUNT':
        return produce(state, draft => {
          draft.pool.input1.amount = action.payload.amount;
          draft.pool.input1.isValid = action.payload.isValid;
        });
      case 'SET_INPUT2_AMOUNT':
        return produce(state, draft => {
          draft.pool.input2.amount = action.payload.amount;
          draft.pool.input2.isValid = action.payload.isValid;
        });

      // Updates the currency in the 'From' input card on the swap tab usually when a user chooses from the
      // currency selection modal
      case 'UPDATE_FROM_SELECTED_CURRENCY':
        return produce(state, draft => {
          draft.swap.from.symbol = action.payload.symbol;
          draft.swap.from.type = action.payload.type;
          draft.swap.from.logoUrl = action.payload.logoUrl;
          draft.swap.from.tokenIndex = action.payload.tokenIndex;
          draft.swap.from.address = action.payload.address;
          draft.swap.from.allowance = "";
          draft.swap.from.balance = action.payload.balance;
          draft.swap.needsApproval = (action.payload.type === "NEP-21");
          draft.swap.status = "notReadyToSwap";
          draft.currencySelectionModal.isVisible = false;
          draft.swap.from.amount = "";
          draft.swap.to.amount = "";
        });

      // Updates the currency in the 'To' input card on the swap tab usually when a user chooses from the
      // currency selection modal
      case 'UPDATE_TO_SELECTED_CURRENCY':
        return produce(state, draft => {
          draft.swap.to.symbol = action.payload.symbol;
          draft.swap.to.type = action.payload.type;
          draft.swap.to.logoUrl = action.payload.logoUrl;
          draft.swap.to.tokenIndex = action.payload.tokenIndex;
          draft.swap.to.address = action.payload.address;
          draft.swap.to.allowance = null;
          draft.swap.to.balance = action.payload.balance;
          draft.swap.needsApproval = (state.swap.to.type === "NEP-21");
          draft.swap.status = "notReadyToSwap";
          draft.currencySelectionModal.isVisible = false;
          draft.swap.from.amount = "";
          draft.swap.to.amount = "";
        });

      // Updates the currency in the first input card for providing liquidity on the pool tab
      // usually when a user chooses from the currency selection modal
      case 'UPDATE_INPUT1_SELECTED_CURRENCY':
        return produce(state, draft => {
          draft.pool.input1.symbol = action.payload.symbol;
          draft.pool.input1.type = action.payload.type;
          draft.pool.input1.logoUrl = action.payload.logoUrl;
          draft.pool.input1.tokenIndex = action.payload.tokenIndex;
          draft.pool.input1.address = action.payload.address;
          draft.pool.input1.allowance = null;
          draft.pool.input1.balance = action.payload.balance;
          draft.currencySelectionModal.isVisible = false;
        });

      // Updates the currency in the second 'Input' input card for providing liquidity on the pool tab
      // usually when a user chooses from the currency selection modal
      // (always set to NEAR)
      case 'UPDATE_INPUT2_SELECTED_CURRENCY':
        return produce(state, draft => {
          draft.pool.input2.symbol = action.payload.symbol;
          draft.pool.input2.type = action.payload.type;
          draft.pool.input2.logoUrl = action.payload.logoUrl;
          draft.pool.input2.tokenIndex = action.payload.tokenIndex;
          draft.pool.input2.address = action.payload.address;
          draft.pool.input2.allowance = null;
          draft.pool.input2.balance = action.payload.balance;
          draft.currencySelectionModal.isVisible = false;
        });

      case 'TOGGLE_CURRENCY_SELECTION_MODAL':
        return produce(state, draft => {
          draft.currencySelectionModal.isVisible = !state.currencySelectionModal.isVisible;
        });
      case 'UPDATE_SWAP_APPROVAL':
        return produce(state, draft => {
          draft.swap.needsApproval = action.payload.needsApproval;
        });
      case 'CLEAR_SWAP_INPUTS':
        return produce(state, draft => {
          draft.swap.from.amount = "";
          draft.swap.from.isValid = false;
          draft.swap.to.amount = "";
          draft.swap.to.isValid = false;
          draft.swap.status = "notReadyToSwap";
          draft.swap.error = null;
        });
      case 'UPDATE_SWAP_ERROR':
        return produce(state, draft => {
          draft.swap.error = action.payload.error;
        });
      case 'UPDATE_FROM_ALLOWANCE':
        return produce(state, draft => {
          draft.swap.from.allowance = action.payload.allowance;
        });
      case 'SWITCH_SWAP_INPUTS':
        let oldTo = state.swap.to;
        let oldFrom = state.swap.from;
        return produce(state, draft => {
          draft.swap.from = {
            ...oldTo,
            amount: 0,
          }
          draft.swap.to = oldFrom;
          draft.swap.needsApproval = (state.swap.from.type === "NEP-21");
          draft.swap.status = "notReadyToSwap";
          draft.swap.error = null;
        });
      case 'UPDATE_SWAP_STATUS':
        return produce(state, draft => {
          draft.swap.status = action.payload.status;
          draft.swap.error = action.payload.error;
          draft.swap.previous = action.payload.previous;
        });
      case 'SET_CURRENCY_SELECTION_INPUT':
        return produce(state, draft => {
          draft.currencySelectionModal.selectedInput = action.payload.input;
          draft.currencySelectionModal.isVisible = !state.currencySelectionModal.isVisible;
        });
      case 'SAVE_INPUTS_TO_LOCAL_STORAGE':
        localStorage.setItem("inputs", JSON.stringify(state));
        return state;
      default:
        throw new Error();
    };
  }, initialState);

  return <Provider value={{ state, dispatch }}>{children}</Provider>;
}

export { InputsContext, InputsProvider };
