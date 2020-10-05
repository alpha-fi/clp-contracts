import React, { useContext, createContext, useReducer } from 'react';
import { produce } from 'immer';
import { NotificationContext } from './NotificationContext';
import { convertToE24Base5Dec } from '../services/near-nep21-util';
import { calcPriceFromIn, calcPriceFromOut, swapFromOut, incAllowance, getAllowance } from "../services/near-nep21-util";
import { saveInputsStateLocalStorage } from '../components/CurrencyTable';
import { computeInAmount } from "../components/SwapInputCards";

const initialInput = {
  amount: "",       // Amount of tokens
  symbol: "",       // Symbol of token
  type: "",         // Native token (NEAR), ERC-20, NEP-21, ...
  logoUrl: "",      // Address of token logo image
  tokenIndex: 0,   // Index of currency within token list
  address: "",      // Token address of selected currency
  isValid: false,   // True if non-zero number, false otherwise
  allowance: null,  // Required for NEP-21 swaps (null otherwise)
  balance: "",      // Balance of selected currency
}

let initialState = {
  swap: {
    in: produce(initialInput, draft => {
      draft.tokenIndex = 0;
    }),
    out: produce(initialInput, draft => {
      draft.tokenIndex = 1;
    }),
    error: null,               // error string if error, null otherwise
    needsApproval: false,      // required for NEP-21 swaps
    status: "notReadyToSwap",  // possible values: notReadyToSwap, readyToSwap, isApproving, isSwapping, fetchingData
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
try {
  let savedInputs = localStorage.getItem("inputs");
  if (savedInputs) {
    initialState = JSON.parse(savedInputs);
    //doit only once - clear
    localStorage.setItem("inputs", "");
  }
}
catch (ex) {
  //invalid state, clear
  localStorage.setItem("inputs", "");
  throw ex;
}

const InputsContext = createContext(initialState);
const { Provider } = InputsContext;



function reduce(state, action) {

  switch (action.type) {

    case 'SET_TOKEN_BALANCE':
      return produce(state, draft => {
        if (action.payload.index == draft.swap.in.tokenIndex) {
          draft.swap.in.balance = action.payload.balance
        }
        if (action.payload.index == draft.swap.out.tokenIndex) {
          draft.swap.out.balance = action.payload.balance
        }
      })

    case 'SET_STATUS_READY':
      return produce(state, draft => {
        draft.swap.status = "readyToSwap";
      })

    //before calling the wallet
    case 'UPDATE_SWAP_STATUS':
      return produce(state, draft => {
        draft.swap.status = action.payload.status;
        draft.swap.error = action.payload.error;
        draft.swap.previous = action.payload.previous;
      });

    case 'CLEAR_PREVIOUS':
      let newState = produce(state, draft => {
        draft.swap.previous = "";
      });
      saveInputsStateLocalStorage(newState);
      return newState;

    // case 'FETCH_NEAR_BALANCES':
    //   let updatedNearTokenList = updateNearBalances(state.tokens);
    //   return { tokenList: updatedNearTokenList };
    //   break;

    // case 'FETCH_ETH_BALANCES':
    //   let updatedEthTokenList = updateEthBalances(state.tokens, action.payload.w3.web3, action.payload.ethAccount);
    //   return { tokenList: updatedEthTokenList };
    //   break;

    case 'SET_OUT_AMOUNT':
      return produce(state, draft => {
        draft.swap.in.amount = action.payload.amount;
        draft.swap.in.isValid = action.payload.isValid;
      });

    case 'SET_IN_AMOUNT':
      return produce(state, draft => {
        draft.swap.out.amount = action.payload.amount;
        draft.swap.out.isValid = action.payload.isValid;
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
    case 'UPDATE_IN_SELECTED_CURRENCY':

      return produce(state, draft => {
        draft.swap.in.symbol = action.payload.symbol;
        draft.swap.in.type = action.payload.type;
        draft.swap.in.logoUrl = action.payload.logoUrl;
        draft.swap.in.tokenIndex = action.payload.tokenIndex;
        draft.swap.in.address = action.payload.address;
        draft.swap.in.allowance = "";
        if (action.payload.balance !== undefined) draft.swap.in.balance = action.payload.balance;
        draft.swap.needsApproval = (action.payload.type === "NEP-21");
        draft.swap.status = "notReadyToSwap";
        draft.currencySelectionModal.isVisible = false;
        draft.swap.in.amount = "";
        //draft.swap.out.amount = "";
      });

    // Updates the currency in the 'To' input card on the swap tab usually when a user chooses from the
    // currency selection modal
    case 'UPDATE_OUT_SELECTED_CURRENCY':
      return produce(state, draft => {
        draft.swap.out.symbol = action.payload.symbol;
        draft.swap.out.type = action.payload.type;
        draft.swap.out.logoUrl = action.payload.logoUrl;
        draft.swap.out.tokenIndex = action.payload.tokenIndex;
        draft.swap.out.address = action.payload.address;
        draft.swap.out.allowance = null;
        if (action.payload.balance !== undefined) draft.swap.out.balance = action.payload.balance;
        draft.swap.needsApproval = false; //(state.swap.in.type === "NEP-21");
        draft.swap.status = "notReadyToSwap";
        draft.currencySelectionModal.isVisible = false;
        //draft.swap.in.amount = "";
        //draft.swap.out.amount = "";
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
        if (action.payload.balance !== undefined) draft.pool.input1.balance = action.payload.balance;
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
        if (action.payload.balance !== undefined) draft.pool.input2.balance = action.payload.balance;
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
        draft.swap.in.amount = "";
        draft.swap.in.isValid = false;
        draft.swap.out.amount = "";
        draft.swap.out.isValid = false;
        draft.swap.status = "notReadyToSwap";
        draft.swap.error = null;
      });

    case 'UPDATE_SWAP_ERROR':
      return produce(state, draft => {
        draft.swap.error = action.payload.error;
      });

    case 'UPDATE_IN_ALLOWANCE':
      return produce(state, draft => {
        draft.swap.in.allowance = action.payload.allowance;
        if (action.payload.needsApproval!==undefined) draft.swap.in.needsApproval = action.payload.needsApproval;
      });

    case 'SWITCH_SWAP_INPUTS':
      let oldOut = state.swap.out;
      let oldIn = state.swap.in;
      return produce(state, draft => {
        draft.swap.in = {
          ...oldOut,
          amount: 0,
        }
        draft.swap.out = oldIn;
        draft.swap.needsApproval = (state.swap.out.type === "NEP-21")
        draft.swap.status = "notReadyToSwap";
        draft.swap.error = null;
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

}

const InputsProvider = ({ children }) => {

  const [state, dispatch] = useReducer(reduce, initialState);

  return <Provider value={{ state, dispatch }}>{children}</Provider>;
}

export { InputsContext, InputsProvider };
