# ⚠️ DEPRECATED - Combined Development Backlog

> **THIS FILE IS DEPRECATED AS OF 2025-11-03**
>
> The backlog has been reorganized into a focus-driven structure for better project management.
>
> **Please use the new files instead**:
> - **`ROADMAP.md`** - Vision, milestones, release plan
> - **`CURRENT_SPRINT.md`** - Active work this week (your daily driver!)
> - **`BUGS.md`** - All bugs by priority (P0-P2)
> - **`FEATURES.md`** - Accepted features by release
> - **`IDEAS.md`** - Deferred/unvalidated ideas
>
> This file is kept for historical reference only.
>
> ---
>
> # Combined Development Backlog (ARCHIVED)
>
> This file previously tracked all ongoing and planned work for FMGoalMusic, including:
> - **Bugs** and technical improvements (P0-P2 priority)
> - **Feature roadmap** (future planned capabilities)
> - **Adoption ideas** (UX improvements to boost usage)
>
> Priorities followed the legend below. Each item included proposed solutions and acceptance criteria.

# Bugs and Improvements Backlog

## Priority Legend
 
- **P0**: Must fix now (critical impact or quick win with high value)
- **P1**: Should fix soon (important, not blocking)
- **P2**: Nice-to-have (valuable, lower urgency)

---

## P0 — Must Fix Now

- **[BUG] Multi-monitor: auto-detect FM screen + manual selection in Settings**
  - Symptom: When multiple monitors are connected, the app captures only one monitor and cannot be changed. We must target the monitor where Football Manager 26 is running.
  - Why it matters: Wrong source breaks OCR and the entire workflow.
  - Proposed approach:
    - Enumerate displays and windows; auto-detect the monitor hosting the FM game window (process/window title heuristics).
    - Provide a Settings dropdown for manual override (“Capture display: X”) with live preview.
    - Persist last successful selection in config; fall back to it if auto-detect fails.
  - Acceptance criteria:
    - On multi-monitor setups, default capture follows the FM window automatically.
    - User can change the target display in Settings and it persists across restarts.
    - Live capture preview reflects the selected display immediately.
  - Notes/Dependencies: `xcap` (capture), `display-info` (enumerate displays), OS window enumeration (Win/macOS) for FM window detection; eframe/egui for Settings UI.

- **[BUG/TECH DEBT] Windows: hide console window on launch + logging to file**
  - Symptom: On Windows, the app launches with a console (cmd) window; logging is noisy and not structured.
  - Proposed approach:
    - Hide console by building the GUI binary with the Windows subsystem (no console), apply in the GUI entry (e.g., `src/gui_main.rs`).
    - Replace env-based stdout logging with structured file logging (daily/size-rotating files, levels, timestamps). Consider `tracing` + `tracing-subscriber` + `tracing-appender` or `flexi_logger`.
    - Store logs under a per-platform app data directory (via `dirs`), e.g., `~/Library/Logs/FMGoalMusic`, `%AppData%/FMGoalMusic/logs`.
    - Add a Settings toggle for log level (Info/Debug) and a button to “Open logs folder”.
  - Acceptance criteria:
    - No console window appears on Windows startup.
    - Logs are written to rotating files with timestamps and levels, not to the terminal.
    - User can change log level and open the logs folder from Settings.
  - Notes/Dependencies: `winres` already present; adopt `tracing` stack or `flexi_logger` for file logging.

---

## P1 — Should Fix Soon

- **[I18N] OCR phrases: make “Goal for / gol” dynamic and language-aware**
  - Symptom: Goal phrases are hardcoded, breaking OCR for other languages.
  - Proposed approach:
    - Introduce a small lexicon (JSON) with per-language phrases and synonyms for goal-related text.
    - Provide a user-editable UI in Settings: select language and add custom phrases for missing languages.
    - Apply Unicode normalization and simple fuzzy matching to improve robustness.
  - Acceptance criteria:
    - User can select language and manage goal phrases.
    - OCR correctly detects goals in configured languages without code changes.
  - Notes/Dependencies: `serde/serde_json`, existing OCR pipeline (`leptess`).

- **[UX] Exit behavior: “Quit” vs “Run in background” (+ system tray)**
  - Symptom: Closing the window fully exits the app; no option to keep running.
  - Proposed approach:
    - On close, prompt: “Exit app” or “Keep running in background”.
    - If background is chosen, minimize to system tray with actions (Start/Stop capture, Open, Quit).
  - Acceptance criteria:
    - Close prompt appears; background mode works reliably.
    - Tray icon provides quick actions and re-opens the window.
  - Notes/Dependencies: eframe/winit integration with a system tray helper (e.g., `tray-icon`).

