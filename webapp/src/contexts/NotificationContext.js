import React, { useReducer } from "react";

const NotificationContext = React.createContext();
const { Provider } = NotificationContext;

const initialState = {
  heading: "",
  message: "",
  show: false,
}

const NotificationProvider = ({ children }) => {
  
  const [state, dispatch] = useReducer((state, action) => {
    switch(action.type) {
      case 'SHOW_NOTIFICATION':
        return { 
          heading: action.payload.heading,
          message: action.payload.message,
          show: true,
        };
      case 'HIDE_NOTIFICATION':
        return { 
          ...state,
          show: false,
        };
      default:
        throw new Error();
    };
  }, initialState);

  return (
    <Provider value={{ state, dispatch }}>
      {children}
    </Provider>
  );
};

export { NotificationContext, NotificationProvider };
