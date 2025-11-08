# Project Context

## Purpose
FM Goal Musics is a cross-platform desktop companion for Football Manager that watches a tight capture region on the user’s screen, recognizes “GOAL FOR” (and future phrases) via OCR, and instantly plays personalized celebration audio. It focuses on sub-100 ms reaction time, zero-click operation once configured, and a friendly GUI that lets players manage music libraries, select detection regions, and tune thresholds without editing files by hand.

## Tech Stack
- Rust 2021 (1.75+) for the entire application, with `cargo` as the build system and `rustfmt`/`clippy` as required tooling.
- `zed/gpui` + `gpui-component` for the GPU-accelerated GUI, `gpui_component::theme` for ActiveTheme tokens, `rfd` for native dialogs, and `rdev` for global hotkeys (Cmd+1, Cmd+Shift+R).
- `xcap` (scap successor) plus native APIs (Metal on macOS, Windows.Graphics.Capture, X11/Wayland) for high-FPS, low-copy screen capture.
- `leptess` + Tesseract OCR with optional morphological preprocessing for text recognition; custom phrase lists live in `assets/i18n`.
- `rodio` + `symphonia` + `hound` for audio decoding, mixing, and playback with MP3 preloading to avoid runtime I/O.
- `crossbeam-channel`, `parking_lot`, `tracing`, `anyhow`, and `thiserror` for concurrency, structured logging, and ergonomic error handling.

## Project Conventions

### Code Style
- Run `cargo fmt`, `cargo clippy --all-targets --all-features`, and `cargo test` before every commit or PR; CI assumes code is formatted and lint-clean.
- Modules follow Rust defaults (`snake_case` files, `CamelCase` types, `SCREAMING_SNAKE_CASE` consts). Keep functions focused (<50 LOC) and prefer splitting logic into modules listed in `ARCHITECTURE.md`.
- GPUI views in `src/gui/view.rs` stay declarative (RenderOnce components); impure work happens in controllers/executors. Use `anyhow::Context` when crossing subsystem boundaries.
- Favor `parking_lot::{Mutex,RwLock}` for shared state, `Arc<AtomicBool>` for hot paths, and avoid blocking the GPUI UI thread.

### Architecture Patterns
- GUI follows GPUI’s component/event model: `state.rs` tracks tabs and settings, `controller.rs` bridges detectors/config, and `view.rs` renders component trees via gpui-component.
- Event/Command segregation (see `messaging/`) isolates side effects: detectors emit events, the command executor performs work, and events are turned back into GUI messages.
- Detection uses a strategy/pipeline pattern so new detectors (goal, kickoff, match end) can be plugged in without touching the core loop.
- Background workers (capture/OCR/audio) run on dedicated threads coordinated through `crossbeam-channel`; no async runtime is used to keep behavior deterministic.

### Testing Strategy
- `cargo test` must stay green; unit tests cover OCR preprocessing, detectors, audio utilities, config parsing, and helpers in `utils.rs`.
- Integration tests exercise the detection pipeline, event bus, and configuration repository; use mocked frames/audio to avoid platform requirements.
- Manual smoke tests are required per platform (macOS app bundle, Windows PowerShell installer, Linux binary). We target <100 ms detection end-to-end and verify with bench frames.
- Before releases, run `cargo check`, `cargo fmt`, `cargo clippy`, `cargo test`, and platform-specific build scripts (`./build_macos.sh`, Windows PowerShell installer).

### Git Workflow
- Default branch is `main`; every change starts from a feature branch (e.g., `feature/match-end-detector`). No direct commits to `main`.
- Feature PR checklist (from `Doc/Plan.md`): update docs, add/adjust tests, run full test suite + benchmarks, then request review. Squash or merge after approval.
- Tag releases after merging to `main` (e.g., `v0.2.4`), generate platform artifacts (`.app`, `.dmg`, Windows zip), and attach them to the GitHub release.
- Commit messages are imperative (“Add kickoff detector”). Avoid rewriting shared history; prefer follow-up commits if fixes are needed.

## Domain Context
- Target users are Football Manager players who stream or play in fullscreen; detection watches for localized phrases (“GOAL FOR Team”) with optional team-specific logic.
- OCR accuracy depends heavily on capture region coordinates; region selection tooling (Cmd+Shift+R) must stay precise and DPI-aware.
- Configuration lives beside the executable (`target/release/config/config.json` by default). Audio assets sit next to the config folder to keep portable installs simple.
- Multi-language phrase files (`assets/i18n/*.json`) drive detection; teams database (`assets/teams/teams.json`) powers team matching and scoreboard handling.
- Hotkeys (Cmd+1, etc.) must remain global yet respectful of OS security prompts (screen recording permissions on macOS, Tesseract install paths on Windows/Linux).

## Important Constraints
- Detection must stay under ~100 ms end-to-end to feel instant; this drives zero-allocation loops, buffer reuse, and preloaded audio.
- App must operate fully offline after dependencies (Tesseract, assets) are installed; no network is required except optional update checks via `ureq`.
- macOS builds require screen-recording entitlements, notarization-ready bundles, and minimal dependencies (Metal/objc only). Windows builds target 1903+ with Graphics.Capture.
- Memory footprint should remain low enough for streamers (avoid >1 GB). Avoid long blocking operations on the GPUI main thread to keep rendering responsive at 60 FPS.
- Config and assets are user-editable; never overwrite user files without explicit action and always debounce disk writes to avoid SSD churn.

## External Dependencies
- **Tesseract OCR** (`leptess`, `tessdata/`) – Required runtime dependency; users must install the engine and language packs locally.
- **xcap / platform capture APIs** – GPU-backed capture surfaces (Metal, DXGI, X11/Wayland) that deliver the high-FPS frames fed into the OCR pipeline.
- **rodio + symphonia + hound** – Audio decoding and playback stack for MP3/AAC/FLAC/WAV assets with in-memory caching.
- **zed/gpui + gpui-component + rfd + rdev** – GUI runtime, component library, native dialogs, and global hotkeys; rely on OS accessibility permissions.
- **tracing + tracing-appender** – Structured logging to rotating files in the user config directory for support/debugging.
- **ureq + semver + open** – Lightweight HTTP client for release checks plus helpers to open release notes in the default browser (optional but supported).
