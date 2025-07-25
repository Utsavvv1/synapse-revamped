import React, { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"

export default function SynapseStatsGrid() {
  const [focusTime, setFocusTime] = useState(0)
  const [distractions, setDistractions] = useState(0)

  useEffect(() => {
    invoke("total_focus_time_today_cmd")
      .then((val) => setFocusTime(Number(val)))
      .catch((err) => {
        setFocusTime(0);
        console.error("Failed to fetch focus time:", err);
      });
    invoke("total_distractions_today_cmd")
      .then((val) => setDistractions(Number(val)))
      .catch(() => setDistractions(0))
  }, [])

  // Helper to format seconds as Hh Mm
  function formatTime(seconds) {
    const h = Math.floor(seconds / 3600)
    const m = Math.floor((seconds % 3600) / 60)
    return `${h > 0 ? h + "h " : ""}${m}m`
  }

  return (
    <div className="flex gap-2 md:gap-2.5 lg:gap-3">
      <div className="glass-card flex flex-col justify-center items-center px-4 py-3 lg:px-5 lg:py-4 rounded-xl lg:rounded-2xl group cursor-pointer flex-[1.6]">
        <span className="text-white font-medium text-xs body-text tracking-box opacity-80 mb-1">
          DEEP WORK
        </span>
        <span className="text-white font-bold text-base md:text-lg lg:text-xl xl:text-2xl body-text transition-all duration-300 mb-1 tracking-box">
          {formatTime(focusTime)}
        </span>
        <span className="text-white/60 text-xs body-text tracking-box">Today</span>
        <div className="absolute inset-0 bg-white/5 rounded-xl lg:rounded-2xl opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
      </div>
      <div className="glass-card flex flex-col justify-center items-center flex-1 text-center px-4 py-3 lg:px-5 lg:py-4 rounded-xl lg:rounded-2xl group cursor-pointer">
        <span className="text-white font-medium text-xs body-text tracking-box opacity-80 mb-1">
          DISTRACTIONS
        </span>
        <span className="text-white font-bold text-base md:text-lg lg:text-xl xl:text-2xl body-text transition-all duration-300 mb-1 tracking-box">
          {distractions}
        </span>
        <span className="text-white/60 text-xs body-text tracking-box">Blocked</span>
        <div className="absolute inset-0 bg-white/5 rounded-xl lg:rounded-2xl opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
      </div>
    </div>
  )
}
