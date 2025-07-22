import { Play, Edit3, AlertOctagon, BarChart3, Zap, Target } from "lucide-react"
import SynapseHeader from "./components/Custom/SynapseHeader"
import SynapseHero from "./components/Custom/SynapseHero"
import SynapseActions from "./components/Custom/SynapseActions"
import SynapseStatsGrid from "./components/Custom/SynapseStatsGrid"
import SynapseFocusSessions from "./components/Custom/SynapseFocusSessions"
import SynapseStatisticsButton from "./components/Custom/SynapseStatisticsButton"
import SynapseDailyGoal from "./components/Custom/SynapseDailyGoal"

export default function SynapseApp() {
  const currentTime = new Date().toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
  })

  return (
    <div className="min-h-screen relative overflow-hidden">
      {/* Background Image with Enhanced Overlay */}
      <div
        className="absolute inset-0 bg-cover bg-center bg-no-repeat"
        style={{
          backgroundImage: `url('/images/rice-terraces-bg.jpg')`,
        }}
      >
        <div className="absolute inset-0 bg-gradient-to-br from-black/30 via-black/25 to-black/35" />
      </div>

      {/* Content Overlay */}
      <div className="relative z-10 h-screen flex flex-col p-4 sm:p-5 md:p-7 lg:p-10 xl:p-12">
        {/* Compact Header */}
        <SynapseHeader currentTime={currentTime} />

        {/* Main Content Area */}
        <div className="flex-1 flex flex-col md:flex-row items-center justify-between gap-4 md:gap-6 lg:gap-8 xl:gap-12">
          {/* Left Column - Hero */}
          <SynapseHero />

          {/* Right Column - Stats and Controls */}
          <div className="flex flex-col gap-2 md:gap-2.5 lg:gap-3 w-full md:w-80 lg:w-96 xl:w-[26rem] flex-shrink-0">
            {/* Action Buttons */}
            <SynapseActions />

            {/* Enhanced Stats Grid */}
            <SynapseStatsGrid />

            {/* Enhanced Focus Sessions */}
            <SynapseFocusSessions />

            {/* Enhanced Statistics Button */}
            <SynapseStatisticsButton />
          </div>
        </div>

        {/* Enhanced Footer - Daily Goal */}
        <SynapseDailyGoal />
      </div>
    </div>
  )
}