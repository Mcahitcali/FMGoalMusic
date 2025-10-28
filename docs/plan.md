# FM Goal Musics - Implementation Plan

## Project Overview
**Goal:** Build a low-latency background utility that detects "GOAL" text in a fixed screen region and plays celebration audio instantly.

**Hard Performance Target:** < 100ms end-to-end latency (capture → OCR → audio playback)

**Tech Stack:**
- `scap` - GPU-assisted screen capture
- `leptess` (Tesseract) - OCR engine
- `rodio` + `mp3` - Audio playback (preloaded in memory)
- `rdev` - Global hotkeys (F8 pause, F9 quit)
- `serde`/`serde_json`, `image`, `tray-item`, `dirs`

**Project Structure:**
```
src/
├── main.rs       - Entry point, hotkey handling, main loop
├── capture.rs    - Screen capture logic (scap wrapper)
├── ocr.rs        - OCR processing (leptess wrapper)
├── audio.rs      - Audio manager (rodio, preloading)
├── config.rs     - Configuration loading/saving
└── utils.rs      - Helper functions, timing utilities
```

---

## Implementation Steps

### Step A.1 — Cargo.toml Setup (Priority: HIGH)
**Goal:** Configure project dependencies and build settings

**Tasks:**
- Add all required dependencies with correct versions
- Dependencies: scap, leptess, rodio (with mp3 feature), rdev, serde, serde_json, image, dirs
- Configure release profile for maximum optimization
- Set binary name to `fm-goal-musics`

**Deliverables:**
- Complete `Cargo.toml` file
- Verify with `cargo check`

---

### Step A.2 — config.rs (Priority: HIGH)
**Goal:** Configuration loading and management

**Tasks:**
- Define `Config` struct matching schema
- Implement `load()` from platform config dir
- Implement `save()` to persist config
- Create defaults if config.json missing
- Print config path for user to edit

**Schema:**
```json
{
  "capture_region": [x, y, width, height],
  "audio_file_path": "path/to/goal.mp3",
  "ocr_threshold": 150,
  "debounce_ms": 800,
  "enable_morph_open": false,
  "bench_frames": 500
}
```

**Deliverables:**
- Complete `src/config.rs` file
- Test with standalone example if needed

---

### Step A.3 — audio.rs (Priority: HIGH)
**Goal:** Audio preloading and non-blocking playback

**Tasks:**
- Create `AudioManager` struct
- Preload MP3 file into memory at initialization
- Setup persistent `OutputStream` and `Sink`
- Implement `play_sound()` - must be non-blocking
- Warm decoder at startup
- Handle errors gracefully (missing file, etc.)

**Key Requirements:**
- No disk I/O on trigger
- Audio already in RAM
- Non-blocking playback

**Deliverables:**
- Complete `src/audio.rs` file
- Can test independently with a sample MP3

---

### Step A.4 — capture.rs (Priority: HIGH)
**Goal:** Screen capture with region selection

**Tasks:**
- Initialize `scap::Capturer` once (reuse instance)
- Implement `capture_region(x, y, w, h) -> Result<RgbaImage>`
- Single allocation reuse pattern
- Handle capture errors

**Key Requirements:**
- Reuse Capturer instance
- Minimize allocations
- Fast capture path

**Deliverables:**
- Complete `src/capture.rs` file
- Can test by saving captured image

---

### Step A.5 — ocr.rs (Priority: HIGH)
**Goal:** OCR text detection optimized for "GOAL"

**Tasks:**
- Initialize `leptess::LepTess` once (reuse instance)
- Implement preprocessing: RGBA → grayscale → binary threshold
- Configure OCR: `PSM_SINGLE_WORD`, whitelist "GOAL"
- Uppercase normalization
- Return `bool` (detected or not)
- Reuse buffers - no allocations in hot path

**Key Requirements:**
- Single allocation reuse
- Fast preprocessing
- Optimized for single word

**Deliverables:**
- Complete `src/ocr.rs` file
- Can test with sample images

---

### Step A.6 — utils.rs (Priority: MEDIUM)
**Goal:** Shared utilities and types

**Tasks:**
- Atomic flags: `Arc<AtomicBool>` for `is_running`, `is_paused`
- Shared types and helper functions
- Error types if needed

