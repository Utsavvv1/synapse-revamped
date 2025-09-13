"use client"

import { useState, useEffect } from "react"
import { AlertOctagon } from "lucide-react"

interface AppBlockModalProps {
  isVisible: boolean
  onClose: () => void
  onUseFor5Mins: () => void
  onShowAgain?: () => void
}

export default function AppBlockModal({ 
  isVisible, 
  onClose, 
  onUseFor5Mins, 
  onShowAgain 
}: AppBlockModalProps) {
  const [isAnimating, setIsAnimating] = useState(false)
  const [shouldRender, setShouldRender] = useState(false)

  useEffect(() => {
    if (isVisible) {
      setShouldRender(true)
      // Small delay to ensure DOM is ready
      setTimeout(() => setIsAnimating(true), 10)
    } else {
      setIsAnimating(false)
      // Wait for animation to complete before removing from DOM
      setTimeout(() => setShouldRender(false), 300)
    }
  }, [isVisible])

  const handleCloseApp = () => {
    onClose()
    // In a real app, this would close the app or navigate away
    console.log("App closed")
  }

  const handleUseFor5Mins = () => {
    onUseFor5Mins()
    // In a real app, this would start a 5-minute timer
    console.log("Using app for 5 minutes")
  }

  const handleShowAgain = () => {
    onShowAgain?.()
  }

  if (!shouldRender) {
    return null
  }

  return (
    <>
      {/* Backdrop Overlay with Animation */}
      <div
        className={`fixed inset-0 backdrop-blur-sm z-40 transition-all duration-300 ease-out ${
          isAnimating ? "bg-black/50 opacity-100" : "bg-black/0 opacity-0"
        }`}
      />

      {/* Subtle Gradient Background with Animation */}
      <div
        className={`fixed inset-0 z-50 flex items-center justify-center p-4 transition-opacity duration-300 ease-out ${
          isAnimating ? "opacity-100" : "opacity-0"
        }`}
      >
        <div
          className="absolute inset-0"
          style={{
            background: `radial-gradient(circle at center, hsl(var(--synapse-dark) / 0.08) 0%, hsl(var(--synapse-dark) / 0.04) 40%, hsl(var(--synapse-dark) / 0.02) 70%, transparent 100%)`,
          }}
        />

        {/* Modal Container with Smooth Animation */}
        <div
          className={`relative w-full max-w-[280px] sm:max-w-[320px] lg:max-w-[360px] mx-auto transition-all duration-300 ease-out ${
            isAnimating ? "scale-100 opacity-100 translate-y-0" : "scale-95 opacity-0 translate-y-4"
          }`}
        >
          {/* Main container with single neon edge */}
          <div className="relative bg-synapse-accent rounded-2xl sm:rounded-3xl px-3 py-4 sm:px-4 sm:py-5 lg:px-5 lg:py-6 flex flex-col items-center gap-2 sm:gap-3 overflow-hidden border-2 border-synapse-dark">
            {/* Content */}
            <div className="relative z-10 flex flex-col items-center gap-2 sm:gap-3 w-full">
              {/* Main Title with Staggered Animation */}
              <h1
                className={`text-synapse-dark text-center font-medium text-base sm:text-lg lg:text-xl leading-tight tracking-tight transition-all duration-500 ease-out delay-100 font-wix ${
                  isAnimating ? "opacity-100 translate-y-0" : "opacity-0 translate-y-2"
                }`}
              >
                {"You're getting distracted!"}
              </h1>

              {/* Warning Icon with Scale Animation */}
              <div
                className={`flex justify-center items-center p-2 transition-all duration-500 ease-out delay-200 ${
                  isAnimating ? "opacity-100 scale-100" : "opacity-0 scale-90"
                }`}
              >
                <AlertOctagon size={48} className="text-synapse-dark sm:w-12 sm:h-12 lg:w-14 lg:h-14" strokeWidth={1.5} />
              </div>

              {/* Subtitle with Staggered Animation */}
              <p
                className={`text-synapse-dark text-center font-normal text-xs sm:text-sm lg:text-base leading-tight tracking-tight max-w-xs transition-all duration-500 ease-out delay-300 font-wix ${
                  isAnimating ? "opacity-100 translate-y-0" : "opacity-0 translate-y-2"
                }`}
              >
                This app is in your block list
              </p>

              {/* Action Buttons with Staggered Animation */}
              <div
                className={`flex flex-col sm:flex-row gap-2 w-full max-w-[240px] transition-all duration-500 ease-out delay-400 ${
                  isAnimating ? "opacity-100 translate-y-0" : "opacity-0 translate-y-3"
                }`}
              >
                {/* Close App Button */}
                <button
                  onClick={handleCloseApp}
                  className="flex-1 py-2 px-3 sm:px-4 lg:px-5 rounded-xl sm:rounded-2xl font-medium text-xs sm:text-sm text-center bg-synapse-dark text-synapse-accent hover:bg-synapse-dark/90 hover:scale-105 active:scale-95 transition-all duration-200 ease-out whitespace-nowrap font-wix"
                >
                  Close App
                </button>

                {/* Use for 5 mins Button */}
                <button
                  onClick={handleUseFor5Mins}
                  className="flex-1 bg-transparent py-2 px-3 sm:px-4 lg:px-5 rounded-xl sm:rounded-2xl font-medium text-xs sm:text-sm text-center text-synapse-dark border border-synapse-dark/30 hover:bg-synapse-dark/5 hover:border-synapse-dark/50 hover:scale-105 active:scale-95 transition-all duration-200 ease-out whitespace-nowrap font-wix"
                >
                  Use for 5 mins
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  )
} 