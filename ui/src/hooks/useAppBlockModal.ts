import { useCallback, useEffect } from 'react'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { listen } from '@tauri-apps/api/event'
import { LogicalSize } from '@tauri-apps/api/dpi'

interface UseAppBlockModalReturn {
  showModal: () => void
  hideModal: () => void
}

export function useAppBlockModal(): UseAppBlockModalReturn {
  const showModal = useCallback(async () => {
    // Check if window already exists
    const label = 'block-warning-modal'
    const existing = await WebviewWindow.getByLabel(label)

    if (existing) {
      // Ensure size is correct even if recycled
      await existing.hide() // Hide first to reset any state/flash
      await existing.setSize(new LogicalSize(360, 420))
      // The page component will trigger show() when ready if we re-navigate or reload,
      // but since we are just showing an existing window, we might need to rely on the event.
      // Actually, if we just show it here, we risk the flash if it was previously closed/hidden but not destroyed.
      // However, if the page is already mounted, the useEffect won't run again unless we reload.
      // Let's just show it here for now, as the page should still be transparent.
      await existing.show()
      await existing.setFocus()
      return
    }

    const webview = new WebviewWindow(label, {
      url: '/block-warning',
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
      showModal();
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