**Deliverables:**
- Complete `src/utils.rs` file

---

### Step A.7 — main.rs (Priority: HIGH)
**Goal:** Wire everything together into working application

**Tasks:**
- Load config
- Initialize all managers (capture, OCR, audio)
- Setup global hotkeys: F8/CMD+1 (pause/resume), F9/CMD+2 (quit)
- Implement main loop:
  - Check if paused (sleep 250ms if paused)
  - Capture → Preprocess → OCR → Trigger audio
  - Target loop time: ≤ 100ms when active
- Handle graceful shutdown

**Key Requirements:**
- Allocation-free inner loop after warm-up
- Proper error handling
- Clean shutdown

**Deliverables:**
- Complete `src/main.rs` file
- Runnable with `cargo build --release`
- Working end-to-end pipeline

---

### Step B — Latency Instrumentation (Priority: HIGH)
**Goal:** Measure actual performance and identify bottlenecks

**Tasks:**
1. Add lightweight timing using `instant::Instant`
2. Instrument key stages:
   - Capture time
   - Preprocessing time
   - OCR time
   - Audio trigger time
3. Implement `--bench` CLI flag
   - Run 500 iterations (configurable via config)
   - Calculate p50 and p95 latencies
   - Print results table
4. Add timing collection without overhead in normal mode

**Deliverables:**
- Diffs for modified files
- CLI flag wiring in main.rs
- Benchmark output format

**Success Criteria:**
- p95 latency < 100ms
- Clear identification of slowest stage

---

### Step C — False-Positive Control (Priority: MEDIUM)
**Goal:** Reduce false triggers and improve detection reliability

**Tasks:**
1. **Configurable Binary Threshold**
   - Already in config, ensure it's applied correctly
   - Make it tunable via config.json

2. **Morphological Open (Optional)**
   - Add behind `enable_morph_open` flag
   - Implement noise reduction
   - Guard with performance check

3. **Debounce Logic**
   - Implement cooldown period (configurable via `debounce_ms`)
   - Ignore repeat triggers within N milliseconds
   - Track last trigger timestamp

**Deliverables:**
- config.json schema update (already defined)
- Code diffs for ocr.rs and main.rs
- Documentation of threshold tuning

**Success Criteria:**
- No duplicate triggers within debounce window
- Configurable sensitivity

---

### Step D — OS-Specific Optimizations (Priority: MEDIUM)
**Goal:** Maximize performance on macOS and Windows

**Tasks:**
1. **macOS Optimizations**
   - Use highest refresh rate path in scap
   - Document Screen Recording permission requirement
   - Add Info.plist notes for permission string
   - Verify capture API performance

2. **Windows Optimizations**
   - Ensure using `Windows.Graphics.Capture` API
   - Verify GPU-assisted capture path
   - Test on Windows target

3. **Conditional Compilation**
   - Add `#[cfg(target_os = "macos")]` guards
   - Add `#[cfg(target_os = "windows")]` guards
   - Platform-specific fast paths

**Deliverables:**
- Code comments explaining platform differences
- Conditional compilation guards
- Permission documentation

**Constraints:**
- No additional crates
- Use existing scap capabilities

---

### Step E — Test Hooks & TDD (Priority: LOW)
**Goal:** Enable unit testing and regression prevention

**Tasks:**
1. **Refactor for Testability**
   - Extract `fn detect_goal(img: GrayImage) -> bool`
   - Make it unit-testable (no global state)

2. **Test Image Set**
   - Create test images with "GOAL" text
   - Create test images without "GOAL"
   - Various fonts, sizes, backgrounds

3. **TDD Approach**
   - Write 2 failing tests first:
     - Test 1: Should detect "GOAL" in clear image
     - Test 2: Should not detect in noise/empty image
   - Implement fixes to make tests pass

4. **Test Suite**
   - Unit tests for detect_goal
   - Integration test for full pipeline (optional)
   - Benchmark tests

**Deliverables:**
- Test module in ocr.rs or tests/ directory
- Test images in tests/fixtures/
- Passing test suite

---

## Quality Gates

### Performance Requirements
- [ ] End-to-end latency p95 < 100ms
- [ ] Active loop cadence ≤ 100ms
- [ ] Paused loop cadence ~250ms
- [ ] No allocations in hot path after warm-up

