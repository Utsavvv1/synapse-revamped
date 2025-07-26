import { useState, useEffect } from "react"
import { Window } from "@tauri-apps/api/window"
import { Minus, Square, X, Copy } from 'lucide-react'

export function WindowControls() {
  const [isMaximized, setIsMaximized] = useState(false)
  const [appWindow, setAppWindow] = useState<Window | null>(null)

  useEffect(() => {
    let unlistenResize: (() => void) | null = null
    let unlistenMax: (() => void) | null = null
    let unlistenUnmax: (() => void) | null = null
    const setupWindow = async () => {
      try {
        // Dynamic import to ensure it works in Tauri environment
        const { getCurrentWindow } = await import("@tauri-apps/api/window")
        const window = getCurrentWindow()
        setAppWindow(window)

        const maximized = await window.isMaximized()
        setIsMaximized(maximized)

        unlistenResize = await window.onResized(async () => {
          const maximized = await window.isMaximized()
          setIsMaximized(maximized)
        })
        // Listen for maximize/unmaximize events using the correct event names
        unlistenMax = await window.listen("tauri://maximize", () => setIsMaximized(true))
        unlistenUnmax = await window.listen("tauri://unmaximize", () => setIsMaximized(false))

        return () => {
          unlistenResize && unlistenResize()
          unlistenMax && unlistenMax()
          unlistenUnmax && unlistenUnmax()
        }
      } catch (error) {
        console.error("Failed to setup window:", error)
      }
    }

    const cleanupPromise = setupWindow()
    return () => {
      cleanupPromise.then((cleanup) => {
        if (typeof cleanup === 'function') cleanup()
      })
    }
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
          setIsMaximized(false)
        } else {
          await appWindow.maximize()
          setIsMaximized(true)
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
    <div className="rounded-full px-3 py-2 gap-0.5 flex items-center transform scale-90 bg-black/30 backdrop-blur-sm border border-white/10">
      {/* Minimize Button */}
      <button
        onClick={handleMinimize}
        className="w-8 h-8 rounded-full hover:bg-white/10 transition-colors duration-150 flex items-center justify-center group"
        title="Minimize"
      >
        <Minus className="w-3.5 h-3.5 text-white/70 group-hover:text-white transition-colors" strokeWidth={2.5} />
      </button>

      {/* Maximize/Restore Button */}
      <button
        onClick={handleMaximize}
        className="w-8 h-8 rounded-full hover:bg-white/10 transition-colors duration-150 flex items-center justify-center group"
        title={isMaximized ? "Restore" : "Maximize"}
      >
        {isMaximized ? (
          <Copy className="w-3.5 h-3.5 text-white/70 group-hover:text-white transition-colors" strokeWidth={2.5} />
        ) : (
          <Square className="w-3.5 h-3.5 text-white/70 group-hover:text-white transition-colors" strokeWidth={2.5} />
        )}
      </button>

      {/* Close Button */}
      <button
        onClick={handleClose}
        className="w-8 h-8 rounded-full hover:bg-red-500/30 transition-colors duration-150 flex items-center justify-center group"
        title="Close"
      >
        <X className="w-3.5 h-3.5 text-white/70 group-hover:text-red-400 transition-colors" strokeWidth={2.5} />
      </button>
    </div>
  )
}