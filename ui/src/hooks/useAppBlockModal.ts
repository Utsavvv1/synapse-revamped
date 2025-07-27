import { useState, useCallback } from 'react'

interface UseAppBlockModalReturn {
  isVisible: boolean
  showModal: () => void
  hideModal: () => void
  handleCloseApp: () => void
  handleUseFor5Mins: () => void
  handleShowAgain?: () => void
}

export function useAppBlockModal(): UseAppBlockModalReturn {
  const [isVisible, setIsVisible] = useState(false)

  const showModal = useCallback(() => {
    setIsVisible(true)
  }, [])

  const hideModal = useCallback(() => {
    setIsVisible(false)
  }, [])

  const handleCloseApp = useCallback(() => {
    setIsVisible(false)
    // In a real app, this would close the app or navigate away
    console.log("App closed")
  }, [])

  const handleUseFor5Mins = useCallback(() => {
    setIsVisible(false)
    // In a real app, this would start a 5-minute timer
    console.log("Using app for 5 minutes")
  }, [])

  const handleShowAgain = useCallback(() => {
    setIsVisible(true)
  }, [])

  return {
    isVisible,
    showModal,
    hideModal,
    handleCloseApp,
    handleUseFor5Mins,
    handleShowAgain
  }
} 