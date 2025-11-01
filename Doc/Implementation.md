### 9. Slug Module (`slug.rs`)
**Responsibility:** Generate ASCII-only slugs for filenames

```rust
pub fn slugify(input: &str) -> String
// Rules:
// - Turkish unique letters mapped: ı→i, İ→I, ğ→g, Ğ→G, ş→s, Ş→S
// - General diacritics removed via Unicode NFD
// - Spaces and non-alphanumerics → underscores; collapsed; trimmed
```

**Usage Points:**
- `audio_converter.rs` when naming output WAVs
- Display names in GUI derived from file stem (no extension)

### 10. Team Database Module (`teams.rs`) 🔄
**Responsibility:** Load and query team database

```rust
#[derive(Deserialize)]
pub struct Team {
    pub display_name: String,
    pub variations: Vec<String>,
}

#[derive(Deserialize)]
pub struct TeamDatabase {
    #[serde(flatten)]
    leagues: HashMap<String, HashMap<String, Team>>, // league -> team_key -> Team
}

impl TeamDatabase {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>>
    pub fn get_leagues(&self) -> Vec<String>
    pub fn get_teams(&self, league: &str) -> Option<Vec<(String, Team)>>
    pub fn find_team(&self, league: &str, team_key: &str) -> Option<Team>
    pub fn search_team(&self, query: &str) -> Vec<(String, String, Team)>
}
```

**Features:**
- Load from `config/teams.json` with embedded fallback if file missing
- Query leagues and teams
- Team lookup by key
- Case-insensitive search by display name or variation

### 11. Team Matcher Module (`team_matcher.rs`) 🔄
**Responsibility:** Match detected team names against selected team

```rust
pub struct TeamMatcher {
    normalized_variations: Vec<String>,
}

impl TeamMatcher {
    pub fn new(team: &Team) -> Self
    pub fn matches(&self, detected_name: &str) -> bool
    fn normalize(text: &str) -> String // ASCII-only, lowercased, collapse spaces
}
```

**Matching Strategy:**
- Exact normalized equality with any variation
- Token-subset match: all tokens in a variation must appear in detected tokens
- ASCII-only normalization to avoid diacritic/Unicode noise
- Case-insensitive, special characters stripped, whitespace normalized
- Performance: ~1000 checks < 10ms on typical hardware
# FM Goal Musics – Technical Implementation

## Technology Stack Overview

### Programming Language
**Rust** (Edition 2021)
- Systems programming language for performance and memory safety
- Zero-cost abstractions
- Guaranteed memory safety without garbage collection
- Cross-platform compilation target support
- Strong type system and excellent error handling

## Core Technologies

### Screen Capture
**Library:** `xcap` v0.7.x
- Cross-platform GPU-accelerated screen capture
- Platform-specific implementations:
  - **macOS:** ScreenCaptureKit (macOS 12.3+) or Core Graphics (older versions)
  - **Windows:** Windows.Graphics.Capture API (DirectX)
  - **Linux:** X11 or Wayland (via pipewire/portals)
- Full screen capture with efficient region cropping
- RGBA output format
- Performance: 10-20ms latency on modern systems
- Stable release, actively maintained (2024-2025)

### Optical Character Recognition (OCR)
**Library:** `leptess` v0.14.0 (Tesseract wrapper)
- Tesseract OCR engine integration
- Configuration:
  - Page Segmentation Mode: `PSM_AUTO (3)` – fully automatic segmentation (no OSD)
  - No whitelist (allows full team text recognition)
  - Uppercase normalization applied post-OCR for matching
- Preprocessing pipeline:
  - RGBA → Grayscale conversion
  - Binary thresholding (auto Otsu or manual)
  - Optional morphological opening
- Performance: 10-20ms typical OCR time

**External Dependency:**
- Tesseract OCR (v4.0+)
- Installation: `brew install tesseract` (macOS), `apt-get install tesseract-ocr` (Linux)

### Audio Playback
**Libraries:**
- `rodio` v0.19.0 - Audio playback engine
- `symphonia` v0.5 - Multi-format decoder (MP3, FLAC, OGG, AAC, WAV)
- `hound` v3.x - WAV encoder for conversion

**Architecture:**
- Preload audio files into memory at startup
- Persistent `OutputStream` and `Sink` for zero-latency playback
- Non-blocking trigger mechanism
- WAV format for optimal playback performance

**Supported Formats:**
- Input: MP3, AAC, FLAC, OGG Vorbis, WAV
- Internal: 16-bit PCM WAV
- Automatic conversion on file import
- Managed output location: `config/musics/<ascii_slug>.wav`

