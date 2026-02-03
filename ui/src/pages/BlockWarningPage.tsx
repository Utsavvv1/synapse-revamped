
import { useEffect, useState, useRef } from "react"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { LogicalSize } from "@tauri-apps/api/dpi"
import AppBlockModal from "../components/AppBlockModal"

export default function BlockWarningPage() {
    // Start visible immediately to avoid flicker
    const [isVisible, setIsVisible] = useState(true)
    const containerRef = useRef<HTMLDivElement>(null)

    useEffect(() => {
        // Force transparent background for the block window
        document.documentElement.style.backgroundColor = 'transparent'
        document.body.style.backgroundColor = 'transparent' // Important for Tauri transparency

        // Show window only after styles are applied to prevent white flash
        const showWindow = async () => {
            const win = getCurrentWindow();
            await win.show();
            await win.setFocus();
        };
        showWindow();

        return () => {
            // Cleanup if needed (though window closes usually)
            document.documentElement.style.backgroundColor = ''
            document.body.style.backgroundColor = ''
        }
    }, [])

    const handleClose = async () => {
        setIsVisible(false)
        setTimeout(async () => {
            await getCurrentWindow().close()
        }, 300) // Match animation duration
    }

    const handleUseFor5Mins = async () => {
        setIsVisible(false)
        console.log("Using for 5 mins from separate window")
        setTimeout(async () => {
            await getCurrentWindow().close()
        }, 300)
    }

    console.log("BlockWarningPage rendering, isVisible:", isVisible);
    return (
        <div className="h-screen w-screen bg-transparent flex items-center justify-center overflow-hidden">
            <AppBlockModal
                isVisible={isVisible}
                onClose={handleClose}
                onUseFor5Mins={handleUseFor5Mins}
                isStandalone={true}
            />
        </div>
    )
}
