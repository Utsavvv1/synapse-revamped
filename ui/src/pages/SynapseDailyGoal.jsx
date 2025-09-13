import React from "react"

export default function SynapseDailyGoal() {
  return (
    <div className="flex flex-col items-center gap-2 mt-4 lg:mt-6">
      <div className="flex items-center gap-2">
        <h3 className="text-white font-medium text-sm lg:text-base body-text">Daily Focus Goal</h3>
        <span className="text-synapse-accent text-xs font-medium">70%</span>
      </div>
      <div className="w-full max-w-xs sm:max-w-sm lg:max-w-md progress-bar rounded-full h-1.5 relative tracking-normal">
        <div className="progress-fill rounded-full h-1 absolute top-0.5 left-0.5 w-[70%] transition-all duration-1000 ease-out" />
      </div>
    </div>
  )
} 