- **[UPDATE] Update checker and optional in-app updater**
  - Symptom: Users are not informed of new versions; manual updates cause friction.
  - Proposed approach:
    - Phase 1 (Notify): On startup and via Settings, check the latest version from GitHub Releases or a lightweight `version.json` endpoint. Show changelog and a “Download” button (opens release URL).
    - Phase 2 (In-app update): Download OS-specific artifact. On Windows, replace the binary on next launch using a small helper; on macOS, download .dmg/.zip and guide the user to replace the app bundle. Verify checksum/signature.
    - Safety & UX: Support “Skip this version”, disable auto-checks, and configurable check cadence.
    - Settings: toggle for auto-check on startup and a “Check for updates now” button.
  - Acceptance criteria:
    - App checks for updates and notifies with version and changelog.
    - Optional in-app download + guided install works on Windows and macOS.
    - Update checks are configurable and non-intrusive.
  - Notes/Dependencies: HTTP client (`reqwest`/`ureq`), GitHub Releases API or custom endpoint, optional `self_update` crate for Windows.

---

## P2 — Nice-to-have (High Value, More Scope)

- **[DATA/UX] Manage leagues/teams and name variations in-app**
  - Symptom: Not all leagues/teams are supported; names vary by language, and `team.json` variations may be incomplete.
  - Proposed approach:
    - Define a robust JSON schema for leagues, teams, and name variations (per language and aliases).
    - Add a simple CRUD UI: edit existing teams/leagues, add new ones, and add name variations.
    - Keep a read-only “base dataset” plus a user override file; merge at runtime.
  - Acceptance criteria:
    - Users can add/edit leagues/teams/aliases via the UI without editing files manually.
    - OCR/team matching uses merged data and works across languages.
  - Notes/Dependencies: `serde/serde_json`, persistence in app config directory.

---

## Quick Wins (suggested first steps)
 
- **Hide Windows console window** (very fast).
- **Add manual display selection in Settings** (before full auto-detect).
- **Introduce basic file logging with rotation** (then iterate), keep a simple “Open logs folder” button.

## Tracking IDs
 
- P0-01: Multi-monitor detection/selection
- P0-02: Windows console + logging overhaul
- P1-01: Dynamic OCR goal phrases
- P1-02: Exit to background + tray
- P2-01: Teams/leagues dataset manager

## Roadmap — Future Features

- **[AUDIO] Reverb effect for goal music (P1)**
  - Proposed approach: Apply a lightweight convolution reverb at load time with adjustable mix/decay; avoid real-time heavy DSP initially.
  - Acceptance criteria: Users can enable reverb per goal sound and adjust mix/decay; playback remains smooth.
  - Notes/Dependencies: Simple convolution or `dasp`/minimal DSP utility.

- **[AUDIO] Match start crowd sound (P1)**
  - Proposed approach: Detect kickoff via OCR (time resets to 00:00 or specific keywords) and play a one-shot ambience.
  - Acceptance criteria: Triggers once at kickoff; volume configurable.

- **[AUDIO] Match end crowd sound, score-aware (P1)**
  - Proposed approach: Detect end (FT/Full Time) and choose cheer/neutral/boo based on result.
  - Acceptance criteria: Correct variant plays by result; volume configurable.

- **[AUDIO] Dynamic crowd bed by score (P2)**
  - Proposed approach: Loop ambience layers and switch intensity based on lead/deficit; crossfade transitions.
  - Acceptance criteria: Seamless transitions; low CPU.

- **[AUDIO] Team chants during play (P2)**
  - Proposed approach: Random/periodic triggers with cooldowns; team-specific assets.
  - Acceptance criteria: No overlap with goal music; respects cooldowns.

- **[AUDIO] Goal/VAR commentator voice (P2)**
  - Proposed approach: Support pre-recorded voice packs; trigger on goal and VAR/no-goal; optional TTS later.
  - Acceptance criteria: Plays appropriate line; supports volume ducking.

- **[AUDIO] Sound timing controls improvements (P1)**
  - Current: start offset + duration exist.
  - Proposed: add fade-in/out and ducking of background when goal music plays.
  - Acceptance criteria: Fade controls visible; ducking works reliably.

- **[AUDIO] Player-specific goal music (P2)**
  - Proposed approach: Map player names to audio; detect scorer via OCR; fallback to team default when unknown.
  - Acceptance criteria: Plays correct player track when recognized; safe fallback.

- **[PLAYBACK] Idle playlist when no match (P2)**
  - Proposed approach: Local playlist support first; later provide optional integration hooks for Spotify/YouTube Music (without bundling SDKs initially).
  - Acceptance criteria: Playlist plays when not in-match; auto-pauses on kickoff/goal.

- **[PERF] OCR gating/optimization (P1)**
  - Proposed approach: Reduce/stop OCR polling when FM is not visible or not in-match; adaptive cadence based on state.
  - Acceptance criteria: Noticeable CPU drop outside matches; no missed goals.

### Adoption ideas
- **Starter packs + one-click import/export** for sounds and phrases
- **First-run setup wizard** (language, monitor selection, sample sounds)
- **Global hotkeys** to pause/resume capture and mute/unmute
- **Optional, privacy-friendly telemetry** (crash/error counts only; opt-in)
