import React, { useReducer, useEffect } from "react";

const NotificationContext = React.createContext();
const { Provider } = NotificationContext;

let initialState = {
  heading: "",
  message: "",
  show: false,
}

// LMT commented, notifications should NOT be persistent
// // Initialize with previous notification state if found in local storage
// let notifs = localStorage.getItem("notifs");
// if (notifs) {
//   initialState = JSON.parse(notifs);
// }

const NotificationProvider = ({ children }) => {
  
  const [state, dispatch] = useReducer((state, action) => {
    switch(action.type) {
      case 'SHOW_NOTIFICATION':
        let newNotif = {
          heading: action.payload.heading,
          message: action.payload.message,
          show: true,
        }
        localStorage.setItem("notifs", JSON.stringify(newNotif));
        return newNotif;
        
      case 'HIDE_NOTIFICATION':
        localStorage.setItem("notifs", JSON.stringify({ show: false }));
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
