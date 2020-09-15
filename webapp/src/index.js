import React from 'react'
import ReactDOM from 'react-dom'
import App from './App'
import { initContract } from './utils'

import { ThemeProvider } from "./contexts/ThemeContext";
import { GlobalStateProvider } from './contexts/GlobalContext';
import { TokenListProvider } from './contexts/TokenListContext';

window.nearInitPromise = initContract()
  .then(() => {
    ReactDOM.render(
      <GlobalStateProvider>
        <TokenListProvider>
          <ThemeProvider>
            <App />
          </ThemeProvider>
        </TokenListProvider>
      </GlobalStateProvider>,
      document.querySelector('#root')
    )
  })
  .catch(console.error)
