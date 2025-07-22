import { Play, Edit3, AlertOctagon, BarChart3, Zap, Target } from "lucide-react"

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
        <header className="flex justify-between items-center mb-4 lg:mb-6">
          <h1 className="font-serif italic text-synapse-dark text-xl sm:text-2xl lg:text-3xl font-light tracking-tight">
            Synapse
          </h1>

          <div className="flex items-center gap-2 sm:gap-3 lg:gap-4">
            <span className="font-medium text-white text-base sm:text-lg lg:text-xl body-text">{currentTime}</span>
            <div className="phone-connected rounded-full px-4 py-2 gap-2 flex items-center transform scale-90">
              <span className="text-gray-200 font-medium text-sm body-text mr-2">Phone Connected</span> {/* Added mr-2 for gap after text */}
              <div className="relative flex items-center gap-2"> {/* Added gap-2 for space after dot and circle */}
                <div className="w-6 h-6 bg-synapse-dark-alt rounded-full flex items-center justify-center mr-1 ml-2"> {/* Moved darkalt circle further right with ml-2 */}
                  <div className="w-1.5 h-1.5 bg-green-400 rounded-full absolute -left-2 pulse-dot" />
                </div>
              </div>
            </div>
          </div>
        </header>

        {/* Main Content Area */}
        <div className="flex-1 flex flex-col md:flex-row items-center justify-between gap-4 md:gap-6 lg:gap-8 xl:gap-12">
          {/* Left Column - Hero */}
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

          {/* Right Column - Stats and Controls */}
          <div className="flex flex-col gap-2 md:gap-2.5 lg:gap-3 w-full md:w-80 lg:w-96 xl:w-[26rem] flex-shrink-0">
            {/* Action Buttons */}
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

            {/* Enhanced Stats Grid */}
            <div className="flex gap-2 md:gap-2.5 lg:gap-3">
              <div className="glass-card flex flex-col justify-center items-center px-4 py-3 lg:px-5 lg:py-4 rounded-xl lg:rounded-2xl group cursor-pointer flex-[1.6]">
                <span className="text-white font-medium text-xs body-text tracking-wide opacity-80 mb-1">
                  DEEP WORK
                </span>
                <span className="text-white font-bold text-base md:text-lg lg:text-xl xl:text-2xl body-text group-hover:scale-110 transition-transform duration-300 mb-1">
                  3h 12m
                </span>
                <span className="text-white/60 text-xs body-text tracking-wide">Today</span>
                <div className="absolute inset-0 shimmer rounded-xl lg:rounded-2xl opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
              </div>

              <div className="glass-card flex flex-col justify-center items-center flex-1 text-center px-4 py-3 lg:px-5 lg:py-4 rounded-xl lg:rounded-2xl group cursor-pointer">
                <span className="text-white font-medium text-xs body-text tracking-wide opacity-80 mb-1">
                  DISTRACTIONS
                </span>
                <span className="text-white font-bold text-base md:text-lg lg:text-xl xl:text-2xl body-text group-hover:scale-110 transition-transform duration-300 mb-1">
                  9
                </span>
                <span className="text-white/60 text-xs body-text tracking-wide">Blocked</span>
                <div className="absolute inset-0 shimmer rounded-xl lg:rounded-2xl opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
              </div>
            </div>

            {/* Enhanced Focus Sessions */}
            <div className="glass-card flex items-center justify-center p-2 md:p-2.5 lg:p-3 rounded-xl lg:rounded-2xl gap-2 md:gap-3 group cursor-pointer hover:scale-105 transition-all duration-300">
              <Target className="w-3 h-3 md:w-3.5 md:h-3.5 text-synapse-accent opacity-80 group-hover:opacity-100 group-hover:rotate-180 transition-all duration-500" />
              <span className="text-white font-medium text-xs lg:text-sm body-text tracking-wide">FOCUS SESSIONS</span>
              <span className="text-white font-bold text-sm md:text-base lg:text-lg body-text">4</span>
            </div>

            {/* Enhanced Statistics Button */}
            <button className="group flex items-center justify-center bg-synapse-accent rounded-full text-synapse-dark font-medium body-text hover:bg-synapse-accent/90 transition-all duration-300 px-4 py-2 md:px-5 md:py-2.5 lg:px-6 lg:py-3 gap-2 text-xs md:text-sm lg:text-sm glow-accent hover:scale-105">
              <BarChart3 className="w-3 h-3 md:w-3.5 md:h-3.5 lg:w-4 lg:h-4 group-hover:scale-110 transition-transform duration-300" />
              Show Statistics
              <div className="w-0 group-hover:w-1 h-1 bg-synapse-dark rounded-full transition-all duration-300" />
            </button>
          </div>
        </div>

        {/* Enhanced Footer - Daily Goal */}
        <div className="flex flex-col items-center gap-2 mt-4 lg:mt-6">
          <div className="flex items-center gap-2">
            <h3 className="text-white font-medium text-sm lg:text-base body-text">Daily Focus Goal</h3>
            <span className="text-synapse-accent text-xs font-medium">70%</span>
          </div>
          <div className="w-full max-w-xs sm:max-w-sm lg:max-w-md progress-bar rounded-full h-1.5 relative">
            <div className="progress-fill rounded-full h-1 absolute top-0.5 left-0.5 w-[70%] transition-all duration-1000 ease-out" />
          </div>
        </div>
      </div>
    </div>
  )
}