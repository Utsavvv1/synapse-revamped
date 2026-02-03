
import { useEffect, useState } from "react"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import AppBlockModal from "../components/AppBlockModal"

export default function BlockWarningPage() {
    // Start visible immediately to avoid flicker
    const [isVisible, setIsVisible] = useState(true)
    const [appName, setAppName] = useState<string | null>(null)
    // Removed unused containerRef

    useEffect(() => {
        // Force transparent background for the block window
        document.documentElement.style.backgroundColor = 'transparent'
        document.body.style.backgroundColor = 'transparent' // Important for Tauri transparency

        // Parse initial URL params
        const params = new URLSearchParams(window.location.search);
        const app = params.get('app');
        if (app) {
            console.log("BlockWarningPage: Initial app name from URL:", app);
            setAppName(app);
        }

        // Listen for updates if window is reused
        const unlistenPromise = listen('update-block-info', (event) => {
            console.log("BlockWarningPage: Received update-block-info:", event.payload);
            setAppName(event.payload as string);
            setIsVisible(true); // Ensure visible if we are updated
        });

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
            unlistenPromise.then(unlisten => unlisten());
        }
    }, [])

    const handleClose = async () => {
        setIsVisible(false)
        if (appName) {
            console.log("Killing app:", appName);
            try {
                await invoke('kill_app_cmd', { appName });
            } catch (e) {
                console.error("Failed to kill app:", e);
            }
        }
        setTimeout(async () => {
            await getCurrentWindow().close()
        }, 300) // Match animation duration
    }

    const handleUseFor5Mins = async () => {
        setIsVisible(false)
        console.log("Using for 5 mins from separate window")
        if (appName) {
            console.log("Snoozing app:", appName);
            try {
                await invoke('snooze_app_cmd', { appName, durationSecs: 300 });
            } catch (e) {
                console.error("Failed to snooze app:", e);
            }
        }
        setTimeout(async () => {
            await getCurrentWindow().close()
        }, 300)
    }

    console.log("BlockWarningPage rendering, isVisible:", isVisible, "appName:", appName);
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
