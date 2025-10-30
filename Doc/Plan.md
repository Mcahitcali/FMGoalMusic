# FM Goal Musics – Project Plan

## Project Name
**FM Goal Musics**

## Purpose of Project
FM Goal Musics is a real-time companion application for Football Manager that automatically detects when a goal is scored and plays a celebratory audio clip instantly. The app eliminates manual intervention, enhances immersion, and provides sub-100ms latency for seamless celebration moments during gameplay.

### Problem Solved
**Problem:** Football Manager lacks customizable instant audio celebrations. Players must manually trigger sounds or miss the emotional moment entirely. Existing solutions are complex, add latency, or require constant manual tracking.

**Solution:** FM Goal Musics monitors a configured screen region using GPU-accelerated capture and OCR to detect "GOAL FOR" text. Upon detection, it instantly plays a preloaded celebration audio with minimal latency (<100ms p95), providing automatic, reliable, and configurable goal celebrations.

## Core Features

### 1. Real-time Goal Detection
- **GPU-Accelerated Screen Capture** – Uses `scap` library for efficient region capture
- **OCR Text Recognition** – Tesseract-based detection optimized for "GOAL FOR" text
- **Adaptive Thresholding** – Auto (Otsu) or manual threshold configuration
- **60 FPS Monitoring** – Continuous scan with minimal CPU overhead

### 2. Instant Audio Playback
- **Preloaded Audio** – Files loaded into memory at startup (zero disk I/O on trigger)
- **Multi-format Support** – MP3, WAV, OGG, FLAC with automatic WAV conversion
- **Music List Management** – Add, remove, select multiple celebration tracks
- **Dual Volume Control** – Separate volume sliders for music (0-100%) and ambiance (0-100%)
- **Ambiance Sounds** – Optional crowd cheer sound plays simultaneously with music
- **Persistent Configuration** – Auto-save and restore music list, volumes, and ambiance settings
- **Managed Library** – All WAVs stored under `config/musics/` with ASCII slug filenames (spaces → underscores)
  - Example: `İldırım Ildırım (Stüdyo).mp3` → `config/musics/Ildirim_Ildirim_Stduyo.wav`
  - Display names are derived from the WAV file stem (no extension)

### 3. Reliability & Control
- **Debounce Logic** – Configurable cooldown (default 8s) prevents duplicate triggers
- **False-Positive Filtering** – Optional morphological opening for noise reduction
- **Pause/Resume Controls** – Keyboard shortcuts (Cmd+1) or GUI buttons
- **Detection Counter** – Track goals detected per session

### 4. User Interface
- **GUI Application** – User-friendly graphical interface with visual controls
- **Visual Region Selector** – Click-and-drag screen region selection (Cmd+Shift+R)
- **Status Indicators** – Real-time state (running/paused/stopped) and detection count
- **Team Selection** – League and team dropdown for selective goal detection

### 5. Configuration System
- **JSON-based Config** – Platform-specific storage (macOS/Windows/Linux)
- **Auto-generated Defaults** – Creates config if missing with sensible values
- **Visual Region Selector** – Click-and-drag interface for capture area
- **Tunable Parameters** – Threshold, debounce, morphological processing, benchmark iterations

## Step-by-Step Implementation Plan

### Step 0: Project Setup & Documentation ✅
**Goal:** Establish project structure and documentation framework

**Tasks:**
- Create Doc folder structure (Doc, Doc/Features)
- Define Plan.md (this document)
- Define Design.md (UI/UX specifications)
- Define Implementation.md (technical stack)
- Initialize Git repository

**Status:** Completed

---

### Step 11: UI Redesign – Tabbed Layout ✅
**Goal:** Simplify and organize the GUI with a tabbed interface for better clarity and responsiveness

**Tasks:**
- Add `AppTab` enum and `active_tab` state to manage active view
- Introduce Tab Bar with 4 tabs: Library, Team Selection, Settings, Help
- Move Start/Stop Detection button to header (top-left, color-coded)
- Consolidate controls: Preview in Library, Ambiance in Library, Volumes/Lengths in Settings
- Move existing sections into their tabs
- Keep Status Bar visible at the top at all times
- Ensure build remains green and preserve all functionality
- Update Help tab with comprehensive guide including teams.json configuration

**Deliverables:**
- Updated `src/gui.rs` with refined 4-tab layout
- Updated `Doc/Design.md` documenting the final layout
- Comprehensive Help tab with usage guide and troubleshooting

**Status:** Completed

