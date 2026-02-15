import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import {
    format,
    addMonths,
    subMonths,
    startOfMonth,
    endOfMonth,
    startOfWeek,
    endOfWeek,
    eachDayOfInterval,
    isSameMonth,
    isToday
} from 'date-fns';
import { SkipBack, SkipForward, Play, Pause, ArrowLeft } from 'lucide-react';
import SynapseHeader from '../layouts/SynapseHeader';
import { useSpotify } from '../hooks/useSpotify';

// Helper to format ms to m:ss
const formatTime = (ms: number) => {
    const totalSeconds = Math.floor(ms / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
};

interface DashboardData {
    weeklySummary: {
        totalTime: string;
        history: { day: string; hours: number }[];
    };
    stats: {
        sessions: number;
        average: string;
        longest: string;
    };
    dailyGoal: {
        target: string;
        progress: number; // 0-100
    };
    distractions: {
        blockedCount: number;
        topDistractions: { name: string; time: string }[];
    };
    tasks: { title: string; completed: boolean }[];
    streak: number;
}

const MOCK_DASHBOARD_DATA: DashboardData = {
    weeklySummary: {
        totalTime: "34h 20m",
        history: [
            { day: 'M', hours: 0.5 },
            { day: 'T', hours: 2.2 },
            { day: 'W', hours: 0.8 },
            { day: 'Th', hours: 0.5 },
            { day: 'F', hours: 0.1 },
            { day: 'S', hours: 0.3 },
            { day: 'S', hours: 0.5 },
        ]
    },
    stats: {
        sessions: 35,
        average: "2h 30m",
        longest: "4h 15m"
    },
    dailyGoal: {
        target: "2h 45m",
        progress: 65
    },
    distractions: {
        blockedCount: 18,
        topDistractions: [
            { name: "YouTube", time: "1h 20m" },
            { name: "Twitter", time: "45m" },
            { name: "Reddit", time: "30m" }
        ]
    },
    tasks: [
        { title: "Finish 25 graph questions", completed: false },
        { title: "Review pull requests", completed: true },
        { title: "Update system docs", completed: false },
        { title: "Team sync meeting", completed: false }
    ],
    streak: 6
};

export default function StatisticsPage() {
    const navigate = useNavigate();
    const { track, user, progress, login, logout, isAuthenticated, togglePlayback, skipNext, skipPrevious, seek } = useSpotify();
    const [currentMonth, setCurrentMonth] = useState(new Date());
    const [currentTime, setCurrentTime] = useState(new Date().toLocaleTimeString("en-US", {
        hour12: false,
        hour: "2-digit",
        minute: "2-digit",
    }));

    // IDLE STATE LOGIC
    const [isIdle, setIsIdle] = useState(false);

    useEffect(() => {
        let timer: ReturnType<typeof setTimeout>;

        if (isAuthenticated && (!track || !track.is_playing)) {
            // If authenticated but not playing (or no track), start idle timer
            timer = setTimeout(() => {
                setIsIdle(true);
            }, 10000); // 10 seconds
        } else {
            // Playing -> Reset idle
            setIsIdle(false);
        }

        return () => clearTimeout(timer);
    }, [isAuthenticated, track?.is_playing]);

    // Local state to handle slider dragging smoothly
    const [isDragging, setIsDragging] = useState(false);
    const [dragValue, setDragValue] = useState(0);

    useEffect(() => {
        const timer = setInterval(() => {
            setCurrentTime(new Date().toLocaleTimeString("en-US", {
                hour12: false,
                hour: "2-digit",
                minute: "2-digit",
            }));
        }, 1000);
        return () => clearInterval(timer);
    }, []);

    const handlePrevMonth = () => setCurrentMonth(subMonths(currentMonth, 1));
    const handleNextMonth = () => setCurrentMonth(addMonths(currentMonth, 1));

    // Generate Calendar Days
    const monthStart = startOfMonth(currentMonth);
    const monthEnd = endOfMonth(monthStart);
    const startDate = startOfWeek(monthStart, { weekStartsOn: 1 });
    const endDate = endOfWeek(monthEnd, { weekStartsOn: 1 });
    const calendarDays = eachDayOfInterval({ start: startDate, end: endDate });

    const [windowDimensions, setWindowDimensions] = useState({
        width: window.innerWidth,
        height: window.innerHeight
    });

    useEffect(() => {
        const handleResize = () => setWindowDimensions({
            width: window.innerWidth,
            height: window.innerHeight
        });
        window.addEventListener('resize', handleResize);
        return () => window.removeEventListener('resize', handleResize);
    }, []);

    const { width: windowWidth, height: windowHeight } = windowDimensions;

    const gridDensityTier = windowWidth < 1024 ? 'small' : windowWidth < 1280 ? 'medium' : 'large';

    return (
        <div
            className="h-screen w-screen p-2 sm:p-3 md:p-4 lg:p-6 bg-black bg-cover bg-center bg-no-repeat overflow-hidden flex flex-col font-sans selection:bg-lime/30"
            style={{
                backgroundImage: `linear-gradient(rgba(0,0,0,0.5), rgba(0,0,0,0.5)), var(--synapse-bg-image)`,
                backgroundAttachment: 'fixed'
            }}
        >
            {/* Drag region wrapper matching main page */}
            <div data-tauri-drag-region className="mb-2 sm:mb-3 md:mb-4">
                <SynapseHeader currentTime={currentTime} />
            </div>
            <div className="max-w-[1800px] w-full mx-auto flex-1 flex flex-col min-h-0">
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2 sm:gap-3 md:gap-4 flex-1 min-h-0 overflow-y-auto">

                    {/* COLUMN 1: Dashboard Title + Weekly Summary + Stats Row */}
                    <div className="flex flex-col gap-2 sm:gap-3 md:gap-4 min-h-0">
                        <div className="flex items-center gap-2 sm:gap-3 md:gap-4 flex-shrink-0">
                            <button
                                onClick={() => navigate('/')}
                                className="p-1.5 sm:p-2 md:p-2.5 bg-dark-bg hover:bg-dark-green text-lime rounded-full transition-all flex items-center justify-center shadow-lg active:scale-95 group"
                                title="Back to Dashboard"
                            >
                                <ArrowLeft className="w-4 h-4 sm:w-5 sm:h-5 md:w-6 md:h-6 group-hover:-translate-x-1 transition-transform" />
                            </button>
                            <h1 className="text-2xl sm:text-3xl md:text-4xl lg:text-5xl font-semibold text-white tracking-tighter leading-none flex-shrink-0">
                                Dashboard
                            </h1>
                        </div>

                        <div className="bg-dark-bg rounded-lg sm:rounded-xl md:rounded-2xl p-3 sm:p-4 md:p-5 flex flex-col justify-between flex-1 min-h-0">
                            <div className="flex flex-col mb-6 sm:mb-8">
                                <h2 className="text-sm sm:text-base md:text-lg lg:text-xl font-semibold text-lime tracking-tight leading-tight">
                                    Weekly Summary
                                </h2>
                                <p className="text-xs sm:text-sm md:text-base text-lime/60 font-medium">Total Week's Time: <span className="text-lime/90 font-bold">{MOCK_DASHBOARD_DATA.weeklySummary.totalTime}</span></p>
                            </div>

                            {/* Bar Graph Container */}
                            <div className="relative flex-1 min-h-0 flex flex-col">
                                {(() => {
                                    const maxHours = Math.max(...MOCK_DASHBOARD_DATA.weeklySummary.history.map(d => d.hours), 0.1);

                                    // Robust algorithm for "nice" intervals (e.g., 0.5, 1, 2, 5, 10, 20, 50, 100...)
                                    const getNiceInterval = (max: number) => {
                                        const rawInterval = max / 10;
                                        const magnitude = Math.pow(10, Math.floor(Math.log10(rawInterval)));
                                        const normalized = rawInterval / magnitude;

                                        let step;
                                        if (normalized < 1.5) step = 1;
                                        else if (normalized < 3) step = 2;
                                        else if (normalized < 7.5) step = 5;
                                        else step = 10;

                                        return Math.max(0.5, step * magnitude);
                                    };

                                    const baseInterval = getNiceInterval(maxHours);

                                    // 3-tier density: Full (>1280), Half (1024-1280), Quarter (<1024)
                                    const intervalMultiplier = gridDensityTier === 'small' ? 4 : gridDensityTier === 'medium' ? 2 : 1;
                                    const interval = baseInterval * intervalMultiplier;

                                    const steps = Math.ceil(maxHours / interval);
                                    const chartMax = Math.max(interval, steps * interval);
                                    const gridValues = Array.from({ length: steps + 1 }, (_, i) => (steps - i) * interval);

                                    return (
                                        <>
                                            {/* Grid Lines Overlay */}
                                            <div className="absolute inset-x-0 top-0 bottom-6 pointer-events-none">
                                                {gridValues.map((val, i) => (
                                                    <div
                                                        key={i}
                                                        className="absolute left-0 right-0 h-0 flex items-center"
                                                        style={{ top: `${(i / steps) * 100}%` }}
                                                    >
                                                        <span className="text-[7px] sm:text-[8px] text-white/20 font-medium whitespace-nowrap min-w-[28px] -translate-y-1/2">
                                                            {val % 1 === 0 ? `${val}h` : `${val.toFixed(1)}h`}
                                                        </span>
                                                        <div className="flex-1 border-t border-white/10 sm:border-white/20 border-dashed"></div>
                                                    </div>
                                                ))}
                                            </div>

                                            {/* Bars Overlay - Perfectly aligned with Grid Area */}
                                            <div className="absolute inset-x-0 top-0 bottom-6 left-8 right-1 flex items-end justify-between gap-1 sm:gap-2 z-10">
                                                {MOCK_DASHBOARD_DATA.weeklySummary.history.map((item, i) => {
                                                    const heightPercent = chartMax > 0 ? (item.hours / chartMax) * 100 : 0;
                                                    const todayIndex = (new Date().getDay() + 6) % 7;
                                                    const isToday = i === todayIndex;

                                                    return (
                                                        <div key={i} className="flex-1 h-full flex flex-col justify-end items-center relative">
                                                            <div
                                                                className={`group w-full max-w-[14px] sm:max-w-[18px] md:max-w-[24px] ${isToday ? 'bg-lime' : 'bg-lime/20'} rounded-[3px] relative transition-all duration-300`}
                                                                style={{ height: `${heightPercent}%` }}
                                                            >
                                                                {/* Hover Label */}
                                                                <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-1 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-20">
                                                                    <div className="bg-lime text-synapse-dark text-[9px] sm:text-[10px] font-bold px-1.5 py-0.5 rounded shadow-lg whitespace-nowrap">
                                                                        {item.hours % 1 === 0 ? `${item.hours}h` : `${item.hours.toFixed(1)}h`}
                                                                    </div>
                                                                </div>
                                                                <div className="absolute inset-0 bg-gradient-to-t from-black/10 to-transparent"></div>
                                                            </div>
                                                        </div>
                                                    );
                                                })}
                                            </div>

                                            {/* Day Labels - Outside the bar/grid scaling area */}
                                            <div className="mt-auto flex justify-between gap-1 sm:gap-2 pl-8 pr-1 h-6">
                                                {MOCK_DASHBOARD_DATA.weeklySummary.history.map((item, i) => {
                                                    const todayIndex = (new Date().getDay() + 6) % 7;
                                                    const isToday = i === todayIndex;
                                                    return (
                                                        <div key={i} className="flex-1 flex justify-center items-end">
                                                            <span className={`text-[8px] sm:text-[10px] ${isToday ? 'text-white font-bold' : 'text-white/40 font-medium'}`}>
                                                                {item.day}
                                                            </span>
                                                        </div>
                                                    );
                                                })}
                                            </div>
                                        </>
                                    );
                                })()}
                            </div>
                        </div>

                        {/* Row of Sessions, Average, Longest */}
                        <div className="grid grid-cols-3 gap-1 sm:gap-1.5 md:gap-2 flex-shrink-0">
                            <div className="bg-lime rounded-md sm:rounded-lg md:rounded-xl p-1.5 sm:p-2 md:p-3 flex flex-col justify-between min-h-[50px] sm:min-h-[60px] md:min-h-[80px]">
                                <h3 className="text-[9px] sm:text-[10px] md:text-xs font-semibold text-synapse-dark tracking-tight">Sessions</h3>
                                <p className="text-sm sm:text-base md:text-lg lg:text-xl font-semibold text-synapse-dark tracking-tight leading-none">{MOCK_DASHBOARD_DATA.stats.sessions}</p>
                            </div>

                            <div className="bg-lime rounded-md sm:rounded-lg md:rounded-xl p-1.5 sm:p-2 md:p-3 flex flex-col gap-0.5 sm:gap-1 min-h-[50px] sm:min-h-[60px] md:min-h-[80px]">
                                <h3 className="text-[9px] sm:text-[10px] md:text-xs font-semibold text-synapse-dark tracking-tight">Average</h3>
                                <div className="bg-white/10 rounded py-0.5 sm:py-1 flex items-center justify-center mt-auto">
                                    <span className="text-[9px] sm:text-[10px] md:text-xs font-semibold text-synapse-dark tracking-tight leading-none">{MOCK_DASHBOARD_DATA.stats.average}</span>
                                </div>
                            </div>

                            <div className="bg-lime rounded-md sm:rounded-lg md:rounded-xl p-1.5 sm:p-2 md:p-3 flex flex-col gap-0.5 sm:gap-1 min-h-[50px] sm:min-h-[60px] md:min-h-[80px]">
                                <h3 className="text-[9px] sm:text-[10px] md:text-xs font-semibold text-synapse-dark tracking-tight">Longest</h3>
                                <div className="bg-white/10 rounded py-0.5 sm:py-1 flex items-center justify-center mt-auto">
                                    <span className="text-[9px] sm:text-[10px] md:text-xs font-semibold text-synapse-dark tracking-tight leading-none">{MOCK_DASHBOARD_DATA.stats.longest}</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* COLUMN 2: Daily Goal + Distractions + Task List + Spotify */}
                    <div className="flex flex-col gap-1.5 sm:gap-2 md:gap-3 h-full md:row-span-2 lg:row-span-1">
                        <div className="bg-dark-bg rounded-lg sm:rounded-xl md:rounded-2xl p-1.5 sm:p-2 md:p-2.5 flex flex-col justify-between flex-shrink-0">
                            <div className="flex justify-between items-start">
                                <div>
                                    <h2 className="text-xs sm:text-sm md:text-base font-semibold text-lime tracking-tight leading-tight">Daily Goal</h2>
                                    <p className="text-[10px] sm:text-xs md:text-sm text-lime/80">{MOCK_DASHBOARD_DATA.dailyGoal.target}</p>
                                </div>
                                <div className="text-right">
                                    <p className="text-xs sm:text-sm font-bold text-lime">{MOCK_DASHBOARD_DATA.dailyGoal.progress}%</p>
                                </div>
                            </div>
                            <div className="flex justify-center mt-2 sm:mt-4 relative h-16 sm:h-20 md:h-24 overflow-hidden">
                                <svg className="w-24 h-24 sm:w-32 sm:h-32 md:w-40 md:h-40 -rotate-[180deg]" viewBox="0 0 100 100">
                                    {/* Background Arc */}
                                    <circle
                                        cx="50"
                                        cy="50"
                                        r="40"
                                        fill="none"
                                        stroke="currentColor"
                                        strokeWidth="10"
                                        strokeDasharray="125.6 251.2"
                                        strokeLinecap="round"
                                        className="text-white/10"
                                    />
                                    {/* Progress Arc */}
                                    <circle
                                        cx="50"
                                        cy="50"
                                        r="40"
                                        fill="none"
                                        stroke="currentColor"
                                        strokeWidth="10"
                                        strokeDasharray="125.6 251.2"
                                        strokeDashoffset={125.6 - (125.6 * MOCK_DASHBOARD_DATA.dailyGoal.progress) / 100}
                                        strokeLinecap="round"
                                        className="text-lime transition-all duration-1000 ease-out"
                                    />
                                </svg>
                                <div className="absolute bottom-0 left-0 right-0 flex flex-col items-center justify-end pb-1">
                                    <p className="text-lg sm:text-xl md:text-2xl font-bold text-white leading-none">{MOCK_DASHBOARD_DATA.dailyGoal.progress}%</p>
                                    <p className="text-[8px] sm:text-[10px] text-white/40 font-bold uppercase tracking-widest">Progress</p>
                                </div>
                            </div>
                        </div>

                        <div className="grid grid-cols-2 gap-1.5 sm:gap-2 flex-1">
                            <div className="bg-dark-bg rounded-md sm:rounded-lg md:rounded-xl p-2 sm:p-2.5 relative overflow-hidden min-h-[60px] sm:min-h-[70px]">
                                <h2 className="text-[10px] sm:text-xs md:text-sm font-semibold text-lime leading-tight relative z-20">Distractions</h2>
                                <p className="text-[9px] sm:text-[10px] font-semibold text-white/60 relative z-20">Blocked</p>
                                <div
                                    className="absolute bg-lime rounded-full flex items-center justify-center shadow-2xl transition-all duration-300 ease-out z-0"
                                    style={{
                                        // Ultra-Aggressive Height Reduction Scaling: 
                                        // Shrinks way faster when height decreases (h-scale ^ 2.5) than when width decreases (w-scale ^ 1.2)
                                        width: `${Math.max(30, Math.min(240, 240 * Math.min(
                                            Math.pow(windowWidth / 1280, 2),
                                            Math.pow(windowHeight / 800, 2.5)
                                        )))}px`,
                                        height: `${Math.max(30, Math.min(240, 240 * Math.min(
                                            Math.pow(windowWidth / 1280, 2),
                                            Math.pow(windowHeight / 800, 2.5)
                                        )))}px`,
                                        // Pushed further into the corner
                                        bottom: '-20%',
                                        right: '-20%',
                                    }}
                                >
                                    <span
                                        className="font-black text-synapse-dark tracking-tighter leading-none select-none transition-all duration-300"
                                        style={{
                                            // Proportional font reduction matching ultra-aggressive height-sensitive shrink
                                            fontSize: `${Math.max(12, Math.min(110, 110 * Math.min(
                                                Math.pow(windowWidth / 1280, 1.2),
                                                Math.pow(windowHeight / 800, 2.5)
                                            )))}px`,
                                            // Position adjustment for overflow
                                            transform: 'translate(-15%, -15%)'
                                        }}
                                    >
                                        {MOCK_DASHBOARD_DATA.distractions.blockedCount}
                                    </span>
                                </div>
                            </div>
                            <div className="bg-dark-bg rounded-md sm:rounded-lg md:rounded-xl p-2 sm:p-2.5 flex flex-col gap-1 overflow-hidden">
                                <h2 className="text-[10px] sm:text-xs md:text-sm font-semibold text-lime leading-tight">Top Distractions</h2>
                                <div className="flex flex-col gap-1 mt-auto">
                                    {MOCK_DASHBOARD_DATA.distractions.topDistractions.slice(0, 2).map((item, i) => (
                                        <div key={i} className="flex justify-between items-center bg-white/5 px-1.5 py-0.5 rounded">
                                            <span className="text-[8px] sm:text-[9px] text-white/80 truncate max-w-[60%]">{item.name}</span>
                                            <span className="text-[8px] sm:text-[9px] text-lime font-bold">{item.time}</span>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        </div>

                        <div className="bg-dark-bg rounded-lg sm:rounded-xl md:rounded-2xl p-2 sm:p-2.5 md:p-3 flex-shrink-0 min-h-[120px] overflow-hidden flex flex-col">
                            <h2 className="text-sm sm:text-base md:text-lg font-semibold text-lime tracking-tight leading-tight mb-1.5 sm:mb-2 flex-shrink-0">Task List</h2>
                            <div className="space-y-1 sm:space-y-1.5 overflow-y-auto pr-1 sm:pr-1.5 custom-scrollbar flex-1 max-h-[150px] lg:max-h-none">
                                {MOCK_DASHBOARD_DATA.tasks.map((task, i) => (
                                    <div key={i} className="flex items-center gap-2 sm:gap-2.5 md:gap-3 bg-dark-green rounded-lg sm:rounded-xl md:rounded-2xl p-2 sm:p-2.5 md:p-3 hover:bg-dark-green/80 transition-colors cursor-pointer group">
                                        <div className={`w-[16px] h-[16px] sm:w-[20px] sm:h-[20px] md:w-[24px] md:h-[24px] rounded-md flex-shrink-0 flex items-center justify-center transition-all ${task.completed ? 'bg-lime' : 'bg-dark-bg border border-lime/20 group-hover:border-lime/40'}`}>
                                            {task.completed && (
                                                <svg className="w-3 h-3 sm:w-4 sm:h-4 text-synapse-dark" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="3">
                                                    <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                                                </svg>
                                            )}
                                        </div>
                                        <span className={`text-[10px] sm:text-xs md:text-sm transition-all ${task.completed ? 'text-lime/50 line-through' : 'text-lime'}`}>
                                            {task.title}
                                        </span>
                                    </div>
                                ))}
                            </div>
                        </div>

                        {/* Spotify player - shown only on small screens below Task List */}
                        <div className="block lg:hidden bg-white/10 backdrop-blur-md rounded-lg sm:rounded-xl md:rounded-2xl p-2 sm:p-3 md:p-4 flex flex-col gap-1.5 sm:gap-2 md:gap-3 border border-white/5 flex-1 lg:flex-shrink-0 justify-center">
                            {isAuthenticated && track ? (
                                <div className="flex flex-col gap-2">
                                    <div className="text-center">
                                        <p className="text-white font-bold text-xs sm:text-sm truncate">{track.name}</p>
                                        <p className="text-white/60 text-[10px] sm:text-xs truncate">{track.artist}</p>
                                    </div>

                                    <div className="flex justify-center items-center gap-3 sm:gap-4">
                                        <button
                                            onClick={skipPrevious}
                                            className="p-1 sm:p-1.5 hover:bg-white/10 active:scale-95 rounded-full transition-all flex items-center justify-center h-6 w-6 sm:h-7 sm:w-7 text-white"
                                            title="Previous"
                                        >
                                            <SkipBack className="w-3 h-3 sm:w-3.5 sm:h-3.5" fill="currentColor" />
                                        </button>

                                        <button
                                            onClick={togglePlayback}
                                            className="w-8 h-8 sm:w-9 sm:h-9 bg-lime active:scale-90 rounded-full flex items-center justify-center hover:scale-105 transition-all shadow-lg shrink-0"
                                            title={track.is_playing ? 'Pause' : 'Play'}
                                        >
                                            {track.is_playing ? (
                                                <Pause className="w-4 h-4 sm:w-4.5 sm:h-4.5 fill-synapse-dark stroke-synapse-dark" />
                                            ) : (
                                                <Play className="w-4 h-4 sm:w-4.5 sm:h-4.5 fill-synapse-dark stroke-synapse-dark ml-0.5" />
                                            )}
                                        </button>

                                        <button
                                            onClick={skipNext}
                                            className="p-1 sm:p-1.5 hover:bg-white/10 active:scale-95 rounded-full transition-all flex items-center justify-center h-6 w-6 sm:h-7 sm:w-7 text-white"
                                            title="Next"
                                        >
                                            <SkipForward className="w-3 h-3 sm:w-3.5 sm:h-3.5" fill="currentColor" />
                                        </button>
                                    </div>
                                </div>
                            ) : (
                                <div className="flex flex-col items-center justify-center text-center gap-2">
                                    <div className="w-10 h-10 sm:w-12 sm:h-12 bg-lime/20 rounded-full flex items-center justify-center">
                                        <svg className="w-5 h-5 sm:w-6 sm:h-6 text-lime" viewBox="0 0 24 24" fill="currentColor">
                                            <path d="M12 0C5.373 0 0 5.373 0 12s5.373 12 12 12 12-5.373 12-12S18.627 0 12 0zm5.485 17.302c-.215.354-.675.466-1.03.25-2.857-1.745-6.453-2.14-10.687-1.173-.406.093-.815-.16-.908-.567-.093-.406.16-.815.567-.908 4.636-1.06 8.594-.61 11.808 1.353.354.215.466.675.25 1.03zm1.464-3.26c-.27.44-.847.58-1.287.31-3.27-2.01-8.254-2.59-12.12-1.415-.494.15-1.025-.13-1.175-.624-.15-.494.13-1.025.624-1.175 4.414-1.34 9.907-.695 13.65 1.616.44.27.58.847.31 1.287zm.126-3.41c-3.922-2.33-10.385-2.545-14.136-1.406-.6.182-1.24-.16-1.423-.762-.182-.6.16-1.24.762-1.423 4.314-1.31 11.448-1.055 15.952 1.62.54.32.716 1.025.397 1.566-.32.54-1.025.716-1.566.397z" />
                                        </svg>
                                    </div>
                                    <div>
                                        <h3 className="text-white font-bold text-xs sm:text-sm">Spotify</h3>
                                        <p className="text-white/60 text-[10px] sm:text-xs">Connect to see what's playing</p>
                                    </div>
                                    <button
                                        onClick={login}
                                        className="bg-lime text-synapse-dark px-3 py-1 sm:px-4 sm:py-1.5 rounded-full font-bold text-[10px] sm:text-xs hover:bg-lime/90 transition-colors"
                                    >
                                        Connect
                                    </button>
                                </div>
                            )}
                        </div>
                    </div>

                    {/* COLUMN 3: Streak Bar + Calendar */}
                    <div className="flex flex-col gap-2 sm:gap-3 md:gap-4 min-h-0">
                        {/* FIGMA STREAK BAR */}
                        <div className="flex items-center gap-1.5 sm:gap-2 md:gap-3 w-full flex-shrink-0">
                            <div className="flex-1 h-[28px] sm:h-[32px] md:h-[36px] px-2 sm:px-3 md:px-4 flex items-center justify-between bg-lime rounded-lg sm:rounded-xl">
                                <span className="text-[8px] sm:text-[9px] md:text-[10px] font-bold text-synapse-dark uppercase">Current Streak</span>
                                <span className="text-[10px] sm:text-xs md:text-sm font-bold text-synapse-dark">{MOCK_DASHBOARD_DATA.streak}</span>
                            </div>
                            <div className="w-[28px] h-[28px] sm:w-[32px] sm:h-[32px] md:w-[36px] md:h-[36px] flex items-center justify-center bg-lime rounded-full">
                                <svg className="w-3 h-3 sm:w-3.5 sm:h-3.5 md:w-4 md:h-4" viewBox="0 0 28 28" fill="none">
                                    <path d="M15.0973 21.5754C17.6715 21.0616 21 19.2179 21 14.4502C21 10.1115 17.8102 7.22242 15.5165 5.89491C15.0076 5.60033 14.4118 5.98773 14.4118 6.57386V8.07306C14.4118 9.25527 13.9125 11.4132 12.5253 12.3107C11.8171 12.769 11.0522 12.0831 10.9661 11.2465L10.8954 10.5596C10.8133 9.76097 9.99634 9.27617 9.35526 9.76308C8.20356 10.6378 7 12.1695 7 14.4502C7 20.2807 11.3556 21.7384 13.5333 21.7384C13.66 21.7384 13.7931 21.7346 13.9317 21.7267C12.8564 21.6352 11.1176 20.9709 11.1176 18.8228C11.1176 17.1426 12.3489 16.0059 13.2844 15.4533C13.5359 15.3047 13.8304 15.4977 13.8304 15.7889V16.272C13.8304 16.6417 13.9741 17.2197 14.3159 17.6153C14.7028 18.0629 15.2706 17.594 15.3164 17.0052C15.3309 16.8195 15.5185 16.7011 15.6801 16.7951C16.2082 17.1024 16.8824 17.7588 16.8824 18.8228C16.8824 20.502 15.9526 21.2745 15.0973 21.5754Z" className="fill-synapse-dark" />
                                </svg>
                            </div>
                        </div>

                        <div className="bg-dark-bg rounded-lg sm:rounded-xl md:rounded-2xl p-2 sm:p-3 md:p-4 flex flex-col flex-1 lg:flex-shrink-0 min-h-0">
                            <div className="flex justify-between items-center mb-2 sm:mb-3">
                                <h3 className="text-[10px] sm:text-xs md:text-sm font-bold text-lime uppercase">
                                    {format(currentMonth, 'MMMM yyyy')}
                                </h3>
                                <div className="flex gap-1 sm:gap-1.5">
                                    <button
                                        onClick={handlePrevMonth}
                                        className="p-0.5 sm:p-1 hover:bg-white/5 rounded transition-colors"
                                    >
                                        <svg className="w-2 h-2 sm:w-2.5 sm:h-2.5" viewBox="0 0 10 10" fill="none">
                                            <path d="M5.80983 0.463481C6.05104 0.211 6.45422 0.211 6.69543 0.463481C6.92158 0.700182 6.92158 1.07288 6.69543 1.30958L3.63022 4.51793L6.69543 7.72628C6.92158 7.96298 6.92158 8.33567 6.69543 8.57238C6.45422 8.82486 6.05104 8.82486 5.80983 8.57238L1.93626 4.51793L5.80983 0.463481Z" className="fill-white/40" />
                                        </svg>
                                    </button>
                                    <button
                                        onClick={handleNextMonth}
                                        className="p-0.5 sm:p-1 hover:bg-white/5 rounded transition-colors"
                                    >
                                        <svg className="w-2 h-2 sm:w-2.5 sm:h-2.5" viewBox="0 0 10 10" fill="none">
                                            <path d="M3.22603 0.463481C2.98481 0.211 2.58164 0.211 2.34042 0.463481C2.11428 0.700182 2.11428 1.07288 2.34042 1.30958L5.40564 4.51793L2.34042 7.72628C2.11428 7.96298 2.11428 8.33567 2.34042 8.57238C2.58164 8.82486 2.98481 8.82486 3.22603 8.57238L7.09959 4.51793L3.22603 0.463481Z" className="fill-lime" />
                                        </svg>
                                    </button>
                                </div>
                            </div>
                            <div className="grid grid-cols-7 mb-1 text-center shrink-0">
                                {['Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa', 'Su'].map(d => (
                                    <div key={d} className="text-[8px] sm:text-[9px] md:text-[10px] font-bold text-lime/40">{d}</div>
                                ))}
                            </div>
                            <div className="grid grid-cols-7 gap-y-1 text-center flex-1 min-h-0 auto-rows-fr items-center">
                                {calendarDays.map(day => {
                                    const isCurrentMonth = isSameMonth(day, monthStart);
                                    const isTodayDate = isToday(day);
                                    return (
                                        <div
                                            key={day.toISOString()}
                                            className={`text-[8px] sm:text-[9px] md:text-[10px] py-0.5 sm:py-1 font-semibold transition-all ${isTodayDate
                                                ? 'bg-lime text-synapse-dark rounded-md'
                                                : !isCurrentMonth
                                                    ? 'text-white/20'
                                                    : 'text-white'
                                                }`}
                                        >
                                            {format(day, 'd')}
                                        </div>
                                    );
                                })}
                            </div>
                        </div>

                        {/* Spotify player - shown only on large screens below Calendar */}
                        <div className="hidden lg:grid grid-rows-[1fr_auto] bg-white/10 backdrop-blur-md rounded-lg sm:rounded-xl md:rounded-2xl p-2 sm:p-3 md:p-4 lg:p-3 gap-1.5 sm:gap-2 md:gap-3 lg:gap-2 border border-white/5 flex-1 min-h-0 overflow-hidden">
                            {isAuthenticated ? (
                                isIdle && user ? (
                                    // IDLE PROFILE VIEW
                                    <div className="flex flex-col items-center justify-center p-4 text-center h-full relative z-20 animate-in fade-in duration-500">
                                        <div className="relative mb-3 sm:mb-4 group">
                                            <img
                                                src={user.images?.[0]?.url || "https://cdn.builder.io/api/v1/image/assets/TEMP/3b1994b2a7713d76ffb8d0e4e3f6f86d662d4483"}
                                                alt={user.display_name}
                                                className="relative w-20 h-20 sm:w-24 sm:h-24 md:w-28 md:h-28 rounded-full object-cover border-2 border-white/10 shadow-lg grayscale hover:grayscale-0 transition-all duration-500"
                                            />
                                        </div>

                                        <h3 className="text-white font-bold text-lg sm:text-xl md:text-2xl mb-1 tracking-tight">{user.display_name}</h3>

                                        <div className="flex items-center gap-1.5 mb-4 sm:mb-6 opacity-60">
                                            <p className="text-white text-[10px] sm:text-xs font-medium uppercase tracking-wider">{user.followers?.total || 0} FOLLOWERS</p>
                                        </div>

                                        <button
                                            onClick={togglePlayback}
                                            className="text-white/80 hover:text-white border border-white/20 hover:border-white/40 hover:bg-white/5 px-6 py-1.5 sm:px-8 sm:py-2 rounded-full font-medium text-xs sm:text-sm active:scale-95 transition-all flex items-center gap-2"
                                        >
                                            <Play className="w-3.5 h-3.5 fill-current" />
                                            Resume
                                        </button>
                                    </div>
                                ) : track ? (
                                    // ACTIVE PLAYER VIEW
                                    <>
                                        <div className="min-h-0 relative w-full h-full overflow-hidden flex items-center justify-center p-2">
                                            <img
                                                src={track.albumArt || "https://cdn.builder.io/api/v1/image/assets/TEMP/3b1994b2a7713d76ffb8d0e4e3f6f86d662d4483"}
                                                className="h-full w-auto aspect-square object-cover rounded-md sm:rounded-lg md:rounded-xl border-2 border-white/20 shadow-lg"
                                                alt="Song Art"
                                            />
                                            <button
                                                onClick={logout}
                                                className="absolute top-3 right-3 bg-black/50 hover:bg-black/80 text-white/70 hover:text-white px-1.5 py-0.5 sm:px-2 sm:py-1 rounded text-[8px] sm:text-[9px] uppercase font-bold opacity-0 group-hover:opacity-100 transition-opacity z-10"
                                                title="Disconnect"
                                            >
                                                Disconnect
                                            </button>
                                        </div>
                                        <div className="text-center flex flex-col gap-1 sm:gap-1.5 md:gap-2 relative z-20">
                                            <div className="mb-0.5 sm:mb-1">
                                                <p className="text-white font-bold text-xs sm:text-sm md:text-base truncate">{track.name}</p>
                                                <p className="text-white/60 text-[10px] sm:text-xs truncate">{track.artist}</p>
                                            </div>

                                            {/* Progress Bar */}
                                            <div className="w-full group">
                                                <input
                                                    type="range"
                                                    min="0"
                                                    max={track.duration_ms}
                                                    value={isDragging ? dragValue : progress}
                                                    onInput={(e: React.FormEvent<HTMLInputElement>) => {
                                                        setIsDragging(true);
                                                        setDragValue(parseInt(e.currentTarget.value));
                                                    }}
                                                    onChange={(e) => {
                                                        const val = parseInt(e.target.value);
                                                        seek(val);
                                                        setIsDragging(false);
                                                    }}
                                                    className="w-full h-0.5 sm:h-1 bg-white/20 rounded-full appearance-none cursor-pointer accent-white hover:accent-lime transition-all"
                                                />
                                                <div className="flex justify-between mt-0.5 sm:mt-1 px-0.5">
                                                    <span className="text-white/40 text-[8px] sm:text-[9px] md:text-[10px] font-medium tabular-nums">
                                                        {formatTime(isDragging ? dragValue : progress)}
                                                    </span>
                                                    <span className="text-white/40 text-[8px] sm:text-[9px] md:text-[10px] font-medium tabular-nums">
                                                        {formatTime(track.duration_ms)}
                                                    </span>
                                                </div>
                                            </div>

                                            <div className="flex justify-center items-center gap-3 sm:gap-4 md:gap-6 min-h-[32px] sm:min-h-[40px]">
                                                <button
                                                    onClick={skipPrevious}
                                                    className="p-1 sm:p-1.5 md:p-2 hover:bg-white/10 active:scale-95 rounded-full transition-all flex items-center justify-center h-6 w-6 sm:h-8 sm:w-8 text-white"
                                                    title="Previous"
                                                >
                                                    <SkipBack className="w-3 h-3 sm:w-4 sm:h-4" fill="currentColor" />
                                                </button>

                                                <button
                                                    onClick={togglePlayback}
                                                    className="w-7 h-7 sm:w-9 sm:h-9 md:w-11 md:h-11 bg-lime active:scale-90 rounded-full flex items-center justify-center hover:scale-105 transition-all shadow-lg shrink-0"
                                                    title={track.is_playing ? 'Pause' : 'Play'}
                                                >
                                                    {track.is_playing ? (
                                                        <Pause className="w-3.5 h-3.5 sm:w-4 sm:h-4 md:w-5 md:h-5 fill-synapse-dark stroke-synapse-dark" />
                                                    ) : (
                                                        <Play className="w-3.5 h-3.5 sm:w-4 sm:h-4 md:w-5 md:h-5 fill-synapse-dark stroke-synapse-dark ml-0.5" />
                                                    )}
                                                </button>

                                                <button
                                                    onClick={skipNext}
                                                    className="p-1 sm:p-1.5 md:p-2 hover:bg-white/10 active:scale-95 rounded-full transition-all flex items-center justify-center h-6 w-6 sm:h-8 sm:w-8 text-white"
                                                    title="Next"
                                                >
                                                    <SkipForward className="w-3 h-3 sm:w-4 sm:h-4" fill="currentColor" />
                                                </button>
                                            </div>
                                        </div>
                                    </>
                                ) : (
                                    // NO TRACK -> CONNECT VIEW (Fallback/Modified)
                                    <div className="flex-1 flex flex-col items-center justify-center text-center gap-2 sm:gap-3">
                                        <div className="w-10 h-10 sm:w-12 sm:h-12 md:w-14 md:h-14 bg-lime/20 rounded-full flex items-center justify-center">
                                            <svg className="w-5 h-5 sm:w-6 sm:h-6 md:w-7 md:h-7 text-lime" viewBox="0 0 24 24" fill="currentColor">
                                                <path d="M12 0C5.373 0 0 5.373 0 12s5.373 12 12 12 12-5.373 12-12S18.627 0 12 0zm5.485 17.302c-.215.354-.675.466-1.03.25-2.857-1.745-6.453-2.14-10.687-1.173-.406.093-.815-.16-.908-.567-.093-.406.16-.815.567-.908 4.636-1.06 8.594-.61 11.808 1.353.354.215.466.675.25 1.03zm1.464-3.26c-.27.44-.847.58-1.287.31-3.27-2.01-8.254-2.59-12.12-1.415-.494.15-1.025-.13-1.175-.624-.15-.494.13-1.025.624-1.175 4.414-1.34 9.907-.695 13.65 1.616.44.27.58.847.31 1.287zm.126-3.41c-3.922-2.33-10.385-2.545-14.136-1.406-.6.182-1.24-.16-1.423-.762-.182-.6.16-1.24.762-1.423 4.314-1.31 11.448-1.055 15.952 1.62.54.32.716 1.025.397 1.566-.32.54-1.025.716-1.566.397z" />
                                            </svg>
                                        </div>
                                        <div>
                                            <h3 className="text-white font-bold text-xs sm:text-sm md:text-base">Spotify Connected</h3>
                                            <p className="text-white/60 text-[10px] sm:text-xs">Waiting for playback...</p>
                                        </div>
                                        <button
                                            onClick={togglePlayback}
                                            className="bg-lime text-synapse-dark px-3 py-1 sm:px-4 sm:py-1.5 md:px-6 md:py-2 rounded-full font-bold text-[10px] sm:text-xs md:text-sm hover:bg-lime/90 transition-colors"
                                        >
                                            Resume
                                        </button>
                                    </div>
                                )
                            ) : (
                                // NOT AUTHENTICATED -> CONNECT BUTTON
                                <div className="flex-1 flex flex-col items-center justify-center text-center gap-2 sm:gap-3">
                                    <div className="w-10 h-10 sm:w-12 sm:h-12 md:w-14 md:h-14 bg-lime/20 rounded-full flex items-center justify-center">
                                        <svg className="w-5 h-5 sm:w-6 sm:h-6 md:w-7 md:h-7 text-lime" viewBox="0 0 24 24" fill="currentColor">
                                            <path d="M12 0C5.373 0 0 5.373 0 12s5.373 12 12 12 12-5.373 12-12S18.627 0 12 0zm5.485 17.302c-.215.354-.675.466-1.03.25-2.857-1.745-6.453-2.14-10.687-1.173-.406.093-.815-.16-.908-.567-.093-.406.16-.815.567-.908 4.636-1.06 8.594-.61 11.808 1.353.354.215.466.675.25 1.03zm1.464-3.26c-.27.44-.847.58-1.287.31-3.27-2.01-8.254-2.59-12.12-1.415-.494.15-1.025-.13-1.175-.624-.15-.494.13-1.025.624-1.175 4.414-1.34 9.907-.695 13.65 1.616.44.27.58.847.31 1.287zm.126-3.41c-3.922-2.33-10.385-2.545-14.136-1.406-.6.182-1.24-.16-1.423-.762-.182-.6.16-1.24.762-1.423 4.314-1.31 11.448-1.055 15.952 1.62.54.32.716 1.025.397 1.566-.32.54-1.025.716-1.566.397z" />
                                        </svg>
                                    </div>
                                    <div>
                                        <h3 className="text-white font-bold text-xs sm:text-sm md:text-base">Spotify</h3>
                                        <p className="text-white/60 text-[10px] sm:text-xs">Connect to see what's playing</p>
                                    </div>
                                    <button
                                        onClick={login}
                                        className="bg-lime text-synapse-dark px-3 py-1 sm:px-4 sm:py-1.5 md:px-6 md:py-2 rounded-full font-bold text-[10px] sm:text-xs md:text-sm hover:bg-lime/90 transition-colors"
                                    >
                                        Connect
                                    </button>
                                </div>
                            )}
                        </div>
                    </div>

                </div>
            </div>
            <style dangerouslySetInnerHTML={{
                __html: `
        .custom-scrollbar::-webkit-scrollbar { width: 4px; }
        .custom-scrollbar::-webkit-scrollbar-track { background: rgba(255, 255, 255, 0.05); border-radius: 10px; }
        .custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(196, 217, 70, 0.2); border-radius: 10px; }
        .custom-scrollbar::-webkit-scrollbar-thumb:hover { background: rgba(196, 217, 70, 0.4); }
      `}} />
        </div>
    );
}
