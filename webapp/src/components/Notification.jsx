import React, { useContext } from "react";

import { NotificationContext } from "../contexts/NotificationContext";

import Alert from 'react-bootstrap/Alert';

export default function Notification(props) {

  // Notification state
  const notification = useContext(NotificationContext);
  const { dispatch } = notification;

  function handleClose() {
    dispatch({ type: 'HIDE_NOTIFICATION' });
  }

  if (notification.state.show) {
    return (
      <Alert variant={notification.state.variant} onClose={handleClose} dismissible>
        <Alert.Heading>{notification.state.heading}</Alert.Heading>
        <p>
          {notification.state.message}
        </p>
      </Alert>
    );
  }
  return null;
}
