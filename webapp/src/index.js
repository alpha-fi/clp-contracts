import React from 'react'
import ReactDOM from 'react-dom'
import App from './App'
import { initContract } from './utils'

import { ThemeProvider } from "./contexts/ThemeContext";

window.nearInitPromise = initContract()
  .then(() => {
    ReactDOM.render(
      <ThemeProvider>
        <App />
      </ThemeProvider>,
      document.querySelector('#root')
    )
  })
  .catch(console.error)
