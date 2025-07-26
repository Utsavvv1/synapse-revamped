import React, { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Edit3, AlertOctagon, X, Check } from "lucide-react"

export default function SynapseActions() {
  const [showFocusDropdown, setShowFocusDropdown] = useState(false)
  const [showDistractionDropdown, setShowDistractionDropdown] = useState(false)
  const [focusSearchTerm, setFocusSearchTerm] = useState("")
  const [distractionSearchTerm, setDistractionSearchTerm] = useState("")
  const [allApps, setAllApps] = useState([])

  const [focusApps, setFocusApps] = useState([])
  const [distractionApps, setDistractionApps] = useState([])

  useEffect(() => {
    async function fetchInstalledApps() {
      try {
        const appNames = await invoke("get_installed_apps_cmd")
        if (Array.isArray(appNames)) {
          const apps = appNames.map((name, index) => ({
            id: index + 1,
            name,
            checked: false
          }))
          setAllApps(apps)
          setFocusApps(apps.slice(0, apps.length / 2))
          setDistractionApps(apps.slice(apps.length / 2))
        }
      } catch (err) {
        console.error("Failed to fetch apps from Tauri backend:", err)
      }
    }

    fetchInstalledApps()
  }, [])

  const filteredFocusApps = focusApps.filter(app =>
    app.name.toLowerCase().includes(focusSearchTerm.toLowerCase())
  )

  const filteredDistractionApps = distractionApps.filter(app =>
    app.name.toLowerCase().includes(distractionSearchTerm.toLowerCase())
  )

  const toggleFocusApp = (id) => {
    setFocusApps(apps =>
      apps.map(app => app.id === id ? { ...app, checked: !app.checked } : app)
    )
  }

  const toggleDistractionApp = (id) => {
    setDistractionApps(apps =>
      apps.map(app => app.id === id ? { ...app, checked: !app.checked } : app)
    )
  }

  const AppDropdown = ({
    isOpen,
    onClose,
    title,
    apps,
    searchTerm,
    onSearchChange,
    onToggleApp,
    className
  }) => {
    if (!isOpen) return null

    return (
      <div className="absolute top-full left-0 right-0 mt-2 z-50 dropdown-enter">
        <div className={`w-full p-4 rounded-xl shadow-2xl ${className}`}>
          <div className="flex items-center justify-between mb-3 dropdown-content-enter">
            <h3 className="text-xs font-semibold truncate">{title}</h3>
            <button
              onClick={onClose}
              className="p-1 rounded-full hover:bg-black/10 transition-colors duration-150 flex-shrink-0"
            >
              <X className="w-4 h-4" />
            </button>
          </div>

          <div className="mb-3 dropdown-content-enter" style={{ animationDelay: '75ms' }}>
            <input
              type="text"
              placeholder="Search apps..."
              value={searchTerm}
              onChange={(e) => onSearchChange(e.target.value)}
              className="w-full px-3 py-2 text-sm bg-transparent border border-black/20 rounded-md placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-black/20 focus:border-transparent transition-all duration-150"
            />
          </div>

          <div className="h-32 overflow-y-auto custom-scrollbar dropdown-content-enter" style={{ animationDelay: '100ms' }}>
            <div className="space-y-1">
              {apps.map((app, index) => (
                <div
                  key={app.id}
                  onClick={() => onToggleApp(app.id)}
                  className="flex items-center justify-between px-3 py-2 text-sm rounded-md hover:bg-black/5 cursor-pointer transition-colors duration-150 select-none dropdown-item-enter"
                  style={{ animationDelay: `${150 + index * 50}ms` }}
                >
                  <span className="font-medium truncate">{app.name}</span>
                  <Check
                    className={`h-4 w-4 transition-all duration-150 ${
                      app.checked ? 'opacity-100 scale-100' : 'opacity-0 scale-75'
                    }`}
                  />
                </div>
              ))}
              {apps.length === 0 && (
                <div className="text-center py-6 text-gray-500 text-sm dropdown-content-enter" style={{ animationDelay: '200ms' }}>
                  No apps found.
                </div>
              )}
            </div>
          </div>

          <div className={`absolute right-0 top-4 bottom-4 w-1 rounded-full dropdown-content-enter ${
            className.includes('bg-synapse-accent') ? 'bg-synapse-dark/20' : 'bg-synapse-accent/30'
          }`} style={{ animationDelay: '200ms' }} />
        </div>
      </div>
    )
  }

  return (
    <div className="relative">
      <div className="flex gap-2 md:gap-2.5 lg:gap-3">
        <div className="relative flex-1">
          <button
            onClick={() => {
              setShowFocusDropdown(!showFocusDropdown)
              setShowDistractionDropdown(false)
            }}
            className="group flex items-center justify-center bg-synapse-accent rounded-full text-synapse-dark font-medium body-text w-full relative overflow-hidden hover:bg-synapse-accent/90 transition-all duration-300 px-3 py-1.5 md:px-4 md:py-2 lg:px-5 lg:py-2.5 gap-1 md:gap-1.5 text-xs sm:text-sm whitespace-nowrap"
          >
            <Edit3 className="w-2.5 h-2.5 md:w-3 md:h-3 lg:w-3.5 lg:h-3.5 group-hover:rotate-12 transition-transform duration-300 flex-shrink-0" />
            <span className="hidden lg:inline">Edit Focus Apps</span>
            <span className="lg:hidden">Focus Apps</span>
            <div className="w-0 group-hover:w-1 h-1 bg-synapse-dark rounded-full transition-all duration-300" />
          </button>

          <AppDropdown
            isOpen={showFocusDropdown}
            onClose={() => setShowFocusDropdown(false)}
            title="Edit Focus Apps"
            apps={filteredFocusApps}
            searchTerm={focusSearchTerm}
            onSearchChange={setFocusSearchTerm}
            onToggleApp={toggleFocusApp}
            className="bg-synapse-accent text-synapse-dark"
          />
        </div>

        <div className="relative flex-1">
          <button
            onClick={() => {
              setShowDistractionDropdown(!showDistractionDropdown)
              setShowFocusDropdown(false)
            }}
            className="group flex items-center justify-center glass-card-dark rounded-full text-synapse-accent font-medium body-text w-full relative overflow-hidden transition-all duration-300 px-3 py-1.5 md:px-4 md:py-2 lg:px-5 lg:py-2.5 gap-1 md:gap-1.5 text-xs sm:text-sm whitespace-nowrap"
            style={{ backgroundColor: 'hsl(var(--secondarycolor))' }}
          >
            <AlertOctagon className="w-2.5 h-2.5 md:w-3 md:h-3 lg:w-3.5 lg:h-3.5 group-hover:shake transition-transform duration-300 flex-shrink-0" />
            <span className="hidden lg:inline">Edit Distraction Apps</span>
            <span className="lg:hidden">Distractions</span>
            <div className="w-0 group-hover:w-1 h-1 bg-synapse-accent rounded-full transition-all duration-300" />
          </button>

          <AppDropdown
            isOpen={showDistractionDropdown}
            onClose={() => setShowDistractionDropdown(false)}
            title="Edit Distraction Apps"
            apps={filteredDistractionApps}
            searchTerm={distractionSearchTerm}
            onSearchChange={setDistractionSearchTerm}
            onToggleApp={toggleDistractionApp}
            className="bg-synapse-dark-alt text-synapse-accent border border-synapse-accent/20"
          />
        </div>
      </div>
    </div>
  )
}