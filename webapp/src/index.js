import React from 'react'
import ReactDOM from 'react-dom'
import App from './App'
import { initContract } from './utils'

import { ThemeProvider } from "./contexts/ThemeContext";
import { GlobalStateProvider } from './contexts/GlobalContext';
import { Web3Provider } from './contexts/Web3Context';
import { TokenListProvider } from './contexts/TokenListContext';

window.nearInitPromise = initContract()
  .then(() => {
    ReactDOM.render(
      <GlobalStateProvider>
        <Web3Provider>
          <TokenListProvider>
            <ThemeProvider>
              <App />
            </ThemeProvider>
          </TokenListProvider>
        </Web3Provider>
      </GlobalStateProvider>,
      document.querySelector('#root')
    )
  })
  .catch(console.error)
