import './index.css' // ✅ import styles

import React from "react"
import { createRoot } from "react-dom/client"
import AppBlockModal from "./components/AppBlockModal"
import { Window } from "@tauri-apps/api/window"

const close = () => Window.getCurrent().close()

createRoot(document.getElementById("modal-root")!).render(
  <AppBlockModal
    isVisible={true}
    onClose={close}
    onUseFor5Mins={close}
    onShowAgain={close}
  />
)
