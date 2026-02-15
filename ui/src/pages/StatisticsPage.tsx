import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
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

export default function StatisticsPage() {
    const navigate = useNavigate();
    const { track, progress, login, logout, isAuthenticated, togglePlayback, skipNext, skipPrevious, seek } = useSpotify();
    const [currentTime, setCurrentTime] = useState(new Date().toLocaleTimeString("en-US", {
        hour12: false,
        hour: "2-digit",
        minute: "2-digit",
    }));

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

    return (
        <div
            className="h-screen w-screen p-4 lg:p-8 bg-black bg-cover bg-center bg-no-repeat overflow-hidden flex flex-col font-sans"
            style={{
                backgroundImage: `linear-gradient(rgba(0,0,0,0.1), rgba(0,0,0,0.1)), url('https://api.builder.io/api/v1/image/assets/TEMP/3b1994b2a7713d76ffb8d0e4e3f6f86d662d4483?width=3504')`,
                backgroundAttachment: 'fixed'
            }}
        >
            {/* Drag region wrapper matching main page */}
            <div data-tauri-drag-region className="mb-4">
                <SynapseHeader currentTime={currentTime} />
            </div>
            <div className="max-w-[1800px] w-full mx-auto flex-1 flex flex-col min-h-0">
                <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 lg:gap-6 flex-1 min-h-0">

                    {/* COLUMN 1: Dashboard Title + Weekly Summary + Stats Row */}
                    <div className="flex flex-col gap-4 lg:gap-6 min-h-0">
                        <div className="flex items-center gap-4 lg:gap-6">
                            <button
                                onClick={() => navigate('/')}
                                className="p-2 lg:p-3 bg-[#061615] hover:bg-[#364721] text-lime rounded-full transition-all flex items-center justify-center shadow-lg active:scale-95 group"
                                title="Back to Dashboard"
                            >
                                <ArrowLeft className="w-6 h-6 lg:w-8 lg:h-8 group-hover:-translate-x-1 transition-transform" />
                            </button>
                            <h1 className="text-4xl lg:text-6xl xl:text-7xl font-semibold text-[#F3F3F3] tracking-tighter leading-none flex-shrink-0">
                                Dashboard
                            </h1>
                        </div>

                        <div className="bg-[#061615] rounded-[30px] p-6 flex flex-col justify-between flex-1 min-h-0">
                            <h2 className="text-2xl lg:text-[36px] font-semibold text-lime tracking-tight leading-tight">
                                Weekly Summary
                            </h2>
                            <p className="text-xl lg:text-2xl text-lime/80 mt-auto">34h 20m</p>
                        </div>

                        {/* Row of Sessions, Average, Longest */}
                        <div className="grid grid-cols-3 gap-3 lg:gap-4 flex-shrink-0">
                            <div className="bg-lime rounded-[28px] lg:rounded-[34px] p-3 lg:p-5 flex flex-col justify-between aspect-square lg:aspect-auto lg:min-h-[140px]">
                                <h3 className="text-sm lg:text-[24px] font-semibold text-[#364721] tracking-tight">Sessions</h3>
                                <p className="text-2xl lg:text-[40px] font-semibold text-[#061615] tracking-tight leading-none">35</p>
                            </div>

                            <div className="bg-lime rounded-[28px] lg:rounded-[34px] p-3 lg:p-5 flex flex-col gap-2 aspect-square lg:aspect-auto lg:min-h-[140px]">
                                <h3 className="text-sm lg:text-[24px] font-semibold text-[#364721] tracking-tight">Average</h3>
                                <div className="bg-[#95A83A] rounded-[14px] py-1 lg:py-2 flex items-center justify-center mt-auto">
                                    <span className="text-xs lg:text-[24px] font-semibold text-[#061615] tracking-tight leading-none">2h 30m</span>
                                </div>
                            </div>

                            <div className="bg-lime rounded-[28px] lg:rounded-[34px] p-3 lg:p-5 flex flex-col gap-2 aspect-square lg:aspect-auto lg:min-h-[140px]">
                                <h3 className="text-sm lg:text-[24px] font-semibold text-[#364721] tracking-tight">Longest</h3>
                                <div className="bg-[#95A83A] rounded-[14px] py-1 lg:py-2 flex items-center justify-center mt-auto">
                                    <span className="text-xs lg:text-[24px] font-semibold text-[#061615] tracking-tight leading-none">2h 30m</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* COLUMN 2: Daily Goal + Distractions + Task List */}
                    <div className="flex flex-col gap-4 lg:gap-6 min-h-0">
                        <div className="bg-[#061615] rounded-[30px] p-4 lg:p-5 flex flex-col justify-between flex-shrink-0">
                            <div>
                                <h2 className="text-xl lg:text-[28px] font-semibold text-lime tracking-tight leading-tight">Daily Goal</h2>
                                <p className="text-base lg:text-lg text-lime/80">2h 45m</p>
                            </div>
                            <div className="flex justify-center mt-2">
                                <svg className="w-24 h-24 lg:w-32 lg:h-32" viewBox="0 0 218 218" fill="none">
                                    <path d="M17 107.181C12.5817 107.181 8.96509 103.593 9.33621 99.1901C10.185 89.1203 12.6347 79.2236 16.6121 69.859C21.6375 58.0265 29.0035 47.2753 38.2893 38.2191C47.5752 29.163 58.5991 21.9792 70.7317 17.0781C82.8642 12.1769 95.8678 9.6543 109 9.6543C122.132 9.6543 135.136 12.1769 147.268 17.0781C159.401 21.9792 170.425 29.163 179.711 38.2192C188.997 47.2753 196.362 58.0266 201.388 69.859C205.365 79.2236 207.815 89.1203 208.664 99.1901C209.035 103.593 205.418 107.181 201 107.181C196.582 107.181 193.041 103.59 192.6 99.1941C191.794 91.1753 189.779 83.3013 186.606 75.8305C182.384 65.8912 176.197 56.8602 168.397 49.253C160.597 41.6458 151.337 35.6115 141.145 31.4945C130.954 27.3775 120.031 25.2585 109 25.2585C97.969 25.2585 87.046 27.3775 76.8546 31.4945C66.6633 35.6115 57.4032 41.6458 49.603 49.253C41.8029 56.8602 35.6155 65.8912 31.3941 75.8305C28.2211 83.3013 26.2056 91.1753 25.4001 99.1941C24.9586 103.59 21.4183 107.181 17 107.181Z" fill="#6D6D6D" />
                                    <path d="M17.0008 107.181C12.5821 107.181 8.96506 103.592 9.33624 99.1892C10.8334 81.4302 17.2984 64.3543 28.1001 49.8562C40.5141 33.194 58.0186 20.792 78.1048 14.4276C98.1911 8.0632 119.828 8.0632 139.914 14.4276C157.378 19.961 172.89 30.0583 184.801 43.5514C187.749 46.8905 186.926 51.9768 183.291 54.5518C179.778 57.0411 174.947 56.2417 172.062 53.0456C162.139 42.0552 149.341 33.8218 134.969 29.2681C118.097 23.922 99.922 23.922 83.0496 29.2681C66.1771 34.6142 51.4733 45.0319 41.0456 59.0281C32.2309 70.8593 26.8542 84.7341 25.4017 99.1932C24.96 103.59 21.4195 107.181 17.0008 107.181Z" fill="#C4D946" />
                                </svg>
                            </div>
                        </div>

                        <div className="grid grid-cols-2 gap-3 lg:gap-4 flex-shrink-0">
                            <div className="bg-[#061615] rounded-[24px] lg:rounded-[30px] p-4 lg:p-6 relative overflow-hidden h-[120px] lg:h-[180px]">
                                <h2 className="text-base lg:text-xl font-semibold text-lime leading-tight">Distractions</h2>
                                <p className="text-xs lg:text-[16px] font-semibold text-[#C4C4C4]">Blocked</p>
                                <div className="absolute -bottom-6 -right-6 w-24 h-24 lg:w-32 lg:h-32 bg-lime rounded-full flex items-center justify-center">
                                    <span className="text-2xl lg:text-[48px] font-semibold text-black absolute left-5 top-5 lg:left-8 lg:top-8">18</span>
                                </div>
                            </div>
                            <div className="bg-[#061615] rounded-[24px] lg:rounded-[30px] p-4 lg:p-6 h-[120px] lg:h-[180px] flex flex-col justify-between">
                                <h2 className="text-base lg:text-xl font-semibold text-lime leading-tight">Top Distractions</h2>
                                <p className="text-xs lg:text-[16px] font-semibold text-[#C4C4C4]">This Week</p>
                            </div>
                        </div>

                        <div className="bg-[#061615] rounded-[30px] p-6 lg:p-8 flex-1 min-h-0 overflow-hidden flex flex-col">
                            <h2 className="text-2xl lg:text-[36px] font-semibold text-lime tracking-tight leading-tight mb-4">Task List</h2>
                            <div className="space-y-2 lg:space-y-3 overflow-y-auto pr-2 custom-scrollbar">
                                {[1, 2, 3, 4].map((_, i) => (
                                    <div key={i} className="flex items-center gap-3 bg-[#364721] rounded-[18px] lg:rounded-[24px] p-3 lg:p-4">
                                        <div className="w-[24px] h-[24px] lg:w-[30px] lg:h-[30px] rounded-lg bg-[#061615] flex-shrink-0"></div>
                                        <span className="text-sm lg:text-lg text-lime">Finish 25 graph questions</span>
                                    </div>
                                ))}
                            </div>
                        </div>
                    </div>

                    {/* COLUMN 3: Streak Bar + Calendar + Song Player */}
                    <div className="flex flex-col gap-4 lg:gap-6 min-h-0">
                        {/* FIGMA STREAK BAR */}
                        <div className="flex items-center gap-3 w-full flex-shrink-0">
                            <div className="flex-1 h-[40px] px-4 lg:px-6 flex items-center justify-between bg-lime rounded-[16px]">
                                <span className="text-[10px] font-bold text-black uppercase">Current Streak</span>
                                <span className="text-[14px] font-bold text-black">6</span>
                            </div>
                            <div className="w-[40px] h-[40px] flex items-center justify-center bg-lime rounded-full">
                                <svg width="22" height="22" viewBox="0 0 28 28" fill="none">
                                    <path d="M15.0973 21.5754C17.6715 21.0616 21 19.2179 21 14.4502C21 10.1115 17.8102 7.22242 15.5165 5.89491C15.0076 5.60033 14.4118 5.98773 14.4118 6.57386V8.07306C14.4118 9.25527 13.9125 11.4132 12.5253 12.3107C11.8171 12.769 11.0522 12.0831 10.9661 11.2465L10.8954 10.5596C10.8133 9.76097 9.99634 9.27617 9.35526 9.76308C8.20356 10.6378 7 12.1695 7 14.4502C7 20.2807 11.3556 21.7384 13.5333 21.7384C13.66 21.7384 13.7931 21.7346 13.9317 21.7267C12.8564 21.6352 11.1176 20.9709 11.1176 18.8228C11.1176 17.1426 12.3489 16.0059 13.2844 15.4533C13.5359 15.3047 13.8304 15.4977 13.8304 15.7889V16.272C13.8304 16.6417 13.9741 17.2197 14.3159 17.6153C14.7028 18.0629 15.2706 17.594 15.3164 17.0052C15.3309 16.8195 15.5185 16.7011 15.6801 16.7951C16.2082 17.1024 16.8824 17.7588 16.8824 18.8228C16.8824 20.502 15.9526 21.2745 15.0973 21.5754Z" fill="#061615" />
                                </svg>
                            </div>
                        </div>

                        <div className="bg-[#061615] rounded-[19px] p-5 lg:p-6 flex flex-col flex-shrink-0">
                            <div className="flex justify-between items-center mb-4 lg:mb-6">
                                <h3 className="text-[13px] lg:text-[15px] font-bold text-lime uppercase">May 2023</h3>
                                <div className="flex gap-2">
                                    <button className="p-1"><svg width="8" height="8" viewBox="0 0 10 10" fill="none"><path d="M5.80983 0.463481C6.05104 0.211 6.45422 0.211 6.69543 0.463481C6.92158 0.700182 6.92158 1.07288 6.69543 1.30958L3.63022 4.51793L6.69543 7.72628C6.92158 7.96298 6.92158 8.33567 6.69543 8.57238C6.45422 8.82486 6.05104 8.82486 5.80983 8.57238L1.93626 4.51793L5.80983 0.463481Z" fill="#AFAFAF" /></svg></button>
                                    <button className="p-1"><svg width="8" height="8" viewBox="0 0 10 10" fill="none"><path d="M3.22603 0.463481C2.98481 0.211 2.58164 0.211 2.34042 0.463481C2.11428 0.700182 2.11428 1.07288 2.34042 1.30958L5.40564 4.51793L2.34042 7.72628C2.11428 7.96298 2.11428 8.33567 2.34042 8.57238C2.58164 8.82486 2.98481 8.82486 3.22603 8.57238L7.09959 4.51793L3.22603 0.463481Z" fill="#C4D946" /></svg></button>
                                </div>
                            </div>
                            <div className="grid grid-cols-7 gap-y-2 lg:gap-y-3 text-center">
                                {['Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa', 'Su'].map(d => (
                                    <div key={d} className="text-[9px] lg:text-[10px] font-bold text-lime/40">{d}</div>
                                ))}
                                {Array.from({ length: 31 }, (_, i) => i + 1).map(day => (
                                    <div key={day} className={`text-[10px] lg:text-xs py-1 font-semibold ${day === 18 ? 'bg-lime text-black rounded-md' : 'text-white/60'}`}>
                                        {day}
                                    </div>
                                ))}
                            </div>
                        </div>

                        <div className="bg-white/10 backdrop-blur-md rounded-[24px] lg:rounded-[30px] p-4 lg:p-6 flex flex-col gap-3 lg:gap-4 border border-white/5 flex-1 min-h-0">
                            {isAuthenticated && track ? (
                                <>
                                    <div className="w-full rounded-xl lg:rounded-2xl overflow-hidden max-h-[160px] lg:max-h-none mx-auto relative group">
                                        <img
                                            src={track.albumArt || "https://cdn.builder.io/api/v1/image/assets/TEMP/3b1994b2a7713d76ffb8d0e4e3f6f86d662d4483"}
                                            className="w-full h-full object-cover opacity-90"
                                            alt="Song Art"
                                        />
                                        <button
                                            onClick={logout}
                                            className="absolute top-4 right-4 bg-black/50 hover:bg-black/80 text-white/70 hover:text-white px-2 py-1 rounded-md text-[10px] uppercase font-bold opacity-0 group-hover:opacity-100 transition-opacity"
                                        >
                                            Disconnect
                                        </button>
                                    </div>
                                    <div className="text-center mt-auto flex flex-col gap-2">
                                        <div className="mb-1">
                                            <p className="text-white font-bold text-lg lg:text-xl truncate">{track.name}</p>
                                            <p className="text-white/60 text-sm lg:text-base truncate">{track.artist}</p>
                                        </div>

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
                                                className="w-full h-1 bg-white/20 rounded-full appearance-none cursor-pointer accent-white hover:accent-lime transition-all"
                                            />
                                            <div className="flex justify-between mt-1 px-0.5">
                                                <span className="text-white/40 text-[10px] font-medium tabular-nums">
                                                    {formatTime(isDragging ? dragValue : progress)}
                                                </span>
                                                <span className="text-white/40 text-[10px] font-medium tabular-nums">
                                                    {formatTime(track.duration_ms)}
                                                </span>
                                            </div>
                                        </div>

                                        <div className="flex justify-center items-center gap-6 lg:gap-8 min-h-[48px]">
                                            <button
                                                onClick={skipPrevious}
                                                className="p-2 hover:bg-white/10 active:scale-95 rounded-full transition-all flex items-center justify-center h-10 w-10 text-white"
                                                title="Previous"
                                            >
                                                <SkipBack size={20} fill="currentColor" />
                                            </button>

                                            <button
                                                onClick={togglePlayback}
                                                className="w-12 h-12 bg-white active:scale-90 rounded-full flex items-center justify-center hover:scale-105 transition-all shadow-lg shrink-0"
                                                title={track.is_playing ? 'Pause' : 'Play'}
                                            >
                                                {track.is_playing ? (
                                                    <Pause size={24} fill="black" stroke="black" />
                                                ) : (
                                                    <Play size={24} fill="black" stroke="black" className="ml-1" />
                                                )}
                                            </button>

                                            <button
                                                onClick={skipNext}
                                                className="p-2 hover:bg-white/10 active:scale-95 rounded-full transition-all flex items-center justify-center h-10 w-10 text-white"
                                                title="Next"
                                            >
                                                <SkipForward size={20} fill="currentColor" />
                                            </button>
                                        </div>
                                    </div>
                                </>
                            ) : (
                                <div className="flex-1 flex flex-col items-center justify-center text-center gap-4">
                                    <div className="w-16 h-16 bg-lime/20 rounded-full flex items-center justify-center">
                                        <svg width="32" height="32" viewBox="0 0 24 24" fill="currentColor" className="text-lime">
                                            <path d="M12 0C5.373 0 0 5.373 0 12s5.373 12 12 12 12-5.373 12-12S18.627 0 12 0zm5.485 17.302c-.215.354-.675.466-1.03.25-2.857-1.745-6.453-2.14-10.687-1.173-.406.093-.815-.16-.908-.567-.093-.406.16-.815.567-.908 4.636-1.06 8.594-.61 11.808 1.353.354.215.466.675.25 1.03zm1.464-3.26c-.27.44-.847.58-1.287.31-3.27-2.01-8.254-2.59-12.12-1.415-.494.15-1.025-.13-1.175-.624-.15-.494.13-1.025.624-1.175 4.414-1.34 9.907-.695 13.65 1.616.44.27.58.847.31 1.287zm.126-3.41c-3.922-2.33-10.385-2.545-14.136-1.406-.6.182-1.24-.16-1.423-.762-.182-.6.16-1.24.762-1.423 4.314-1.31 11.448-1.055 15.952 1.62.54.32.716 1.025.397 1.566-.32.54-1.025.716-1.566.397z" />
                                        </svg>
                                    </div>
                                    <div>
                                        <h3 className="text-white font-bold text-lg lg:text-xl">Spotify</h3>
                                        <p className="text-white/60 text-sm">Connect to see what's playing</p>
                                    </div>
                                    <button
                                        onClick={login}
                                        className="bg-lime text-[#061615] px-6 py-2 rounded-full font-bold text-sm lg:text-base hover:bg-lime-dark transition-colors"
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
