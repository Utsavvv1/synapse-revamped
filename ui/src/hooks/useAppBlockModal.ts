import { useCallback, useEffect } from 'react'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { listen } from '@tauri-apps/api/event'
import { LogicalSize } from '@tauri-apps/api/dpi'

interface UseAppBlockModalReturn {
  showModal: () => void
  hideModal: () => void
}

export function useAppBlockModal(): UseAppBlockModalReturn {
  const showModal = useCallback(async (appName?: string) => {
    // Check if window already exists
    const label = 'block-warning-modal'
    const existing = await WebviewWindow.getByLabel(label)

    // Construct URL with app param if provided
    const url = appName ? `/block-warning?app=${encodeURIComponent(appName)}` : '/block-warning';

    if (existing) {
      if (appName) {
        // If reusing window, update the URL/search params so the page knows which app
        // But webview.window.location.href might not update if we don't navigate.
        // We can use emit to send update to existing window, or simple re-navigate.
        // Creating a second window with same label fails, so we must reuse.
        // Evaluating JS might be easiest to update the state if we don't want full reload.
        // BUT given simpler approach: just navigate.
        // But navigate might flash.
        // Alternative: emit event 'update-block-info' to the window.
        await existing.emit('update-block-info', appName);
      }

      // Ensure size is correct even if recycled
      await existing.hide() // Hide first to reset any state/flash
      await existing.setSize(new LogicalSize(360, 420))
      await existing.show()
      await existing.setFocus()
      return
    }

    const webview = new WebviewWindow(label, {
      url: url,
      title: 'Action Blocked',
      alwaysOnTop: true,
      decorations: false,
      width: 360,     // Fixed width that fits the card design well
      height: 420,    // Fixed height allowing space for content without compression
      center: true,
      transparent: true,
      resizable: false,
      skipTaskbar: false,
      focus: true,
      shadow: false,
      visible: false, // Start hidden to prevent white flash
    })

    webview.once('tauri://created', function () {
      console.log('Block modal window created')
    })

    webview.once('tauri://error', function (e) {
      console.error('Error creating block modal window', e)
    })
  }, [])

  const hideModal = useCallback(async () => {
    const label = 'block-warning-modal'
    const existing = await WebviewWindow.getByLabel(label)
    if (existing) {
      await existing.close()
    }
  }, [])

  useEffect(() => {
    const unlistenPromise = listen('app-blocked', (event) => {
      console.log('Received app-blocked event:', event.payload);
      showModal(event.payload as string);
    });

    return () => {
      unlistenPromise.then(unlisten => unlisten());
    }
  }, [showModal]);

  return {
    showModal,
    hideModal
  }
} 