import React from "react"
import { Play, Zap } from "lucide-react"

export default function SynapseHero() {
  return (
    <div className="flex-1 flex flex-col justify-center w-full min-w-0">
      <div className="mb-3 lg:mb-4">
        <h2 className="font-medium text-synapse-dark text-xl sm:text-2xl md:text-3xl lg:text-4xl xl:text-5xl 2xl:text-6xl hero-text leading-tight mb-0.5 transform scale-110 origin-left">
          Good morning!
        </h2>
        <div className="font-medium text-synapse-dark text-2xl sm:text-3xl md:text-4xl lg:text-5xl xl:text-6xl 2xl:text-7xl hero-text leading-[0.85] transform scale-110 origin-left">
          Let's set the <span className="text-synapse-accent font-serif italic">tone</span> for
          <br className="hidden sm:block" />
          <span className="sm:hidden"> </span>the day.
        </div>
      </div>
      <button className="group flex items-center bg-synapse-accent rounded-full text-synapse-dark font-medium body-text hover:bg-synapse-accent/90 transition-all duration-300 w-fit px-4 py-2 md:px-5 md:py-2.5 lg:px-6 lg:py-3 gap-2 text-xs md:text-sm lg:text-base glow-accent hover:scale-105">
        <Play className="w-3 h-3 md:w-3.5 md:h-3.5 lg:w-4 lg:h-4 fill-current group-hover:scale-110 transition-transform duration-300" />
        Start Focus
        <Zap className="w-2.5 h-2.5 md:w-3 md:h-3 opacity-0 group-hover:opacity-100 transition-all duration-300" />
      </button>
    </div>
  )
} 