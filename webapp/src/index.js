import React from 'react'
import ReactDOM from 'react-dom'
import App from './App'
import { initContract } from './utils'

import { ThemeProvider } from "./contexts/ThemeContext";
import { GlobalStateProvider } from './contexts/GlobalContext';
import { TokenListProvider } from './contexts/TokenListContext';
import { Web3Provider } from './contexts/Web3Context';

window.nearInitPromise = initContract()
  .then(() => {
    ReactDOM.render(
      <GlobalStateProvider>
        <TokenListProvider>
          <Web3Provider>
            <ThemeProvider>
              <App />
            </ThemeProvider>
          </Web3Provider>
        </TokenListProvider>
      </GlobalStateProvider>,
      document.querySelector('#root')
    )
  })
  .catch(console.error)