### GUI Framework
**Libraries:**
- `egui` v0.29.1 - Immediate mode GUI framework
- `eframe` v0.29.1 - Application framework for egui
- `rfd` v0.15.1 - Native file dialogs (cross-platform)

**Features:**
- 60 FPS rendering
- Minimal memory footprint
- Platform-native look and feel
- Hot-reload friendly (development)

### Image Processing
**Library:** `image` v0.25.5
- Image format handling (PNG, JPEG, BMP, etc.)
- Pixel manipulation and color space conversion
- Buffer management for screen captures
- Grayscale and binary threshold operations

### Configuration Management
**Libraries:**
- `serde` v1.0 - Serialization framework
- `serde_json` v1.0 - JSON serialization
- `dirs` v5.0 - Platform-specific directory paths
- `unicode-normalization` v0.1 - Slug generation (diacritic removal)

**Storage Locations:**
- macOS: `~/Library/Application Support/fm-goal-musics/config.json`
- Windows: `%APPDATA%\fm-goal-musics\config.json`
- Linux: `~/.config/fm-goal-musics/config.json`

### Keyboard Controls
**Library:** `rdev` v0.5.4
- Global keyboard hook registration
- Cross-platform hotkey support
- Event-driven architecture
- Controls: Cmd+1 (pause/resume), Ctrl+C (quit)

## Architecture Design

### Application Structure
```
fm-goal-musics/
├── src/
│   ├── gui_main.rs          # GUI entry point
│   ├── gui.rs               # GUI implementation
│   ├── capture.rs           # Screen capture manager
│   ├── ocr.rs               # OCR text detection
│   ├── audio.rs             # Audio playback manager
│   ├── audio_converter.rs   # Format conversion
│   ├── config.rs            # Configuration management
│   ├── region_selector.rs   # Visual region picker
│   ├── slug.rs              # ASCII slug generation for filenames
│   ├── teams.rs             # Team database loader
│   ├── team_matcher.rs      # Team name matching logic
│   └── utils.rs             # Timing, debounce, shared utilities
├── tests/
│   ├── integration_tests.rs # Integration test suite
│   └── fixtures/            # Test images and data
├── Doc/
│   ├── Plan.md              # Project plan
│   ├── Design.md            # Design specifications
│   ├── Implementation.md    # Technical documentation (this file)
│   └── Features/            # Feature-specific documentation
├── Cargo.toml               # Rust dependencies and build config
├── build_app.sh             # macOS app bundle builder
└── README.md                # User documentation
```

## Module Architecture

### Core Detection Pipeline
```
┌─────────────────────────────────────────────────────────────┐
│                    Detection Loop (main.rs)                 │
│                                                             │
│  ┌──────────┐    ┌────────────┐    ┌─────┐    ┌────────┐  │
│  │ Capture  │ -> │ Preprocess │ -> │ OCR │ -> │ Audio  │  │
│  │ Manager  │    │  Pipeline  │    │ Mgr │    │ Trigger│  │
│  └──────────┘    └────────────┘    └─────┘    └────────┘  │
│       │                │                │           │       │
│       └────────────────┴────────────────┴───────────┘       │
│                         │                                   │
│                    Config & State                           │
│                   (Arc<AtomicBool>)                        │
└─────────────────────────────────────────────────────────────┘
```

### 1. Capture Manager (`capture.rs`)
**Responsibility:** Screen region capture with GPU acceleration

```rust
pub struct CaptureManager {
    monitor: xcap::Monitor,
    region: CaptureRegion,
}

impl CaptureManager {
    pub fn new(region: CaptureRegion) -> Result<Self>
    pub fn capture_region(&mut self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>>
}
```

**Key Features:**
- Monitor instance reused across captures for optimal performance
- Full screen capture with efficient region cropping
- Cross-platform compatibility (Windows, macOS, Linux)
- Error handling with helpful validation messages
- Platform-specific GPU acceleration

### 2. OCR Manager (`ocr.rs`) 🔄
**Responsibility:** Text detection and preprocessing

```rust
pub struct OcrManager {
    tess: LepTess,
    manual_threshold: Option<u8>,
    enable_morph_open: bool,
}

impl OcrManager {
    pub fn new_with_options(threshold: u8, morph: bool) -> Result<Self>
    pub fn detect_goal(&mut self, img: &RgbaImage) -> Result<bool>
    pub fn detect_goal_with_team(&mut self, img: &RgbaImage) -> Result<Option<String>> // 🔄 New
    fn preprocess(&self, img: &RgbaImage) -> GrayImage
    fn morphological_opening(&self, img: &GrayImage) -> GrayImage
}
```

