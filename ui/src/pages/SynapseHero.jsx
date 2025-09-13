import React, { useState, useEffect } from "react"
import { Play, Square } from "lucide-react"
import { invoke } from "@tauri-apps/api/core"

// Reusable button component
function MonitoringButton({ content, Icon, onClick, isLoading, disabled }) {
  return (
    <button
      className={`my-6 group flex items-center bg-synapse-accent rounded-full text-synapse-dark font-medium body-text hover:bg-synapse-accent/90 transition-all duration-300 w-fit px-4 py-2 md:px-5 md:py-2.5 lg:px-6 lg:py-3 gap-2 text-xs md:text-sm lg:text-base glow-accent hover:scale-105 ${
        disabled ? "opacity-50 cursor-not-allowed" : ""
      }`}
      onClick={onClick}
      disabled={disabled}
    >
      {isLoading ? (
        <div className="w-3 h-3 md:w-3.5 md:h-3.5 lg:w-4 lg:h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
      ) : (
        <Icon className="w-3 h-3 md:w-3.5 md:h-3.5 lg:w-4 lg:h-4 fill-current group-hover:scale-110 transition-transform duration-300" />
      )}
      {content}
    </button>
  )
}

export default function SynapseHero() {
  const [isMonitoring, setIsMonitoring] = useState(false)
  const [isLoading, setIsLoading] = useState(false)

  // Check initial monitoring state on component mount
  useEffect(() => {
    checkMonitoringState()
  }, [])

  const checkMonitoringState = async () => {
    try {
      const monitoring = await invoke("is_monitoring_cmd")
      setIsMonitoring(monitoring)
    } catch (err) {
      console.error("Failed to check monitoring state:", err)
    }
  }

  const startMonitoring = async () => {
    setIsLoading(true)
    try {
      await invoke("start_monitoring_cmd")
      setIsMonitoring(true)
      console.log("Monitoring started successfully")
    } catch (err) {
      console.error("Failed to start monitoring:", err)
    } finally {
      setIsLoading(false)
    }
  }

  const stopMonitoring = async () => {
    setIsLoading(true)
    try {
      await invoke("stop_monitoring_cmd")
      setIsMonitoring(false)
      console.log("Monitoring stopped successfully")
    } catch (err) {
      console.error("Failed to stop monitoring:", err)
    } finally {
      setIsLoading(false)
    }
  }

  const handleToggle = () => {
    if (isMonitoring) {
      stopMonitoring()
    } else {
      startMonitoring()
    }
  }

  return (
    <div className="flex-1 flex flex-col justify-center w-full min-w-0">
      <div className="mb-3 lg:mb-4">
        <h2 className="font-medium text-synapse-dark text-xl sm:text-2xl md:text-3xl lg:text-4xl xl:text-5xl 2xl:text-6xl hero-text leading-tight mb-0.5 transform scale-110 origin-left tracking-tightest">
          Good morning!
        </h2>
        <div className="font-medium text-synapse-dark text-2xl sm:text-3xl md:text-4xl lg:text-5xl xl:text-6xl 2xl:text-7xl hero-text leading-[0.85] transform scale-110 origin-left tracking-tightest">
          Let's set the <span className="font-instrument italic tracking-tighter text-synapse-accent">tone</span> for
          <br className="hidden sm:block" />
          <span className="sm:hidden"> </span>the day.
        </div>
      </div>
      <MonitoringButton 
        content={isLoading ? "Loading..." : isMonitoring ? "Stop Monitoring" : "Start Monitoring"}
        Icon={isMonitoring ? Square : Play}
        onClick={handleToggle}
        isLoading={isLoading}
        disabled={isLoading}
      />
    </div>
  )
} 