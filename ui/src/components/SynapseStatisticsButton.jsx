import React from "react"
import { BarChart3 } from "lucide-react"
import { useNavigate } from "react-router-dom"

export default function SynapseStatisticsButton() {
  const navigate = useNavigate();

  return (
    <button
      onClick={() => navigate('/statistics')}
      className="group flex items-center justify-center bg-synapse-accent rounded-full text-synapse-dark font-medium body-text relative overflow-hidden transition-all duration-300 px-4 py-2 md:px-5 md:py-2.5 lg:px-6 lg:py-3 gap-2 text-xs md:text-sm lg:text-sm"
    >
      <BarChart3 className="w-3 h-3 md:w-3.5 md:h-3.5 lg:w-4 lg:h-4 transition-all duration-300" />
      Show Statistics
      <div className="w-0 group-hover:w-1 h-1 bg-synapse-dark rounded-full transition-all duration-300" />
      <div className="absolute inset-0 bg-white/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
    </button>
  )
}
