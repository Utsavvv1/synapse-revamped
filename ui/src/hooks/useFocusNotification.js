import { useState, useEffect } from 'react';

/**
 * useFocusNotification Hook
 * 
 * Manages the focus notification state and animation.
 * 
 * Returns:
 * - isVisible: boolean - Whether notification is visible
 * - position: string - Current position (vh value)
 * - triggerNotification: function - Function to trigger the notification
 * - resetNotification: function - Function to reset notification state
 */
export const useFocusNotification = (duration = 6000) => {
  const [isVisible, setIsVisible] = useState(false);
  const [position, setPosition] = useState(-100);

  const triggerNotification = () => {
    console.log("🚀 Starting notification animation");
    setIsVisible(true);
    setPosition(-100); // Start from above screen

    // Enter screen after 100ms
    setTimeout(() => {
      console.log("📱 Notification entering screen");
      setPosition(15); // Position at center top
    }, 100);

    // Hide after specified duration
    setTimeout(() => {
      console.log("⏰ Notification timeout - hiding");
      setIsVisible(false);
      setPosition(-100);
    }, duration);
  };

  const resetNotification = () => {
    setIsVisible(false);
    setPosition(-100);
  };

  return {
    isVisible,
    position: `${position}vh`,
    triggerNotification,
    resetNotification
  };
}; 