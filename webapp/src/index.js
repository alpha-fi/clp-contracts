import React from 'react'
import ReactDOM from 'react-dom'
import App from './App'
import { initContract } from './utils'

import { ThemeProvider } from "./contexts/ThemeContext";
import { InputsProvider } from './contexts/InputsContext';
import { Web3Provider } from './contexts/Web3Context';
import { TokenListProvider } from './contexts/TokenListContext';

window.nearInitPromise = initContract()
  .then(() => {
    ReactDOM.render(
      <InputsProvider>
        <Web3Provider>
          <TokenListProvider>
            <ThemeProvider>
              <App />
            </ThemeProvider>
          </TokenListProvider>
        </Web3Provider>
      </InputsProvider>,
      document.querySelector('#root')
    )
  })
  .catch(console.error)
