import { Play, Edit3, AlertOctagon, BarChart3, Zap, Target } from 'lucide-react'
import SynapseHeader from "../layouts/SynapseHeader"
import SynapseHero from "./SynapseHero"
import SynapseActions from "../components/SynapseActions";
import SynapseStatsGrid from "./SynapseStatsGrid"
import SynapseFocusSessions from "./SynapseFocusSessions"
import SynapseStatisticsButton from "../components/SynapseStatisticsButton";
import SynapseDailyGoal from "./SynapseDailyGoal"
import FocusNotification from "../components/FocusNotification"
import { useFocusNotification } from "../hooks/useFocusNotification"

export default function SynapseApp() {
  const { isVisible, position, triggerNotification } = useFocusNotification(6000);
  const currentTime = new Date().toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
  })

  // Handler to trigger notification
  const handleFocusStart = () => {
    console.log("🎯 Focus button clicked - starting notification");
    triggerNotification();
  };

  // Debug log when notification state changes
  console.log("🔍 Current notification state:", isVisible, "Position:", position);

  return (
    <div className="min-h-screen relative overflow-hidden overscroll-none">
      {/* Modular Focus Notification */}
      <FocusNotification
        isVisible={isVisible}
        topOffset={position}
        text="Focus Mode ON"
        bgColor="#D4E84D"
        textColor="#000000"
      />
      
      {/* Background Image with Enhanced Overlay */}
      <div
        className="absolute inset-0 bg-cover bg-center bg-no-repeat"
        style={{
          backgroundImage: `var(--synapse-bg-image)`,
        }}
      >
        <div className="absolute inset-0 bg-gradient-to-br from-black/30 via-black/25 to-black/35" />
      </div>

      {/* Content Overlay */}
      <div className="relative z-10 h-screen flex flex-col overflow-hidden">
        {/* Drag region wrapper */}
        <div data-tauri-drag-region className="px-4 sm:px-5 md:px-7 lg:px-10 xl:px-12 pt-4 sm:pt-5 md:pt-7 lg:pt-10 xl:pt-12">
          <SynapseHeader currentTime={currentTime} />
        </div>

        {/* Main Content Area */}
        <div className="flex-1 flex flex-col md:flex-row items-center justify-between gap-4 md:gap-6 lg:gap-8 xl:gap-12 px-4 sm:px-5 md:px-7 lg:px-10 xl:px-12 overflow-hidden">
          {/* Left Column - Hero */}
          <SynapseHero onStartFocus={handleFocusStart} />

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
        <div className="px-4 sm:px-5 md:px-7 lg:px-10 xl:px-12 pb-4 sm:pb-5 md:pb-7 lg:pb-10 xl:pb-12">
          <SynapseDailyGoal />
        </div>
      </div>
    </div>
  )
}
