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
- **Persistent Configuration** – Auto-save and restore music list between sessions

### 3. Reliability & Control
- **Debounce Logic** – Configurable cooldown (default 8s) prevents duplicate triggers
- **False-Positive Filtering** – Optional morphological opening for noise reduction
- **Pause/Resume Controls** – Keyboard shortcuts (Cmd+1) or GUI buttons
- **Detection Counter** – Track goals detected per session

### 4. Performance Monitoring
- **Benchmark Mode** (`--bench`) – Measures p50/p95/p99 latency across 500 iterations
- **Stage Breakdown** – Separate timing for capture, preprocess, OCR, audio trigger
- **Bottleneck Identification** – Highlights slowest pipeline stage
- **Performance Target** – Validates p95 < 100ms requirement

### 5. User Interfaces
- **CLI Version** – Lightweight command-line tool with keyboard controls
- **GUI Version** – User-friendly interface with visual region selection
- **Test Mode** (`--test`) – Verify OCR detection on current screen region
- **Status Indicators** – Real-time state (running/paused/stopped) and detection count

### 6. Configuration System
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

### Step 1: Core Detection Pipeline ✅
**Goal:** Implement baseline capture → OCR → audio trigger loop

**Tasks:**
- Setup Rust project with Cargo.toml dependencies
- Implement `src/capture.rs` – Screen capture with scap
- Implement `src/ocr.rs` – Tesseract OCR wrapper
- Implement `src/audio.rs` – Audio preloading and playback
- Implement `src/main.rs` – Main detection loop
- Add keyboard controls (Cmd+1 pause, Ctrl+C quit)

**Deliverables:**
- Working CLI binary `fm-goal-musics`
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
  "audio_file_path": "goal.mp3",
  "ocr_threshold": 0,
  "debounce_ms": 8000,
  "enable_morph_open": false,
  "bench_frames": 500,
  "music_list": [],
  "selected_music_index": 0
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

### Step 4: Latency Instrumentation ✅
**Goal:** Performance measurement and validation

**Tasks:**
- Implement `src/utils.rs` timing infrastructure
- Add `IterationTiming` struct for per-stage metrics
- Add `LatencyStats` with percentile calculations
- Implement `--bench` CLI flag
- Generate formatted performance reports

**Metrics Tracked:**
- Capture time
- Preprocessing time
- OCR time
- Audio trigger time
- Total end-to-end latency

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
- 37 total tests
- OCR: 8 tests
- Utils: 13 tests
- Audio: 2 tests
- Capture: 2 tests
- Config: 2 tests
- Integration: 10 tests

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

## Current Project Status

### Completed Milestones ✅
- [x] All implementation steps (0-9) completed
- [x] CLI version fully functional
- [x] GUI version fully functional
- [x] Performance target met (p95 < 100ms)
- [x] Multi-platform support (macOS, Windows, Linux)
- [x] Comprehensive test coverage (37 tests)
- [x] Production-ready quality

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
| T1.1 | Core capture module | CLI | High | ✅ |
| T1.2 | OCR detection module | CLI | High | ✅ |
| T1.3 | Audio playback module | CLI | High | ✅ |
| T1.4 | Main detection loop | CLI | High | ✅ |
| T2.1 | Config persistence | Core | High | ✅ |
| T3.1 | Audio converter | Core | Medium | ✅ |
| T3.2 | Music list management | Core | Medium | ✅ |
| T4.1 | Benchmark instrumentation | CLI | Medium | ✅ |
| T5.1 | Debounce logic | Core | High | ✅ |
| T5.2 | Threshold configuration | Core | Medium | ✅ |
| T5.3 | Morphological filtering | Core | Low | ✅ |
| T6.1 | Platform documentation | Documentation | Medium | ✅ |
| T7.1 | Unit test suite | Testing | High | ✅ |
| T7.2 | Integration tests | Testing | Medium | ✅ |
| T8.1 | GUI implementation | GUI | High | ✅ |
| T8.2 | Region selector | GUI | High | ✅ |
| T9.1 | Release documentation | Documentation | High | ✅ |

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
| GUI/CLI divergence | Medium | Low | Shared config schema, synchronized updates | ✅ Mitigated |
| Platform incompatibility | High | Low | Comprehensive platform testing, documented requirements | ✅ Mitigated |
| Memory leaks | Medium | Low | Rust memory safety, allocation-free hot path | ✅ Mitigated |

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
- [x] Comprehensive test coverage (37 tests)
- [x] Zero unsafe code
- [x] No compiler warnings
- [x] Clean code architecture

## Future Enhancements

### Planned Features (Backlog)
1. **Team-Specific Audio** – Different celebration sounds per team
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
