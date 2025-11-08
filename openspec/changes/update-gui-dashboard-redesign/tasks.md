## 1. Implementation
- [ ] 1.1 Update theme tokens to dark-red palette (primary `#EA2831`, surfaces `#121212`/`#1E1E1E`, success `#39FF14`)
- [ ] 1.2 Implement Sidebar: branding, Start/Stop button, navigation list (Dashboard, Library, Team Selection, Detection, Settings, Help)
- [ ] 1.3 Implement Dashboard header with status chip (Running/Stopped)
- [ ] 1.4 Implement Team Selection callout with "Configure" action → navigates to Team Selection
- [ ] 1.5 Implement cards: "Goal Music" and "Other Music" with browse button → navigates to Library
- [ ] 1.6 Wire interactions to controller/state (start/stop, nav). Reuse existing methods in `GuiController` and `AppTab`
- [ ] 1.7 Responsive/polish: reasonable layout at 1180×760 window; non-overflowing components
- [ ] 1.8 QA: `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test`
- [ ] 1.9 Update README/Doc screenshots if needed

## 2. Validation
- [ ] 2.1 `openspec validate update-gui-dashboard-redesign --strict`
- [ ] 2.2 Compile and run smoke test of GUI
