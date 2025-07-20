# Synapse Revamped

## Folder Structure

```
synapse-revamped/
│
├── main-logic/
│   ├── src/
│   │   ├── apprules.rs        # Loads and manages app whitelist/blacklist rules
│   │   ├── logger.rs          # Handles logging of events to file and database
│   │   ├── main.rs            # Main entry point, runs the event loop
│   │   ├── metrics.rs         # Tracks and summarizes app usage metrics
│   │   ├── platform/
│   │   │   ├── linux.rs       # Platform-specific logic for Linux
│   │   │   ├── mod.rs         # Platform abstraction
│   │   │   └── windows.rs     # Platform-specific logic for Windows
│   │   ├── session.rs         # Manages focus sessions and app polling
│   │   └── db.rs              # Handles all SQLite database operations
│   ├── apprules.json          # JSON file for custom whitelist/blacklist rules
│   ├── synapse_metrics.db     # SQLite database for app usage and session logs
│   ├── synapse.log            # Plaintext log of app events
│   ├── Cargo.toml             # Rust dependencies and project metadata
│   └── Cargo.lock             # Cargo dependency lockfile
│
├── target/                    # Build artifacts (auto-generated)
└── README.md                  # Project overview and structure (this file)
```

---

**For more details, see comments in each source file.**

to view focus sessins
sqlite3 synapse_metrics.db ".tables" ".mode column" ".headers on" "SELECT * FROM focus_sessions;"
sqlite3 synapse_metrics.db ".tables" ".mode column" ".headers on" "SELECT * FROM app_usage_events;"