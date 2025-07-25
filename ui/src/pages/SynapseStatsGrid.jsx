import React from "react"

export default function SynapseStatsGrid() {
  return (
    <div className="flex gap-2 md:gap-2.5 lg:gap-3">
      <div className="glass-card flex flex-col justify-center items-center px-4 py-3 lg:px-5 lg:py-4 rounded-xl lg:rounded-2xl group cursor-pointer flex-[1.6]">
        <span className="text-white font-medium text-xs body-text tracking-box opacity-80 mb-1">
          DEEP WORK
        </span>
        <span className="text-white font-bold text-base md:text-lg lg:text-xl xl:text-2xl body-text transition-all duration-300 mb-1 tracking-box">
          3h 12m
        </span>
        <span className="text-white/60 text-xs body-text tracking-box">Today</span>
        <div className="absolute inset-0 bg-white/5 rounded-xl lg:rounded-2xl opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
      </div>
      <div className="glass-card flex flex-col justify-center items-center flex-1 text-center px-4 py-3 lg:px-5 lg:py-4 rounded-xl lg:rounded-2xl group cursor-pointer">
        <span className="text-white font-medium text-xs body-text tracking-box opacity-80 mb-1">
          DISTRACTIONS
        </span>
        <span className="text-white font-bold text-base md:text-lg lg:text-xl xl:text-2xl body-text transition-all duration-300 mb-1 tracking-box">
          9
        </span>
        <span className="text-white/60 text-xs body-text tracking-box">Blocked</span>
        <div className="absolute inset-0 bg-white/5 rounded-xl lg:rounded-2xl opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
      </div>
    </div>
  )
}
