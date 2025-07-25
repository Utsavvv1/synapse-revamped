import { useState, useEffect } from "react"

export function WindowControls() {
  const [isMaximized, setIsMaximized] = useState(false)
  const [appWindow, setAppWindow] = useState(null)

  useEffect(() => {
    const setupWindow = async () => {
      try {
        // Dynamic import to ensure it works in Tauri environment
        const { getCurrentWindow } = await import("@tauri-apps/api/window")
        const window = getCurrentWindow()
        setAppWindow(window)

        const maximized = await window.isMaximized()
        setIsMaximized(maximized)

        const unlistenResize = await window.onResized(() => {
          window.isMaximized().then(setIsMaximized)
        })

        return () => {
          unlistenResize()
        }
      } catch (error) {
        console.error("Failed to setup window:", error)
      }
    }

    setupWindow()
  }, [])

  const handleMinimize = async () => {
    try {
      if (appWindow) {
        await appWindow.minimize()
      }
    } catch (error) {
      console.error("Failed to minimize:", error)
    }
  }

  const handleMaximize = async () => {
    try {
      if (appWindow) {
        if (isMaximized) {
          await appWindow.unmaximize()
        } else {
          await appWindow.maximize()
        }
      }
    } catch (error) {
      console.error("Failed to maximize/restore:", error)
    }
  }

  const handleClose = async () => {
    try {
      if (appWindow) {
        await appWindow.close()
      }
    } catch (error) {
      console.error("Failed to close:", error)
    }
  }

  return (
    <div className="flex items-center space-x-1 ml-4">
      <button
        onClick={handleMinimize}
        className="w-3 h-3 rounded-full bg-yellow-500 hover:bg-yellow-400 transition-colors duration-200 flex items-center justify-center group"
        title="Minimize"
      >
        <div className="w-2 h-0.5 bg-yellow-800 rounded-full opacity-0 group-hover:opacity-100 transition-opacity"></div>
      </button>

      <button
        onClick={handleMaximize}
        className="w-3 h-3 rounded-full bg-green-500 hover:bg-green-400 transition-colors duration-200 flex items-center justify-center group"
        title={isMaximized ? "Restore" : "Maximize"}
      >
        <div className="w-1.5 h-1.5 border border-green-800 rounded-sm opacity-0 group-hover:opacity-100 transition-opacity"></div>
      </button>

      <button
        onClick={handleClose}
        className="w-3 h-3 rounded-full bg-red-500 hover:bg-red-400 transition-colors duration-200 flex items-center justify-center group"
        title="Close"
      >
        <div className="relative w-1.5 h-1.5 opacity-0 group-hover:opacity-100 transition-opacity">
          <div className="absolute top-1/2 left-0 w-1.5 h-0.5 bg-red-800 rounded-full transform -translate-y-1/2 rotate-45"></div>
          <div className="absolute top-1/2 left-0 w-1.5 h-0.5 bg-red-800 rounded-full transform -translate-y-1/2 -rotate-45"></div>
        </div>
      </button>
    </div>
  )
}