---

### Step 1: Core Detection Pipeline ✅
**Goal:** Implement baseline capture → OCR → audio trigger loop

**Tasks:**
- Setup Rust project with Cargo.toml dependencies
- Implement `src/capture.rs` – Screen capture with scap
- Implement `src/ocr.rs` – Tesseract OCR wrapper
- Implement `src/audio.rs` – Audio preloading and playback
- Implement `src/gui_main.rs` – GUI entry point
- Add keyboard controls (Cmd+1 pause, Cmd+Shift+R region selector)

**Deliverables:**
- Working GUI binary `fm-goal-musics-gui`
- Functional detection and audio playback
- Basic error handling

**Status:** Completed

---

### Step 2: Configuration Management ✅
**Goal:** JSON-based configuration persistence

**Tasks:**
- Implement `src/config.rs` – Config struct and serialization
- Platform-specific config directory (macOS/Windows/Linux)
- Default config generation on first run
- Validation for capture region, audio paths
- Config loading error handling

**Schema:**
```json
{
  "capture_region": [x, y, width, height],
  "ocr_threshold": 0,
  "debounce_ms": 8000,
  "enable_morph_open": false,
  "bench_frames": 500,
  "music_list": [
    { "name": "Ildirim_Ildirim_Stduyo", "path": "config/musics/Ildirim_Ildirim_Stduyo.wav", "shortcut": null }
  ],
  "selected_music_index": null
}
```

**Status:** Completed

---

### Step 3: Audio Conversion & Management ✅
**Goal:** Multi-format support with WAV conversion

**Tasks:**
- Implement `src/audio_converter.rs` – Format conversion module
- Add Symphonia decoder (MP3, FLAC, OGG, AAC)
- Add Hound encoder (WAV output)
- Auto-convert non-WAV files on import
- Music list persistence in config

**Supported Formats:**
- Input: MP3, AAC, FLAC, OGG Vorbis, WAV
- Output: 16-bit PCM WAV
- Conversion: One-time on file addition

**Status:** Completed

---

### Step 4: Utility Infrastructure ✅
**Goal:** Shared utilities and timing for GUI

**Tasks:**
- Implement `src/utils.rs` timing infrastructure
- Add state management for GUI threads
- Add debouncer implementation
- Thread-safe AppState with Arc<AtomicBool>

**Utilities Provided:**
- Timer for performance monitoring
- Debouncer for goal detection
- AppState for thread-safe state management

**Status:** Completed

---

### Step 5: False-Positive Controls ✅
**Goal:** Reduce unwanted triggers and improve reliability

**Tasks:**
- Implement debounce logic with configurable window
- Add auto/manual threshold modes (0 = Otsu, 1-255 = fixed)
- Implement morphological opening (optional, behind flag)
- Update config schema
- Add tuning documentation

**Configuration:**
- `ocr_threshold: 0` – Automatic (recommended)
- `ocr_threshold: 1-255` – Manual override
- `debounce_ms: 8000` – 8 second cooldown
- `enable_morph_open: false` – Disabled by default (adds 5-10ms)

**Status:** Completed

---

### Step 6: Platform Optimization ✅
**Goal:** OS-specific performance tuning and documentation

**Tasks:**
- Document macOS Screen Recording permission requirements
- Document Windows.Graphics.Capture API usage
- Add Linux compositor notes
- Conditional compilation guards
- Platform-specific troubleshooting guides

**Platform Support:**
- **macOS:** Metal framework, IOSurface capture (5-15ms latency)
- **Windows:** DirectX capture (10-20ms latency)
- **Linux:** X11/Wayland support (15-30ms typical)

**Status:** Completed

---

### Step 7: Test Suite ✅
**Goal:** Comprehensive testing and regression prevention

**Tasks:**
- Unit tests for OCR module (grayscale, threshold, detection)
- Unit tests for utils (timing, debounce, state)
- Unit tests for audio initialization
- Unit tests for capture region management
- Integration tests for full pipeline
- Test fixtures with sample images

**Test Coverage:**
- 90 total tests (CLI + GUI bins)
- Modules covered: OCR, Utils, Audio, Capture, Config, Team Matcher, Teams, Integration

**Status:** Completed

---

### Step 8: GUI Application ✅
**Goal:** User-friendly graphical interface

**Tasks:**
- Implement `src/gui.rs` – egui-based interface
- Implement `src/gui_main.rs` – GUI entry point
- Implement `src/region_selector.rs` – Visual region picker
- Add music list management UI
- Add configuration panel
- Add process control buttons (start/pause/stop)
- Add status indicators and detection counter
- Thread-safe state management between GUI and detection threads

