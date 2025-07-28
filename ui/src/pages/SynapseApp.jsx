import { Play, Edit3, AlertOctagon, BarChart3, Zap, Target } from 'lucide-react'
import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import SynapseHeader from "../layouts/SynapseHeader"
import SynapseHero from "./SynapseHero"
import SynapseActions from "../components/SynapseActions";
import SynapseStatsGrid from "./SynapseStatsGrid"
import SynapseFocusSessions from "./SynapseFocusSessions"
import SynapseStatisticsButton from "../components/SynapseStatisticsButton";
import SynapseDailyGoal from "./SynapseDailyGoal"
import FocusNotification from "../components/FocusNotification"
import AppBlockModal from "../components/AppBlockModal"
import { useFocusNotification } from "../hooks/useFocusNotification"
import { useAppBlockModal } from "../hooks/useAppBlockModal"

export default function SynapseApp() {
  const { isVisible, position, triggerNotification } = useFocusNotification(6000);
  const { 
    isVisible: isBlockModalVisible, 
    showModal: showBlockModal, 
    handleCloseApp, 
    handleUseFor5Mins,
    handleShowAgain
  } = useAppBlockModal();
  
  const currentTime = new Date().toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
  })

  // Set up distraction event listener
  useEffect(() => {
    let unlisten;
    
    const setupListener = async () => {
      try {
        unlisten = await listen('show-distraction-modal', (event) => {
          console.log('🚫 Distraction detected:', event.payload);
          showBlockModal(); // Show your custom modal!
        });
        console.log('✅ Distraction event listener set up');
      } catch (error) {
        console.error('❌ Failed to set up distraction listener:', error);
      }
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [showBlockModal]);

  // Handler to trigger notification
  const handleFocusStart = () => {
    console.log("🎯 Focus button clicked - starting notification");
    triggerNotification();
  };

  // Handler to trigger block modal (for testing)
  const handleTriggerBlockModal = () => {
    console.log("🚫 Triggering block modal");
    showBlockModal();
  };

  // Debug log when notification state changes
  console.log("🔍 Current notification state:", isVisible, "Position:", position);

  return (
    <div className="min-h-screen relative overflow-hidden overscroll-none">
      {/* Modular Focus Notification */}
      <FocusNotification
        isVisible={isVisible}
        topOffset={position}
        text="Monitoring ON"
        bgColor="#D4E84D"
        textColor="#000000"
      />
      
      {/* App Block Modal */}
      <AppBlockModal
        isVisible={isBlockModalVisible}
        onClose={handleCloseApp}
        onUseFor5Mins={handleUseFor5Mins}
        onShowAgain={handleShowAgain}
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

            {/* Test Button for Block Modal */}
            <button
              onClick={handleTriggerBlockModal}
              className="px-4 py-2 bg-red-500 text-white rounded-lg hover:bg-red-600 transition-colors text-sm"
            >
              Test Block Modal
            </button>

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
