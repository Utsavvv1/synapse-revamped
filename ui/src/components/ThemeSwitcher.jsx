"use client"

import React, { useState, useEffect } from "react"
import { ChevronLeft, ChevronRight } from "lucide-react"
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "../components/popover"
import { applyTheme } from "../theme-loader"

const THEMES = ["purple", "green", "blue", "black"]

export function ThemeSwitcher({ children }) {
    // Initialize with 'purple' or try to guess/save in localStorage if desired.
    // Since theme-loader applies 'purple' by default, we start there.
    const [currentIndex, setCurrentIndex] = useState(0)
    const [isOpen, setIsOpen] = useState(false)

    // Optional: Load saved theme from localStorage on mount
    useEffect(() => {
        const savedTheme = localStorage.getItem("synapse-theme")
        if (savedTheme) {
            const index = THEMES.indexOf(savedTheme)
            if (index !== -1) {
                setCurrentIndex(index)
                applyTheme(savedTheme).catch(console.error)
            }
        }
    }, [])

    const handlePrev = () => {
        const newIndex = (currentIndex - 1 + THEMES.length) % THEMES.length
        changeTheme(newIndex)
    }

    const handleNext = () => {
        const newIndex = (currentIndex + 1) % THEMES.length
        changeTheme(newIndex)
    }

    const changeTheme = (index) => {
        setCurrentIndex(index)
        const themeName = THEMES[index]
        applyTheme(themeName).catch(console.error)
        localStorage.setItem("synapse-theme", themeName)
    }

    const currentThemeName = THEMES[currentIndex].charAt(0).toUpperCase() + THEMES[currentIndex].slice(1)

    return (
        <Popover open={isOpen} onOpenChange={setIsOpen}>
            <PopoverTrigger asChild>
                {/* We pass the trigger child (the profile circle) */}
                <div className="cursor-pointer" onClick={() => setIsOpen(true)}>
                    {children}
                </div>
            </PopoverTrigger>
            <PopoverContent className="w-48 p-3 bg-synapse-dark border-synapse-border" side="bottom" align="end">
                <div className="flex flex-col gap-2">
                    <span className="text-xs font-medium text-gray-400 uppercase tracking-wider text-center">Theme</span>
                    <div className="flex items-center justify-between bg-black/20 rounded-lg p-1">
                        <button
                            onClick={handlePrev}
                            className="p-1 hover:bg-white/10 rounded-md transition-colors"
                        >
                            <ChevronLeft className="w-4 h-4 text-white" />
                        </button>
                        <span className="text-sm font-medium text-white min-w-[60px] text-center select-none">
                            {currentThemeName}
                        </span>
                        <button
                            onClick={handleNext}
                            className="p-1 hover:bg-white/10 rounded-md transition-colors"
                        >
                            <ChevronRight className="w-4 h-4 text-white" />
                        </button>
                    </div>
                </div>
            </PopoverContent>
        </Popover>
    )
}