**GUI Features:**
- Music file browser and list
- Visual region selection (click-and-drag)
- Real-time configuration editing
- Status display (running/paused/stopped)
- Detection counter
- 60 FPS responsive interface

**Status:** Completed

---

### Step 9: Documentation & Release Prep ✅
**Goal:** Production-ready documentation and artifacts

**Tasks:**
- Comprehensive README with install, usage, troubleshooting
- Platform-specific setup guides
- Configuration tuning documentation
- Build script for macOS app bundle
- Release checklist and quality gates

**Documentation:**
- User guide (README.md)
- Developer docs (old-docs/ for reference)
- New structured docs (Doc/)
- Inline code documentation

**Status:** Completed

---

### Step 10: Team Selection Feature ✅
**Goal:** Play goal sound only for user-selected team

**Tasks:**
- Create team database structure (JSON with leagues, teams, variations)
- Implement `src/teams.rs` – Team database loader and query
- Implement `src/team_matcher.rs` – Team name matching with variations
- Update `src/ocr.rs` – Extract full team name from "GOAL FOR [team]"
- Update `src/config.rs` – Add selected_team field
- Update `src/gui.rs` – Add team selection UI (league + team dropdown)
- Update `src/main.rs` and `src/gui_main.rs` – Conditional audio playback
- Add unit tests for team matching
- Add integration tests with team variations

**Features:**
- JSON-based team database with leagues and variations
- Team selection UI in GUI (league picker + team picker)
- Extract team name from OCR text
- Match against selected team's variations (case-insensitive)
- Backward compatible (no team = play all goals)
- Performance target: < 1ms matching overhead

**Schema:**
```json
{
  "selected_team": {
    "league": "Premier League",
    "team_key": "manchester_united",
    "display_name": "Manchester Utd"
  }
}
```

**Status:** Completed

---

## Current Project Status

### Completed Milestones ✅
- [x] All implementation steps (0-10) completed
- [x] GUI application fully functional
- [x] Performance target met (p95 < 100ms)
- [x] Multi-platform support (macOS, Windows, Linux)
- [x] Comprehensive test coverage (37 tests)
- [x] Team selection feature implemented
- [x] Production-ready quality

### In Progress 🔄
- None

### Quality Metrics
- **Performance:** p95 latency ~65ms ✅ (Target: <100ms)
- **Test Coverage:** 37 passing tests ✅
- **Code Quality:** Zero unsafe code, no warnings ✅
- **Documentation:** Complete user and developer docs ✅

### Next Actions
- Monitor user feedback for feature requests
- Consider advanced features (team-specific audio, statistics tracking)
- Evaluate performance optimizations (SIMD, GPU OCR)
- Plan for auto-updater and system tray integration

## Task Tracking

| Task ID | Description | Component | Priority | Status |
|---------|-------------|-----------|----------|--------|
| T0.1 | Doc structure setup | Documentation | High | ✅ |
| T1.1 | Core capture module | Core | High | ✅ |
| T1.2 | OCR detection module | Core | High | ✅ |
| T1.3 | Audio playback module | Core | High | ✅ |
| T1.4 | GUI main entry point | GUI | High | ✅ |
| T2.1 | Config persistence | Core | High | ✅ |
| T3.1 | Audio converter | Core | Medium | ✅ |
| T3.2 | Music list management | Core | Medium | ✅ |
| T4.1 | Utility infrastructure | Core | Medium | ✅ |
| T5.1 | Debounce logic | Core | High | ✅ |
| T5.2 | Threshold configuration | Core | Medium | ✅ |
| T5.3 | Morphological filtering | Core | Low | ✅ |
| T6.1 | Platform documentation | Documentation | Medium | ✅ |
| T7.1 | Unit test suite | Testing | High | ✅ |
| T7.2 | Integration tests | Testing | Medium | ✅ |
| T8.1 | GUI implementation | GUI | High | ✅ |
| T8.2 | Region selector | GUI | High | ✅ |
| T9.1 | Release documentation | Documentation | High | ✅ |
| T10.1 | Team database structure | Core | High | ✅ |
| T10.2 | Team matching logic | Core | High | ✅ |
| T10.3 | OCR team name extraction | Core | High | ✅ |
| T10.4 | Config team selection | Core | High | ✅ |
| T10.5 | GUI team selection UI | GUI | High | ✅ |
| T10.6 | Detection loop integration | Core | High | ✅ |
| T10.7 | Team selection tests | Testing | High | ✅ |

