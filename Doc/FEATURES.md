# Features Roadmap

This document tracks **accepted features** planned for upcoming releases. These are features that have been validated and prioritized based on user value, implementation complexity, and strategic alignment.

For unvalidated ideas and deferred items, see `IDEAS.md`.

---

## ✅ Shipped in v0.1 (FOUNDATION + CRITICAL FIXES) - Completed 2025-11-03

### Update Checker (Notify-Only) ✅ Shipped in v0.2.0

**Release**: https://github.com/Mcahitcali/FMGoalMusic/releases/tag/v0.2.0

**User Value**: Users automatically discover new releases and updates

**Description**:
On app startup and via Settings, check for latest version from GitHub Releases API. Show notification with version number, changelog, and "Download" button (opens GitHub release page in browser).

**Implemented**:
- ✅ Check GitHub Releases API on startup (non-blocking)
- ✅ Settings: "Check for updates now" button
- ✅ Show modal popup with version comparison
- ✅ Display changelog from release notes
- ✅ "Download" button → opens browser to release page
- ✅ "Skip this version" option
- ✅ Configurable: auto-check on startup (toggle in Settings)

**NOT in scope** (defer to future):
- In-app download/install
- Auto-update functionality
- Background update checks

**Implementation Details**:
- Used `ureq` for HTTP (lighter than reqwest)
- Parse GitHub API JSON response
- Compare semantic versions with `semver` crate
- Cache last check time (avoid spam)

**Time**: 4-6 hours with AI (estimated) / ~5 hours (actual)

**Files Modified**:
- New: `src/update_checker.rs`
- Modified: `src/gui/mod.rs` (Settings tab + startup check)
- Modified: `Cargo.toml` (add ureq, semver)

**Acceptance Criteria**: ✅ ALL MET
- ✅ App checks for updates on startup (if enabled)
- ✅ Settings has "Check for updates now" button
- ✅ Modal shows current vs latest version
- ✅ Changelog displayed from GitHub release notes
- ✅ "Download" button opens GitHub releases page
- ✅ "Skip this version" persists in config
- ✅ Auto-check can be disabled in Settings

---

## Planned for v0.2 (UNBLOCK + ATTRACT)

### Match Start Crowd Sound

**User Value**: Enhanced immersion with match atmosphere at kickoff

**Description**:
Detect match start via OCR (match time resets to "00:00" or "Kick Off" keyword) and play one-shot crowd ambience sound. Creates stadium atmosphere as match begins.

**Scope**:
- OCR enhancement to detect "00:00" or "Kick Off" text
- Trigger logic (play once per match start, avoid repeats)
- Load and play kickoff ambience sound
- Volume control in Settings (separate from goal music)
- Include default kickoff sound with app