**Pipeline:**
1. RGBA → Grayscale conversion
2. Binary thresholding (auto Otsu or manual)
3. Optional morphological opening (noise reduction)
4. OCR text extraction
5. "GOAL" keyword detection
6. **Team name extraction (🔄 New)**: Parse "GOAL FOR [team_name]" pattern

### 3. Audio Manager (`audio.rs`)
**Responsibility:** Audio preloading and playback with volume control

```rust
pub struct AudioManager {
    _stream: OutputStream,
    sink: Sink,
    audio_data: Arc<Vec<u8>>,
    volume: Mutex<f32>,
}

impl AudioManager {
    pub fn new(audio_path: &str) -> Result<Self>
    pub fn play_sound(&self)
    pub fn set_volume(&self, volume: f32) // 0.0 to 1.0
    pub fn get_volume(&self) -> f32
}
```

**Key Features:**
- Preload audio into memory at initialization
- Persistent output stream (no setup latency)
- Non-blocking playback trigger
- Volume control (0.0 to 1.0 range)
- Thread-safe volume adjustment
- Warm decoder at startup

### 4. Audio Converter (`audio_converter.rs`)
**Responsibility:** Multi-format audio conversion to WAV

```rust
pub fn convert_to_wav(input_path: &Path) -> Result<PathBuf>
// Saves to config/musics/<ascii_slug>.wav and returns final path
```

**Conversion Process:**
1. Detect input format (MP3, FLAC, OGG, AAC, WAV)
2. Decode using Symphonia
3. Extract PCM samples
4. Encode to 16-bit PCM WAV using Hound
5. Preserve channel configuration (mono/stereo)
6. Filename slugging: ASCII-only name with underscores; stored under `config/musics/`

### 5. Configuration (`config.rs`) 🔄
**Responsibility:** Configuration persistence and validation

```rust
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub capture_region: (u32, u32, u32, u32),
    pub ocr_threshold: u8,
    pub debounce_ms: u64,
    pub enable_morph_open: bool,
    pub bench_frames: usize,
    pub music_list: Vec<MusicEntry>,
    pub selected_music_index: Option<usize>,
    pub selected_team: Option<SelectedTeam>, // 🔄
    pub music_volume: f32, // 🔄 New - Music volume (0.0-1.0, default 1.0)
    pub ambiance_volume: f32, // 🔄 New - Ambiance volume (0.0-1.0, default 0.6)
    pub goal_ambiance_path: Option<String>, // 🔄 New - Goal crowd cheer sound path
}

#[derive(Serialize, Deserialize)] // 🔄
pub struct SelectedTeam {
    pub league: String,
    pub team_key: String,
    pub display_name: String,
}

impl Config {
    pub fn load() -> Result<Self>
    pub fn save(&self) -> Result<()>
    pub fn default() -> Self
}
```

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
  "selected_music_index": null,
  "selected_team": {
    "league": "Premier League",
    "team_key": "manchester_united",
    "display_name": "Manchester Utd"
  },
  "music_volume": 1.0,
  "ambiance_volume": 0.6,
  "goal_ambiance_path": "config/sounds/goal_crowd_cheer.wav"
}
```

### 6. Utilities (`utils.rs`)
**Responsibility:** Shared utilities and timing

```rust
pub struct Timer { /* ... */ }
pub struct Debouncer { /* ... */ }
pub struct IterationTiming { /* ... */ }
pub struct LatencyStats { /* ... */ }

impl Timer {
    pub fn elapsed_us(&self) -> u64
    pub fn elapsed_ms(&self) -> u64
}

impl Debouncer {
    pub fn new(cooldown_ms: u64) -> Self
    pub fn should_trigger(&mut self) -> bool
    pub fn reset(&mut self)
}
```

### 7. GUI Implementation (`gui.rs`, `gui_main.rs`)
**Responsibility:** User interface and state management

```rust
pub struct AppState {
    pub config: Config,
    pub is_running: bool,
    pub is_paused: bool,
    pub detection_count: usize,
    // ... other state
}

