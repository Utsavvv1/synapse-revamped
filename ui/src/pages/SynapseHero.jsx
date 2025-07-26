import React, { useState } from "react"
import { Play, Square } from "lucide-react"
import { invoke } from "@tauri-apps/api/core"

// Reusable button component
function MonitoringButton({ content, Icon, onClick }) {
  return (
    <button
      className="my-6 group flex items-center bg-synapse-accent rounded-full text-synapse-dark font-medium body-text hover:bg-synapse-accent/90 transition-all duration-300 w-fit px-4 py-2 md:px-5 md:py-2.5 lg:px-6 lg:py-3 gap-2 text-xs md:text-sm lg:text-base glow-accent hover:scale-105"
      onClick={onClick}
    >
      <Icon className="w-3 h-3 md:w-3.5 md:h-3.5 lg:w-4 lg:h-4 fill-current group-hover:scale-110 transition-transform duration-300" />
      {content}
    </button>
  )
}

export default function SynapseHero() {
  const [isMonitoring, setIsMonitoring] = useState(false)

  // These functions just toggle the button
  const func1 = () => setIsMonitoring(true)
  const func2 = () => setIsMonitoring(false)

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
      {isMonitoring ? (
        <MonitoringButton content="Stop Monitoring" Icon={Square} onClick={func2} />
      ) : (
        <MonitoringButton content="Start Monitoring" Icon={Play} onClick={func1} />
      )}
    </div>
  )
} 