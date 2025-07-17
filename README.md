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


the barebones desktop version does this:

Desktop Flow:
1 Monitor foreground window

2 If blacklisted app is active → show popup prompt

3 Log app + justification + timestamp

4 Send to Supabase via REST

focus-monitor/
├── src/
│   ├── main.rs                  # Entry point
│   ├── monitor.rs               # Foreground app detection
│   ├── popup.rs                 # Justification prompt
│   └── sync.rs                  # Send to Supabase
├── .env                         # Supabase URL & key
├── Cargo.toml
├── README.md

| File / Folder | Purpose                                      |
| ------------- | -------------------------------------------- |
| `main.rs`     | Loop + glue logic between modules            |
| `monitor.rs`  | Get foreground app/process name              |
| `popup.rs`    | Show prompt and get user input               |
| `sync.rs`     | Struct + function to POST to Supabase        |
| `.env`        | Store your `SUPABASE_URL` and `SUPABASE_KEY` |
| `README.md`   | Setup + dev guide for team                   |