**Implementation Notes**:
- Extend OCR text extraction to look for kickoff patterns
- Add debouncing (don't retrigger if already played recently)
- Reuse existing audio playback infrastructure
- State tracking: has_kickoff_played_this_match flag

**Time Estimate**: 4-6 hours with AI

**Dependencies**: None (uses existing OCR + audio systems)

**Priority**: HIGH (exciting new capability, good for marketing)

**Files**:
- Modify: `src/ocr/text_extraction.rs` (add kickoff detection)
- Modify: `src/audio.rs` (add kickoff sound manager)
- Modify: `src/gui/mod.rs` (Settings: kickoff volume control)
- Modify: `src/config.rs` (add kickoff_volume, kickoff_enabled)
- Add: `config/sound/kickoff_crowd.wav` (default sound)

**Acceptance Criteria**:
- [ ] Detects match start when time shows "00:00"
- [ ] Plays crowd ambience once at kickoff
- [ ] Volume configurable in Settings
- [ ] Can be disabled in Settings
- [ ] Doesn't retrigger if match paused/resumed
- [ ] Works across different FM UI languages (time-based detection)

---

### Enhanced Multi-Monitor (Polish from v0.1 MVP)

**User Value**: Seamless multi-monitor setup with visual feedback

**Description**:
Enhance the v0.1 basic dropdown with display information, live preview, and optional auto-detection of FM window.

**Scope**:
- Display dropdown shows: "Display 1: 1920x1080 (Primary)", "Display 2: 2560x1440"
- Live capture preview updates when selection changes
- Optional: Auto-detect which display has FM window (Win: FindWindow, macOS: Quartz)
- "Detect FM Window" button for easy setup

**Implementation Notes**:
- Use `display-info` crate to get resolution data
- Enumerate windows to find FM process
- Update preview without restarting detection

**Time Estimate**: 6-10 hours with AI

**Dependencies**: Window enumeration (platform-specific)

**Priority**: MEDIUM (polish on v0.1 MVP)

**Files**:
- Modify: `src/capture.rs` (add display info queries, window detection)
- Modify: `src/gui/mod.rs` (enhance Settings dropdown, add "Detect" button)

**Acceptance Criteria**:
- [ ] Dropdown shows resolution and "Primary" label
- [ ] Preview updates when selection changes
- [ ] "Detect FM Window" button auto-selects correct display
- [ ] Works on Windows and macOS

---

## Planned for v0.3 (GO INTERNATIONAL)

### Match End Crowd Sound (Score-Aware)

**User Value**: Complete match atmosphere with emotional final whistle

**Description**:
Detect full-time whistle via OCR ("FT", "Full Time", match time >= 90:00) and play appropriate crowd reaction based on match result (cheer if won, boo if lost, neutral if draw).

**Scope**:
- OCR enhancement to detect match end patterns
- Parse final score from OCR (e.g., "3-1 FT")
- Determine result (win/draw/loss) based on selected team
- Play appropriate sound variant
- Volume control in Settings

**Implementation Notes**:
- Extend OCR to detect "FT", "Full Time", etc.
- Score parsing logic (extract numbers from "3-1" format)
- Three sound files: cheer.wav, boo.wav, neutral.wav
- Result logic: compare scores to determine outcome

**Time Estimate**: 4-6 hours with AI

**Dependencies**: Enhanced OCR score parsing

**Priority**: MEDIUM (completes match atmosphere story)

**Files**:
- Modify: `src/ocr/text_extraction.rs` (add FT detection, score parsing)
- Modify: `src/audio.rs` (add match end sound manager)
- Modify: `src/gui/mod.rs` (Settings: match end volume control)
- Modify: `src/config.rs` (add match_end_volume, match_end_enabled)
- Add: `config/sound/crowd_cheer_win.wav`, `crowd_boo_loss.wav`, `crowd_neutral_draw.wav`

**Acceptance Criteria**:
- [ ] Detects match end at full time
- [ ] Correctly parses final score
- [ ] Plays cheer if selected team won
- [ ] Plays boo if selected team lost
- [ ] Plays neutral if draw
- [ ] Volume configurable in Settings
- [ ] Can be disabled in Settings
- [ ] Triggers once per match

---

## Planned for v0.4+ (FUTURE)

### Audio: Goal Music Reverb Effect

**User Value**: Professional stadium sound for goal music

**Description**:
Apply lightweight convolution reverb effect to goal music with adjustable mix and decay parameters.

**Scope**:
- Reverb effect at audio load time (not real-time)
- Settings controls: reverb enabled, mix (0-100%), decay time
- Apply per-sound or globally

**Time Estimate**: 4-6 hours with AI

**Priority**: MEDIUM (enhances existing core feature)

**Dependencies**: DSP library (dasp or simple convolution)

**Status**: Planned for v0.4+

---

### Audio: Sound Timing Controls

**User Value**: Precise control over audio playback

**Current State**:
- Start offset exists
- Duration limiting exists

**Enhancements**:
- Fade-in duration control (currently 200ms hardcoded)
- Fade-out duration control (currently 2s hardcoded)
- Ducking of background ambience when goal music plays

**Time Estimate**: 2-3 hours with AI

**Priority**: LOW (current defaults work well)

**Status**: Planned for v0.4+

---

### UX: Global Hotkeys

**User Value**: Quick control without switching windows

**Description**:
System-wide hotkeys to control app without focusing window.

**Scope**:
- Configurable hotkeys in Settings
- Actions: Start/Stop capture, Mute/Unmute, Pause goal music
- Respect OS hotkey conventions

**Time Estimate**: 3-4 hours with AI

**Dependencies**: `rdev` crate (already in Cargo.toml)

**Priority**: MEDIUM (nice UX improvement)

**Status**: Planned for v0.4+

---

### UX: First-Run Setup Wizard

**User Value**: Easy onboarding for new users

**Description**:
Multi-step wizard on first launch to configure essentials.

**Steps**:
1. Welcome + explain what app does
2. Language selection
3. Multi-monitor selection (if applicable)
4. Select favorite team
5. Import sample goal sounds (or use defaults)
6. Test setup (simulate goal detection)

**Time Estimate**: 6-8 hours with AI

**Priority**: MEDIUM (improves adoption)

**Status**: Planned for v0.4+

---

## Feature Tracking Summary

| Release | Feature | Priority | Time Est. | Status |
|---------|---------|----------|-----------|--------|
| v0.1 | Update checker (notify-only) | INFRA | 4-6h | ✅ Shipped (v0.2.0) |
| v0.1 | teams.json user config fix | CRITICAL | 2-4h | ✅ Shipped (v0.2.1) |
| v0.1 | Multi-monitor selection (MVP) | CRITICAL | 1-2h | ✅ Shipped (v0.2.2) |
| v0.1 | Hide Windows console | QUICK WIN | 5min | ✅ Shipped (v0.2.3) |
| v0.1 | File logging with rotation | CRITICAL | 3-4h | ✅ Shipped (v0.2.4) |
| v0.2 | Match start crowd sound | HIGH | 4-6h | Planned |
| v0.2 | Enhanced multi-monitor | MEDIUM | 6-10h | Planned |
| v0.3 | Match end crowd (score-aware) | MEDIUM | 4-6h | Planned |
| v0.4+ | Goal music reverb | MEDIUM | 4-6h | Future |
| v0.4+ | Sound timing controls | LOW | 2-3h | Future |
| v0.4+ | Global hotkeys | MEDIUM | 3-4h | Future |
| v0.4+ | First-run wizard | MEDIUM | 6-8h | Future |

---

## Deferred Features

See `IDEAS.md` for features intentionally deferred or not currently planned:
- Dynamic crowd bed by score
- Team chants during play
- Goal/VAR commentator voice
- Player-specific goal music
- Idle playlist when no match
- Teams/leagues CRUD UI (after v0.1 fixes teams.json bug)

---

**Last Updated**: 2025-11-04
**Next Review**: After v0.2 ships

---

## Release Notes

### v0.1 Milestone (Completed 2025-11-03)
All v0.1 features successfully shipped across 5 releases (v0.2.0 through v0.2.4). See CURRENT_SPRINT.md for detailed retrospective and BUGS.md for bug fix details.