### Code Quality
- [ ] No unsafe code (unless justified with rationale)
- [ ] All unit tests passing
- [ ] Self-review checklist completed for each change:
  - Race condition risks?
  - Potential panics?
  - Unnecessary allocations?
  - Error handling complete?

### Documentation
- [ ] Config schema documented
- [ ] Platform permissions documented
- [ ] Build and run instructions clear
- [ ] Feature flags explained (one line each)

---

## Build & Run Commands

```bash
# Build (release mode)
cargo build --release

# Run (normal mode)
./target/release/fm-goal-musics

# Run (benchmark mode)
./target/release/fm-goal-musics --bench

# Run tests
cargo test

# Run with logging (if added)
RUST_LOG=info ./target/release/fm-goal-musics
```

---

## Configuration

**Default Config Location:**
- macOS: `~/Library/Application Support/fm-goal-musics/config.json`
- Windows: `%APPDATA%\fm-goal-musics\config.json`

**Config Schema:**
```json
{
  "capture_region": [0, 0, 200, 100],
  "audio_file_path": "goal.mp3",
  "ocr_threshold": 150,
  "debounce_ms": 800,
  "enable_morph_open": false,
  "bench_frames": 500
}
```

---

## Performance Optimization Checklist

### Capture Stage
- [ ] Reuse Capturer instance
- [ ] Minimize region size
- [ ] GPU-assisted capture enabled

### Preprocessing Stage
- [ ] Single allocation reuse
- [ ] Grayscale conversion optimized
- [ ] Binary threshold only (no expensive filters)
- [ ] Morphological operations behind flag

### OCR Stage
- [ ] Reuse LepTess instance
- [ ] PSM_SINGLE_WORD mode
- [ ] Whitelist "GOAL" only
- [ ] Uppercase normalization

### Audio Stage
- [ ] Preload into memory at init
- [ ] Persistent OutputStream
- [ ] Non-blocking play_sound()
- [ ] Warm decoder at startup

### Threading
- [ ] Single producer (capture/OCR)
- [ ] Lightweight notifier to audio sink
- [ ] Minimal synchronization overhead

---

## Iteration Protocol

1. **After each step:**
   - Run benchmarks
   - Report p50/p95 latency table
   - Identify bottlenecks

2. **If p95 > 100ms:**
   - Propose ONE change at a time
   - Re-benchmark after each change
   - Document impact

3. **Progress to next step:**
   - Wait for "Next" command
   - Ensure current step quality gates pass

---

## Security & Platform Notes

### macOS
- **Screen Recording Permission Required**
  - App must request permission on first run
  - User must grant in System Preferences > Security & Privacy > Privacy > Screen Recording
  - Add to Info.plist: `NSScreenCaptureUsageDescription`

### Windows
- **No special permissions required**
- Uses Windows.Graphics.Capture API (Windows 10+)

### Code Safety
- Avoid unsafe code unless performance-critical
- Document any unsafe blocks with rationale
- Use safe abstractions where possible

---

## Known Constraints

### Hard Constraints
- Keep inner loop allocation-free after warm-up
- Initialize Capturer and LepTess once, reuse forever
- Audio must be in RAM, no disk streaming
- End-to-end loop ≤ 100ms (active), ~250ms (paused)

### Don'ts
- Don't modify unrelated modules
- Don't add heavyweight logging (use counters/timers)
- Don't refactor interfaces unless prompted
- Don't create additional binaries/GUIs/services
- Don't add new crates unless explicitly requested

### File Boundaries
**Allowed to modify:**
- `Cargo.toml`
- `src/main.rs`
- `src/capture.rs`
- `src/ocr.rs`
- `src/audio.rs`
- `src/config.rs`
- `src/utils.rs`
- `config.json`

**Not allowed to modify:**
- Any other files

---

## Current Status

- [x] Documentation created
- [x] Plan defined
- [ ] Step A - Baseline Core
- [ ] Step B - Latency Instrumentation
- [ ] Step C - False-Positive Control
- [ ] Step D - OS-Specific Optimizations
- [ ] Step E - Test Hooks & TDD

**Next Action:** Begin Step A - Implement baseline core functionality
