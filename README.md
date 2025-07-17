# synapse-revamped

---

## 🧱 Tech Stack Overview

### 🖥 Frontend

* **Desktop UI**: [Tauri](https://tauri.app/) + Svelte *(or React)* — modern, lightweight UI for prompts, logs, and settings
* **Mobile App**: Flutter — displays real-time popups based on desktop focus state
* **Realtime Sync**: Supabase Realtime — syncs focus events from desktop to mobile

### ⚙️ Backend

* **Core Engine**: Rust — detects active apps, enforces blacklists, logs events
* **Process Monitoring**: `sysinfo`, `windows` crate — foreground app detection
* **Cloud Sync**: Supabase (PostgreSQL + REST + Realtime) — stores logs, justifications, and state
* **Local Storage**: `rusqlite` or JSON — offline caching of events and config

---
