# Bugs Backlog

## Priority Legend

- **P0**: Must fix now (critical impact, blocks core functionality, or quick win with high value)
- **P1**: Should fix soon (important, not blocking)
- **P2**: Nice-to-have (valuable, lower urgency)

---

## ✅ Fixed in v0.1 (Released 2025-11-03)

### [P0-01] teams.json in wrong location - users can't customize teams ✅

**Status**: ✅ **FIXED in v0.2.1**

**Release**: https://github.com/Mcahitcali/FMGoalMusic/releases/tag/v0.2.1

**What Was Fixed**:
- ✅ teams.json now loads from user config directory
  - Windows: `%APPDATA%\FMGoalMusic\teams.json`
  - macOS: `~/Library/Application Support/FMGoalMusic/teams.json`
  - Linux: `~/.config/FMGoalMusic/teams.json`
- ✅ Automatic first-run migration from embedded default
- ✅ Users can edit without admin rights
- ✅ UI error messages when team database fails to load
- ✅ Help text updated to match actual implementation

**Implementation**:
- Updated `src/teams.rs` to use `dirs::config_dir()` pattern
- Added `save()` method and `database_path()` helper
- Enhanced error messages in `src/gui/mod.rs`
- All acceptance criteria met

---

### [P0-02] Multi-monitor: Manual display selection ✅

**Status**: ✅ **FIXED in v0.2.2**

**Release**: https://github.com/Mcahitcali/FMGoalMusic/releases/tag/v0.2.2

**What Was Fixed**:
- ✅ Settings has dropdown for monitor selection
  - Shows "Monitor 1 (Primary)", "Monitor 2 (Secondary)", etc.
- ✅ Selection persists in config.json (`selected_monitor_index`)
- ✅ Capture uses selected display immediately
- ✅ Automatic fallback to primary if selected monitor unavailable
- ✅ Works seamlessly with single or multiple monitors

**Implementation**:
- Added `selected_monitor_index: usize` field to Config and AppState
- Modified `CaptureManager::new()` to accept monitor index parameter
- Added ComboBox UI in Settings tab (Configuration section)
- All acceptance criteria met, tested with external monitor

---

### [P0-03a] Windows: Hide console window on launch ✅

**Status**: ✅ **FIXED in v0.2.3**

**Release**: https://github.com/Mcahitcali/FMGoalMusic/releases/tag/v0.2.3

**What Was Fixed**:
- ✅ No console window on Windows in release builds
- ✅ Debug builds keep console for development
- ✅ macOS/Linux behavior unchanged

**Implementation**:
- Added `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` to `src/gui_main.rs`
- Conditional compilation ensures console only hidden in release mode
- Completed in exactly 5 minutes as estimated!

---

### [P0-03b] File logging with rotation (structured logs to user directory) ✅

**Status**: ✅ **FIXED in v0.2.4**

**Release**: https://github.com/Mcahitcali/FMGoalMusic/releases/tag/v0.2.4

**What Was Fixed**:
- ✅ Logs written to rotating files with timestamps and levels
  - macOS: `~/Library/Application Support/FMGoalMusic/logs/`
  - Windows: `%APPDATA%/FMGoalMusic/logs/`
  - Linux: `~/.config/FMGoalMusic/logs/`
- ✅ Automatic rotation (10 MB max file size, keep last 5 files)
- ✅ All user actions logged (music, teams, detection, config changes)
- ✅ **BONUS**: Replaced 110 println!/eprintln! with proper log macros
- ✅ Startup diagnostics (version, OS, displays)
- ✅ Detailed config change tracking (shows old → new values)

**Implementation**:
- Replaced `env_logger` with `flexi_logger` in Cargo.toml
- Created comprehensive `initialize_logging()` function
- Replaced all 110 print statements across 12 files
- Enhanced with config change tracking and startup diagnostics
- All acceptance criteria met plus bonus features

---

## P1 — Should Fix Soon (v0.2-v0.3)

### [P1-01] Dynamic OCR goal phrases (internationalization)

**Status**: 🟡 **AFFECTS NON-ENGLISH USERS**

