import React from "react"
import { Zap } from 'lucide-react'
import { WindowControls } from "../components/WindowControls"

export default function SynapseHeader({ currentTime }) {
  return (
    <header data-tauri-drag-region className="flex justify-between items-center mb-4 lg:mb-6">
      <h1 className="font-dmserif tracking-tightest italic text-synapse-dark text-xl sm:text-2xl lg:text-3xl font-light">
        Synapse
      </h1>
      <div className="flex items-center gap-2 sm:gap-3 lg:gap-4">
        <span className="font-medium text-white text-base sm:text-lg lg:text-xl body-text">{currentTime}</span>
        <div className="phone-connected rounded-full px-4 py-2 gap-2 flex items-center transform scale-90">
          <span className="text-gray-200 font-medium text-sm body-text mr-2">Phone Connected</span>
          <div className="relative flex items-center gap-2">
            <div className="w-6 h-6 bg-synapse-dark-alt rounded-full flex items-center justify-center mr-1 ml-2">
              <div className="w-1.5 h-1.5 bg-green-400 rounded-full absolute -left-2 pulse-dot" />
            </div>
          </div>
        </div>
        <WindowControls />
      </div>
    </header>
  )
}