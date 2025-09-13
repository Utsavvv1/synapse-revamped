import React, { useEffect, useState } from "react"
import { Target } from "lucide-react"
import { invoke } from "@tauri-apps/api/core"

export default function SynapseFocusSessions() {
  const [sessionCount, setSessionCount] = useState(0)

  function fetchSessions() {
    invoke("total_focus_sessions_today_cmd")
      .then((val) => setSessionCount(Number(val)))
      .catch((err) => {
        setSessionCount(0);
        console.error("Failed to fetch focus sessions:", err);
      });
  }

  useEffect(() => {
    fetchSessions(); // fetch immediately
    const interval = setInterval(fetchSessions, 2000); // poll every 2s
    return () => clearInterval(interval);
  }, [])

  return (
    <div className="glass-card flex items-center justify-center p-2 md:p-2.5 lg:p-3 rounded-xl lg:rounded-2xl gap-2 md:gap-3 group cursor-pointer relative transition-all duration-300">
      <Target className="w-3 h-3 md:w-3.5 md:h-3.5 text-synapse-accent opacity-80 group-hover:opacity-100 group-hover:rotate-180 transition-all duration-500" />
      <span className="text-white font-medium text-xs lg:text-sm body-text tracking-box">FOCUS SESSIONS</span>
      <span className="text-white font-bold text-sm md:text-base lg:text-lg body-text tracking-box">{sessionCount}</span>
      <div className="absolute inset-0 bg-white/5 rounded-xl lg:rounded-2xl opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
    </div>
  )
}
