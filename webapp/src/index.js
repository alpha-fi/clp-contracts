import React from 'react'
import ReactDOM from 'react-dom'
import App from './App'
import { initContract } from './utils'

import { ThemeProvider } from "./contexts/ThemeContext";
import { InputsProvider } from './contexts/InputsContext';
import { Web3Provider } from './contexts/Web3Context';
import { TokenListProvider } from './contexts/TokenListContext';
import { NotificationProvider } from './contexts/NotificationContext';

window.nearInitPromise = initContract()
  .then(() => {
    ReactDOM.render(
      <InputsProvider>
        <Web3Provider>
          <TokenListProvider>
            <NotificationProvider>
              <ThemeProvider>
                <App />
              </ThemeProvider>
            </NotificationProvider>
          </TokenListProvider>
        </Web3Provider>
      </InputsProvider>,
      document.querySelector('#root')
    )
  })
  .catch(console.error)