impl eframe::App for FmGoalMusicsApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame)
}
```

**Threading Model:**
- **Main Thread:** GUI rendering and event handling
- **Detection Thread:** Background capture/OCR/audio pipeline
- **Communication:** `Arc<Mutex<AppState>>` for shared state

#### Tabbed UI (Updated)
- `enum AppTab { Library, Audio, Detection, Settings, Help }`
- `active_tab: AppTab` manages which section is visible
- Status bar is always visible at the top (state, detections, display/window size)
- Tab contents:
  - **Library:** music list, add/remove
  - **Audio:** volumes, sound length sliders, ambiance enable/file
  - **Detection:** team selection (league/team), controls (start/pause/stop, preview), capture preview
  - **Settings:** capture region, OCR threshold, debounce, morphology
  - **Help:** quick usage tips

### 8. Region Selector (`region_selector.rs`)
**Responsibility:** Visual screen region selection

```rust
pub struct RegionSelector {
    screenshot: Option<RgbaImage>,
    selection: Option<(u32, u32, u32, u32)>,
}

impl eframe::App for RegionSelector {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame)
}
```

**Workflow:**
1. Capture full screen
2. Display in fullscreen overlay
3. Track mouse drag for selection
4. Calculate region coordinates
5. Return selected region

## Data Flow

### GUI Detection Flow
```
1. Launch GUI (Main Thread)
   ↓
2. Load Config & State
   ↓
3. Render UI (60 FPS)
   ├─ Music list management
   ├─ Configuration editing
   └─ Process control buttons
   ↓
4. User clicks "Start"
   ↓
5. Spawn Detection Thread
   ├─ Initialize managers
   ├─ Run detection loop
   ├─ Update shared state (count, status)
   └─ Respect pause/stop signals
   ↓
6. Main Thread updates UI based on shared state
   ↓
7. User clicks "Stop" → Signal detection thread → Join thread
```

## Performance Optimizations

### 1. Allocation-Free Hot Path
- Reuse capturer instance across frames
- Reuse OCR engine instance
- Persistent audio output stream
- Single buffer reuse for image processing

### 2. GPU Acceleration
- Platform-native GPU capture APIs
- Hardware-accelerated frame grabbing
- Minimal CPU involvement in capture

### 3. Optimized OCR
- Single-word page segmentation mode
- Character whitelist ("GOALFOR")
- Small capture region (reduces processing)
- Uppercase normalization only

### 4. Audio Preloading
- Load entire audio file into memory at startup
- Persistent sink (no re-initialization)
- WAV format (no decoding overhead)
- Non-blocking trigger (fire-and-forget)

### 5. Threading Strategy
- Single-threaded detection loop (no synchronization overhead)
- Separate GUI thread (no UI blocking)
- Atomic flags for state (lock-free)

## Build Configuration

### Cargo.toml Dependencies
```toml
[dependencies]
# Screen Capture
xcap = "0.7"

# OCR
leptess = "0.14.0"

# Audio
rodio = { version = "0.19", features = ["mp3"] }
symphonia = { version = "0.5", features = ["mp3", "aac", "flac", "isomp4", "ogg"] }
hound = "3"

# GUI
egui = "0.33.0"
eframe = "0.33.0"
rfd = "0.14"

# Utilities
image = "0.25"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
dirs = "5.0"
rdev = "0.5"
unicode-normalization = "0.1"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
overflow-checks = false
```

### Build Profiles

#### Debug Build
```bash
cargo build
```
- Fast compilation
- Debug symbols included
- No optimizations
- Use for development

#### Release Build
```bash
cargo build --release
```
- Maximum optimization (opt-level = 3)
- Link-time optimization (LTO)
- Single codegen unit
- Stripped symbols
- Panic = abort (smaller binary)

### Platform-Specific Builds

#### macOS App Bundle
```bash
./build_app.sh
```
- Creates `FM Goal Musics.app`
- Includes Info.plist with permissions
- Code signing ready
- Located in `target/release/`

#### Windows Executable
```bash
cargo build --release --target x86_64-pc-windows-msvc
```
- MSVC toolchain (recommended)
- Windows subsystem
- No console window (GUI version)

#### Linux Binary
```bash
cargo build --release --target x86_64-unknown-linux-gnu
```
- Dynamic linking to system libraries
- Requires Tesseract OCR installed

## Testing Infrastructure

### Unit Tests
Located within source modules (`#[cfg(test)]` blocks)

**Coverage:**
- `ocr.rs`: Grayscale, threshold, morphology, detection (8 tests)
- `utils.rs`: Timing, debounce, state management (13 tests)
- `audio.rs`: Initialization, error handling (2 tests)
- `capture.rs`: Region management (2 tests)
- `config.rs`: Defaults, serialization (2 tests)

### Integration Tests
Located in `tests/integration_tests.rs`