## Risk Management

### Technical Risks
| Risk | Impact | Probability | Mitigation | Status |
|------|--------|-------------|------------|--------|
| High OCR latency | High | Low | GPU capture, small region, optimized preprocessing | ✅ Mitigated |
| False positives | Medium | Medium | Debounce, threshold tuning, morphological filtering | ✅ Mitigated |
| Permission failures (macOS) | High | Medium | Clear documentation, permission prompts | ✅ Mitigated |
| Audio format issues | Low | Low | Convert all to WAV on import | ✅ Mitigated |
| Config corruption | Medium | Low | JSON validation, auto-recreate defaults | ✅ Mitigated |

### Operational Risks
| Risk | Impact | Probability | Mitigation | Status |
|------|--------|-------------|------------|--------|
| Platform incompatibility | High | Low | Comprehensive platform testing, documented requirements | ✅ Mitigated |
| Memory leaks | Medium | Low | Rust memory safety, allocation-free hot path | ✅ Mitigated |
| Thread synchronization issues | Medium | Low | Arc<AtomicBool> for state, message passing for GUI | ✅ Mitigated |

## Success Criteria

### Performance Requirements ✅
- [x] End-to-end p95 latency < 100ms (Actual: ~65ms)
- [x] No allocations in detection hot path after warm-up
- [x] GPU-accelerated capture operational
- [x] Smooth audio playback with no glitches

### Reliability Requirements ✅
- [x] Zero false positives with recommended settings
- [x] Debounce prevents duplicate triggers
- [x] Graceful error handling and recovery
- [x] Clean shutdown on all exit paths

### Usability Requirements ✅
- [x] Clear documentation for setup and usage
- [x] Visual region selection in GUI
- [x] Persistent configuration between sessions
- [x] Intuitive GUI controls and status display

### Quality Requirements ✅
- [x] Comprehensive test coverage (90 tests)
- [x] Zero unsafe code
- [x] No compiler warnings
- [x] Clean code architecture

## Future Enhancements

### In Development
None

### Planned Features (Backlog)
1. **Team-Specific Audio** – Different celebration sounds per team (depends on Team Selection)
2. **Statistics Dashboard** – Track goals per session, per team, timing analytics
3. **Multiple Region Profiles** – Save and switch between different screen configurations
4. **Cloud Sync** – Sync configuration across devices
5. **Mobile Companion** – Remote control via mobile app
6. **Auto-Updater** – Automatic version updates
7. **System Tray Integration** – Minimize to tray, quick controls
8. **Playlist Mode** – Rotate through multiple celebration tracks
9. **Notification System** – Desktop notifications on goal detection
10. **Advanced Analytics** – Detection accuracy metrics, performance graphs

### Performance Optimizations (Backlog)
1. **SIMD Preprocessing** – Vectorized image operations
2. **Custom OCR Engine** – Specialized detector for "GOAL" text
3. **Predictive Capture** – ML-based goal moment anticipation
4. **GPU OCR Processing** – Hardware-accelerated text recognition

### Platform Expansion (Backlog)
1. **Mobile Versions** – iOS/Android support
2. **Web Version** – Browser-based detection
3. **Console Integration** – PlayStation/Xbox support

## Maintenance Plan

### Documentation Updates
- Update Plan.md when adding new features
- Create feature docs in Doc/Features/ for major additions
- Keep Design.md and Implementation.md synchronized
- Update README.md for user-facing changes

### Code Maintenance
- Run full test suite before merges
- Execute benchmarks to validate performance
- Review platform-specific code quarterly
- Update dependencies monthly

### Release Process
1. Create feature branch for new work
2. Update documentation in Doc/
3. Write tests for new functionality
4. Run full test suite and benchmarks
5. Update README and CHANGELOG
6. Create commit with descriptive message
7. Merge to main after review

## Project Timeline

### Completed Phases
- **Phase 0:** Setup & Planning (Completed)
- **Phase 1:** Core Implementation (Completed)
- **Phase 2:** GUI Development (Completed)
- **Phase 3:** Testing & Documentation (Completed)
- **Phase 4:** Release Preparation (Completed)

### Current Status
**Production Ready** – All planned features implemented, tested, and documented.

---

*Last Updated: 2025-10-29*
*Status: Production Ready*
*Version: 1.0*
