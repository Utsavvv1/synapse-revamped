import React from "react"
import { Edit3, AlertOctagon } from "lucide-react"

export default function SynapseActions() {
  return (
    <div className="flex gap-2 md:gap-2.5 lg:gap-3">
      <button className="group flex items-center justify-center bg-synapse-accent rounded-full text-synapse-dark font-medium body-text flex-1 button-accent-hover hover:bg-synapse-accent/90 transition-all duration-300 px-3 py-1.5 md:px-4 md:py-2 lg:px-5 lg:py-2.5 gap-1 md:gap-1.5 text-xs sm:text-sm glow-accent whitespace-nowrap">
        <Edit3 className="w-2.5 h-2.5 md:w-3 md:h-3 lg:w-3.5 lg:h-3.5 group-hover:rotate-12 transition-transform duration-300 flex-shrink-0" />
        <span className="hidden lg:inline">Edit Focus Apps</span>
        <span className="lg:hidden">Focus Apps</span>
      </button>
      <button className="group flex items-center justify-center glass-card-dark rounded-full text-synapse-accent font-medium body-text flex-1 button-hover hover:bg-black/60 transition-all duration-300 px-3 py-1.5 md:px-4 md:py-2 lg:px-5 lg:py-2.5 gap-1 md:gap-1.5 text-xs sm:text-sm whitespace-nowrap">
        <AlertOctagon className="w-2.5 h-2.5 md:w-3 md:h-3 lg:w-3.5 lg:h-3.5 group-hover:shake transition-transform duration-300 flex-shrink-0" />
        <span className="hidden lg:inline">Edit Distraction Apps</span>
        <span className="lg:hidden">Distractions</span>
      </button>
    </div>
  )
} 