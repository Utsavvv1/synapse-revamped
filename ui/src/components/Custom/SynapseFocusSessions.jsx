import React from "react"
import { Target } from "lucide-react"

export default function SynapseFocusSessions() {
  return (
    <div className="glass-card flex items-center justify-center p-2 md:p-2.5 lg:p-3 rounded-xl lg:rounded-2xl gap-2 md:gap-3 group cursor-pointer hover:scale-105 transition-all duration-300">
      <Target className="w-3 h-3 md:w-3.5 md:h-3.5 text-synapse-accent opacity-80 group-hover:opacity-100 group-hover:rotate-180 transition-all duration-500" />
      <span className="text-white font-medium text-xs lg:text-sm body-text tracking-wide">FOCUS SESSIONS</span>
      <span className="text-white font-bold text-sm md:text-base lg:text-lg body-text">4</span>
    </div>
  )
} 