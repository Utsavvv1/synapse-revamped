"use client"

import { useState, useEffect, useMemo, useRef } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Edit3, AlertOctagon, X, Check } from "lucide-react"

/**
 * Dropdown component used by both Focus and Distraction app buttons.
 */
const AppDropdown = ({
  isOpen,
  onClose,
  title,
  apps,
  searchTerm,
  onSearchChange,
  onToggleApp,
  className,
  dropdownRef,
}) => {
  const inputRef = useRef(null)

  // Focuses the input when dropdown opens
  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus()
    }
  }, [isOpen])

  if (!isOpen) return null

  return (
    <div
      ref={dropdownRef}
      key={title}
      className={`absolute top-full left-0 right-0 mt-2 z-50 dropdown-enter ${className} rounded-xl`}
      style={{ overflow: "hidden", maxHeight: "none" }}
      onClick={(e) => e.stopPropagation()}
    >
      <div className="w-full p-4 rounded-xl shadow-2xl">
        {/* Dropdown header */}
        <div className="flex items-center justify-between mb-3 dropdown-content-enter">
          <h3 className="text-xs font-semibold truncate">{title}</h3>
          <button
            onClick={(e) => {
              e.stopPropagation()
              onClose()
            }}
            className="p-1 rounded-full hover:bg-black/10 transition-colors duration-150 flex-shrink-0"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Search box */}
        <div className="mb-3 dropdown-content-enter" style={{ animationDelay: "75ms" }}>
          <input
            ref={inputRef}
            type="text"
            placeholder="Search apps..."
            value={searchTerm}
            onChange={(e) => {
              e.stopPropagation()
              onSearchChange(e.target.value)
            }}
            onKeyDown={(e) => {
              e.stopPropagation()
              if (e.key === "Escape") {
                onClose()
              }
            }}
            onFocus={(e) => e.stopPropagation()}
            className="w-full px-3 py-2 text-sm bg-transparent border border-black/20 rounded-md placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-black/20 focus:border-transparent transition-all duration-150"
          />
        </div>

        {/* App list */}
        <div
          className="h-32 overflow-y-auto custom-scrollbar dropdown-content-enter"
          style={{ animationDelay: "100ms" }}
        >
          <div className="space-y-1">
            {apps.map((app, index) => (
              <div
                key={app[1]} // use exe name as key
                onClick={(e) => {
                  e.stopPropagation()
                  onToggleApp(app[1]) // toggle using exe name
                }}
                className="flex items-center justify-between px-3 py-2 text-sm rounded-md hover:bg-black/5 cursor-pointer transition-colors duration-150 select-none dropdown-item-enter gap-3"
                style={{ animationDelay: `${150 + index * 50}ms` }}
              >
                <span className="font-medium truncate flex-1 min-w-0">{app[1]}</span>
                <div className="flex-shrink-0">
                  <Check
                    className={`h-4 w-4 transition-all duration-150 ${
                      app[2] ? "opacity-100 scale-100" : "opacity-0 scale-75"
                    }`}
                  />
                </div>
              </div>
            ))}
            {apps.length === 0 && (
              <div
                className="text-center py-6 text-gray-500 text-sm dropdown-content-enter"
                style={{ animationDelay: "200ms" }}
              >
                No apps found.
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

export default function SynapseActions() {
  // Dropdown states
  const [showFocusDropdown, setShowFocusDropdown] = useState(false)
  const [showDistractionDropdown, setShowDistractionDropdown] = useState(false)

  // Search terms
  const [focusSearchTerm, setFocusSearchTerm] = useState("")
  const [distractionSearchTerm, setDistractionSearchTerm] = useState("")

  // App state
  const [allApps, setAllApps] = useState([])
  const [focusApps, setFocusApps] = useState([])
  const [distractionApps, setDistractionApps] = useState([])

  // Refs for dropdown click detection
  const focusDropdownRef = useRef(null)
  const distractionDropdownRef = useRef(null)

  // Close dropdown on outside click
  useEffect(() => {
    const handleClickOutside = (event) => {
      if (showFocusDropdown && focusDropdownRef.current && !focusDropdownRef.current.contains(event.target)) {
        setShowFocusDropdown(false)
      }
      if (
        showDistractionDropdown &&
        distractionDropdownRef.current &&
        !distractionDropdownRef.current.contains(event.target)
      ) {
        setShowDistractionDropdown(false)
      }
    }

    document.addEventListener("mousedown", handleClickOutside)
    return () => {
      document.removeEventListener("mousedown", handleClickOutside)
    }
  }, [showFocusDropdown, showDistractionDropdown])

  // Fetch installed apps from Tauri backend
  useEffect(() => {
    async function fetchInstalledApps() {
      try {
        const appData = await invoke("get_installed_apps_cmd")
        if (Array.isArray(appData)) {
          // Filter out duplicate .exe names
          const seen = new Set()
          const uniqueApps = appData.filter(([_, exe]) => {
            if (seen.has(exe)) return false
            seen.add(exe)
            return true
          })

          const apps = uniqueApps.map(([name, exe]) => [name, exe, false])
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

  const filteredFocusApps = useMemo(() => {
    return focusApps.filter((app) =>
      app[0].toLowerCase().includes(focusSearchTerm.toLowerCase())
    )
  }, [focusApps, focusSearchTerm])

  const filteredDistractionApps = useMemo(() => {
    return distractionApps.filter((app) =>
      app[0].toLowerCase().includes(distractionSearchTerm.toLowerCase())
    )
  }, [distractionApps, distractionSearchTerm])

  const toggleFocusApp = (exe) => {
    setFocusApps((prevApps) =>
      prevApps.map((app) =>
        app[1] === exe ? [app[0], app[1], !app[2]] : app
      )
    )
  }

  const toggleDistractionApp = (exe) => {
    setDistractionApps((prevApps) =>
      prevApps.map((app) =>
        app[1] === exe ? [app[0], app[1], !app[2]] : app
      )
    )
  }

  const updateAppRules = async () => {
    // Deduplicate using Set to avoid duplicates in apprules.json
    const whitelist = [...new Set(focusApps.filter((app) => app[2]).map((app) => app[1]))]
    const blacklist = [...new Set(distractionApps.filter((app) => app[2]).map((app) => app[1]))]

    console.log("Focus apps (checked):", whitelist)
    console.log("Distraction apps (checked):", blacklist)

    try {
      await invoke("update_app_rules_cmd", { whitelist, blacklist })
      console.log("App rules updated successfully")
    } catch (err) {
      console.error("Failed to update app rules:", err)
    }
  }

  // Update rules after dropdowns close and state settles
  useEffect(() => {
    if (!showFocusDropdown && !showDistractionDropdown) {
      updateAppRules()
    }
  }, [showFocusDropdown, showDistractionDropdown])

  return (
    <div className="relative">
      <div className="flex gap-2 md:gap-2.5 lg:gap-3">
        {/* Focus dropdown trigger */}
        <div className="relative flex-1">
          <button
            onClick={() => {
              setShowFocusDropdown(!showFocusDropdown)
              setShowDistractionDropdown(false)
            }}
            className="group flex items-center justify-center bg-synapse-accent rounded-full text-synapse-dark font-medium body-text w-full relative overflow-hidden hover:bg-synapse-accent/90 transition-all duration-150 px-3 py-1.5 md:px-4 md:py-2 lg:px-5 lg:py-2.5 gap-1 md:gap-1.5 text-xs sm:text-sm whitespace-nowrap"
          >
            <Edit3 className="w-2.5 h-2.5 md:w-3 md:h-3 lg:w-3.5 lg:h-3.5 group-hover:rotate-12 transition-transform duration-150 flex-shrink-0" />
            <span className="hidden lg:inline">Edit Focus Apps</span>
            <span className="lg:hidden">Focus Apps</span>
            <div className="w-0 group-hover:w-1 h-1 bg-synapse-dark rounded-full transition-all duration-150" />
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
            dropdownRef={focusDropdownRef}
          />
        </div>

        {/* Distraction dropdown trigger */}
        <div className="relative flex-1">
          <button
            onClick={() => {
              setShowDistractionDropdown(!showDistractionDropdown)
              setShowFocusDropdown(false)
            }}
            className="group flex items-center justify-center glass-card-dark rounded-full text-synapse-accent font-medium body-text w-full relative overflow-hidden transition-all duration-150 px-3 py-1.5 md:px-4 md:py-2 lg:px-5 lg:py-2.5 gap-1 md:gap-1.5 text-xs sm:text-sm whitespace-nowrap"
            style={{ backgroundColor: "hsl(var(--secondarycolor))" }}
          >
            <AlertOctagon className="w-2.5 h-2.5 md:w-3 md:h-3 lg:w-3.5 lg:h-3.5 group-hover:shake transition-transform duration-150 flex-shrink-0" />
            <span className="hidden lg:inline">Edit Distraction Apps</span>
            <span className="lg:hidden">Distractions</span>
            <div className="w-0 group-hover:w-1 h-1 bg-synapse-accent rounded-full transition-all duration-150" />
          </button>

          <AppDropdown
            isOpen={showDistractionDropdown}
            onClose={() => setShowDistractionDropdown(false)}
            title="Edit Distraction Apps"
            apps={filteredDistractionApps}
            searchTerm={distractionSearchTerm}
            onSearchChange={setDistractionSearchTerm}
            onToggleApp={toggleDistractionApp}
            className="bg-synapse-dark text-synapse-accent"
            dropdownRef={distractionDropdownRef}
          />
        </div>
      </div>
    </div>
  )
}