**Coverage:**
- Full pipeline validation (10 tests)
- Real OCR detection with fixtures
- Config load/save round-trip
- Error handling scenarios

### Test Execution
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_tests
```

### Test Fixtures
Located in `tests/fixtures/`
- Sample images with "GOAL" text
- Various fonts, sizes, backgrounds
- Negative test cases (no text, noise)

## Database & Storage

**Note:** This project does not use a traditional database.

### Storage Approach
- **Configuration:** JSON file in platform-specific app data directory
- **State:** In-memory only (not persisted between sessions)
- **Music List:** Stored in config JSON as file paths
- **Statistics:** Not persisted (future enhancement)

### Config Storage Details
```
Platform: macOS
Location: ~/Library/Application Support/fm-goal-musics/config.json
Format: JSON
Size: < 1 KB

Platform: Windows
Location: %APPDATA%\fm-goal-musics\config.json
Format: JSON
Size: < 1 KB

Platform: Linux
Location: ~/.config/fm-goal-musics/config.json
Format: JSON
Size: < 1 KB
```

### Future Storage Considerations
If enhanced features require persistent storage:
- **Option 1:** SQLite (embedded, zero-config)
- **Option 2:** Additional JSON files (statistics, history)
- **Option 3:** Cloud sync (remote storage, API)

## Deployment

### Distribution Format

#### macOS
- **App Bundle:** `FM Goal Musics.app`
- **Requires:** macOS 10.13+ (High Sierra or later)
- **Permissions:** Screen Recording (requested on first run)
- **Distribution:** Direct download, no code signing (dev)

#### Windows
- **Executable:** `fm-goal-musics-gui.exe`
- **Requires:** Windows 10 version 1903+ (for Graphics.Capture API)
- **Dependencies:** Tesseract OCR (bundled or separate installer)
- **Distribution:** Zip archive or installer

#### Linux
- **Binary:** `fm-goal-musics-gui`
- **Requires:** X11 or Wayland compositor, Tesseract OCR
- **Package:** DEB/RPM (future), AppImage, or direct binary
- **Distribution:** Package manager or direct download

### Installation Requirements

#### All Platforms
1. Download appropriate binary for platform
2. Install Tesseract OCR:
   - macOS: `brew install tesseract`
   - Windows: Download from [UB-Mannheim](https://github.com/UB-Mannheim/tesseract/wiki)
   - Linux: `sudo apt-get install tesseract-ocr`
3. Grant screen recording permission (macOS only)
4. Run application

#### No External Database Required
- Self-contained configuration
- No server components
- No network dependencies
- Local-only operation

## Security Considerations

### Permissions
- **macOS:** Screen Recording permission (sensitive)
- **Windows:** No special permissions required
- **Linux:** No special permissions required

### Data Privacy
- No telemetry or analytics
- No network communication
- All data stored locally
- Config file readable by user only (file permissions)

### Audio File Security
- User-provided audio files
- No validation of audio content
- Files remain in original location
- Converted WAV files created with same permissions

### Screen Capture Security
- Captures only configured region (not full screen)
- Captured data processed in memory only
- No screenshots saved (except debug mode)
- No logging of captured content

## Monitoring & Debugging

### Logging Strategy
Currently minimal logging; focused on:
- Initialization success/failure
- Configuration loading status
- Error conditions
- Performance metrics (benchmark mode)

### Future Logging Enhancement
```rust
// Potential integration with log crate
env_logger::init();
log::info!("Starting detection...");
log::warn!("High latency detected: {}ms", latency);
log::error!("OCR failed: {}", error);
```

### Debug Mode
- GUI provides real-time status and detection count
- Configuration can be edited live in GUI
- Region selector for visual debugging

### Performance Monitoring
- Real-time FPS and timing in GUI
- Detection count tracking
- Visual status indicators

## Error Handling

### Error Types
```rust
// Custom error types (implied from Result usage)
- ConfigError: Config load/save failures
- CaptureError: Screen capture failures
- OcrError: OCR initialization or processing failures
- AudioError: Audio load or playback failures
```

### Error Recovery Strategy
- **Config errors:** Create default config, continue
- **Capture errors:** Show error, guide user to grant permissions
- **OCR errors:** Show error, suggest Tesseract installation
- **Audio errors:** Show error, continue without audio
- **GUI errors:** Show message dialog, don't crash

### User-Facing Errors
All errors presented with:
- Clear description of problem
- Suggested solution or action
- Relevant file paths or settings
- Link to documentation (future)

---

*Last Updated: 2025-10-30*
*Version: 1.0*