**Symptom**:
- Goal detection phrases hardcoded: "GOAL FOR" and "GOL " only
- Breaks OCR for Spanish, German, Italian, French, Portuguese, etc.

**Impact**:
- **MEDIUM-HIGH** - App unusable for non-English FM versions
- Limits addressable market to English speakers only

**Affected Platforms**: All

**Proposed Fix**:
1. Create `config/phrases.json` with per-language goal phrases
2. Add language dropdown in Settings (English, Spanish, Turkish, German, etc.)
3. Add custom phrase editor in Settings (for missing languages)
4. Apply Unicode normalization and fuzzy matching for robustness
5. Load and merge user overrides from user config directory

**Files to Change**:
- `src/ocr/text_extraction.rs` - Replace hardcoded phrases with dynamic lookup
- `config/phrases.json` - New file with language phrase packs
- `src/config.rs` - Add selected_language field
- `src/gui/mod.rs` - Settings tab: Language dropdown + custom phrase editor

**Time Estimate**: 6-8 hours with AI

**Dependencies**: None (serde_json already available)

**Acceptance Criteria**:
- [ ] User can select language in Settings
- [ ] OCR detects goals using selected language phrases
- [ ] User can add custom phrases for missing languages
- [ ] Changes persist and work without code changes

**Assigned to**: v0.3

---

### [P1-02] Exit behavior: "Quit" vs "Run in background" (system tray)

**Status**: 🟡 **UX IMPROVEMENT**

**Symptom**:
- Closing window fully exits app
- No option to keep running in background
- Users must keep window open during matches

**Impact**:
- **MEDIUM** - UX friction, users want to minimize
- Not blocking core functionality

**Affected Platforms**: All

**Proposed Fix**:
1. On close event, show prompt: "Exit app" or "Keep running in background"
2. If background chosen, minimize to system tray
3. Tray icon with menu: Start/Stop capture, Open window, Quit
4. Remember last choice in config

**Files to Change**:
- `src/gui_main.rs` - Handle close event
- `src/gui/mod.rs` - Close prompt dialog
- `Cargo.toml` - Add tray-icon dependency
- New module for tray management

**Time Estimate**: 3-4 hours with AI

**Dependencies**: tray-icon crate (egui/winit integration tricky)

**Acceptance Criteria**:
- [ ] Close prompt appears with two options
- [ ] Background mode keeps capture running
- [ ] Tray icon provides quick actions
- [ ] Tray icon can re-open main window

**Assigned to**: Deferred (not in v0.1-v0.3) - See IDEAS.md

**Note**: Deferred due to complexity vs value trade-off for part-time development. Users can minimize window as workaround.

---

## P2 — Nice-to-Have (Future)

### [P2-01] OCR performance optimization (adaptive polling)

**Symptom**:
- OCR runs continuously even when FM not visible
- CPU usage higher than needed outside matches

**Proposed Fix**:
- Reduce/stop OCR polling when FM window not focused
- Adaptive cadence based on match state (slower during gameplay, faster near goal moments)

**Time Estimate**: 4-6 hours

**Assigned to**: Future (after v0.3)

---

### [P2-02] Enhanced error messages and validation

**Symptom**:
- Various error cases show generic messages
- Config validation could be more helpful

**Proposed Fix**:
- Improve all error messages with actionable guidance
- Add config validation with helpful warnings

**Time Estimate**: 2-3 hours

**Assigned to**: Future (after v0.3)

---

## Tracking IDs

**Fixed (v0.1)**:
- ✅ P0-01: teams.json user config directory fix (v0.2.1)
- ✅ P0-02: Multi-monitor manual selection (v0.2.2)
- ✅ P0-03a: Hide Windows console (v0.2.3)
- ✅ P0-03b: File logging with rotation (v0.2.4)

**Planned**:
- P1-01: Dynamic OCR goal phrases (i18n) - v0.3
- P1-02: Exit to background + system tray - Deferred

**Future**:
- P2-01: OCR performance optimization
- P2-02: Enhanced error messages

---

**Last Updated**: 2025-11-03 (v0.1 milestone complete)
**Next Review**: After v0.2 features complete
