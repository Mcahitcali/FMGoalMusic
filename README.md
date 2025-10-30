# FM Goal Musics 🎯⚽🎵

A high-performance goal detection system for Football Manager that plays celebration music when "GOAL FOR" text appears on screen.

## Features

- **Real-time Goal Detection**: Monitors screen for "GOAL FOR" text using OCR
- **Instant Audio Playback**: Preloaded MP3 in memory for zero-latency playback
- **GPU-Accelerated Capture**: Uses scap for efficient screen capture
- **Configurable**: JSON config for capture region, audio file, thresholds
- **User-Friendly GUI**: Graphical interface with music management and visual controls
- **Keyboard Controls**: Cmd+Shift+R for region selection, Cmd+1 to pause/resume

## Installation

### Prerequisites

**macOS:**
```bash
brew install tesseract
```

**Requirements:**
- macOS 10.13+ (High Sierra or later)
- Screen Recording permission (will be requested on first run)

**Linux:**
```bash
sudo apt-get install tesseract-ocr
```

**Requirements:**
- X11 or Wayland compositor
- No special permissions required

**Windows:**
- Windows 10 version 1903 or later (for Windows.Graphics.Capture API)
- Tesseract OCR (download from [GitHub](https://github.com/UB-Mannheim/tesseract/wiki))
- No special permissions required

## 🚀 Quick Start

### For macOS Development:
```bash
./build_macos.sh
```
Creates: `build/macos/FM Goal Musics.app` and `.dmg` installer

### For Windows Distribution:
Send users: `FM-Goal-Musics-Windows-Source-*.zip`
Users: Right-click `INSTALL-WINDOWS.ps1` → "Run with PowerShell"

That's it! 🎉

## Configuration

On first run, a default config is created at:
- **macOS/Linux:** `./target/release/config/config.json` (next to executable)

### Config Structure

```json
{
  "capture_region": [400, 900, 1024, 50],
  "audio_file_path": "goal.mp3",
  "ocr_threshold": 0,
  "debounce_ms": 8000,
  "enable_morph_open": false,
  "bench_frames": 500
}
```

**Parameters:**
- `capture_region`: `[x, y, width, height]` - Screen region to monitor
- `audio_file_path`: Path to MP3 file (relative to config directory)
- `ocr_threshold`: Binary threshold for OCR (0 = automatic Otsu, 1-255 = manual)
- `debounce_ms`: Minimum time between goal detections (8000ms = 8 seconds recommended)
- `enable_morph_open`: Enable morphological opening for noise reduction (may impact performance)
- `bench_frames`: Number of frames for benchmark mode

### Setup Audio

Place your goal celebration MP3 at:
```
./target/release/config/goal.mp3
```

### Finding the Capture Region

**For Football Manager on your screen:**

1. Play Football Manager
2. When a goal appears, take a screenshot (Cmd+Shift+3)
3. Open the screenshot and note where "GOAL FOR" text appears
4. Update `capture_region` in config.json with those coordinates

**Example positions:**
- Bottom center (1920x1080): `[0, 980, 1920, 100]`
- Bottom area (3024x1898): `[400, 900, 1024, 50]`

## Usage

```bash
./target/release/fm-goal-musics-gui
```

Or run the macOS app:
```bash
open "target/release/FM Goal Musics.app"
```

**Features:**
- 🎵 Add and manage multiple music files
- 🎮 Start/Stop/Pause detection with buttons
- ⚙️ Visual configuration editor
- 📊 Real-time status and detection counter
- 💾 Easy music file selection
- 🔲 Region selector (Cmd+Shift+R)
- ⚡ Team-specific goal detection

**Controls:**
- **Cmd+1**: Toggle detection on/off
- **Cmd+Shift+R**: Open region selector

## Performance

**Target:** <100ms total response time

Typical performance on modern hardware:
- **Capture:** 5-15ms (macOS), 10-20ms (Windows), varies (Linux)
- **OCR:** 10-20ms
- **Total:** 30-65ms
- **FPS:** 60 FPS
- **Response:** <100ms from goal appearing to sound playing

### Platform-Specific Performance

**macOS:**
- Uses Metal framework for GPU-accelerated capture
- Excellent performance on both Intel and Apple Silicon
- Respects Retina display scaling automatically
- Typical capture latency: 5-15ms

**Windows:**
- Uses Windows.Graphics.Capture API (DirectX-backed)
- GPU-accelerated capture on Windows 10+
- Good performance on modern GPUs
- Typical capture latency: 10-20ms

**Linux:**
- Uses X11 or Wayland capture
- Performance varies by compositor and GPU
- May have higher latency than macOS/Windows
- Typical capture latency: 15-30ms

## Tuning False-Positive Controls

### Debounce Time
If you're getting multiple triggers for the same goal:
- Increase `debounce_ms` in config.json (default: 8000ms = 8 seconds)
- This prevents the system from triggering again within the specified time

### OCR Threshold
If detection is unreliable:
- **Automatic (recommended)**: Set `ocr_threshold: 0` - uses Otsu's method
- **Manual tuning**: Set to 1-255 to override automatic threshold
  - Lower values (80-120): Better for light text on dark background
  - Higher values (150-200): Better for dark text on light background
  - Run `--test` mode to see what text is detected and adjust

### Morphological Opening
If you're getting false positives from noise/artifacts:
- Set `enable_morph_open: true` in config.json
- This removes small noise while preserving text
- **Warning**: May add 5-10ms latency - benchmark before/after

## Troubleshooting

### macOS: "Permission denied" or Screen Recording Issues

**Grant Screen Recording permission:**
1. Open System Preferences > Security & Privacy
2. Click Privacy tab > Screen Recording
3. Click the lock icon to make changes
4. Add Terminal (or your terminal app like iTerm2, Warp, etc.)
5. Check the box next to the terminal app
6. Restart the terminal application
7. Run the app again

**If permission is granted but capture still fails:**
- Ensure you're running macOS 10.13 (High Sierra) or later
- Try logging out and back in
- Check if any screen recording software is already running
- Verify the capture region is within screen bounds

### Windows: Capture Fails or Black Screen

**Requirements:**
- Windows 10 version 1903 or later
- Graphics drivers up to date

**If capture fails:**
- Update Windows to the latest version
- Update graphics drivers (NVIDIA/AMD/Intel)
- Disable any screen overlay software (Discord, OBS, etc.)
- Run as administrator if needed
- Check Windows Graphics settings for the app

### Linux: Capture Issues

**X11:**
- Ensure X11 is running (not Wayland)
- Check compositor settings
- May need to disable compositing for best performance

**Wayland:**
- Support varies by compositor
- May need additional permissions
- Consider switching to X11 for better compatibility

### No Goals Detected

1. Check `capture_region` in config.json
2. Make sure Football Manager is in the capture area
3. Test with: `cargo run --release -- --test`
4. The app looks for exact "GOAL FOR" text

### Audio Not Playing

1. Verify `goal.mp3` exists in config directory
2. Check file format (must be valid MP3)
3. Test with: `cargo run --release -- --test`

### High CPU Usage

The app runs at 60 FPS for fast response. If needed:
- Increase debounce time to reduce false triggers
- Use smaller capture region

## Detection Details

### What It Detects

✅ **Will detect:**
- "GOAL FOR F.C. INTERNAZIONALE MILANO"
- "GOAL FOR MANCHESTER UNITED"
- "GOAL FOR BARCELONA"
- Any "GOAL FOR [TEAM NAME]" text

❌ **Won't detect:**
- "GOALKEEPER"
- "GOALS" (plural)
- "FMGOALMUSIC"
- Standalone "GOAL"

### Color Handling

Uses adaptive Otsu thresholding:
- Automatically adjusts to any color combination
- Works with white text on dark backgrounds
- Works with colored text (cyan, yellow, etc.)
- Auto-inverts for optimal OCR (black text on white)

## Project Structure

```
src/
├── gui_main.rs        # GUI entry point
├── gui.rs             # GUI implementation
├── config.rs          # Configuration management
├── audio.rs           # Audio preloading and playback
├── audio_converter.rs # Audio format conversion
├── capture.rs         # Screen capture with scap
├── ocr.rs             # OCR with adaptive thresholding
├── region_selector.rs # Screen region selector
├── teams.rs           # Team database
├── team_matcher.rs    # Team name matching
└── utils.rs           # Shared utilities, timing, debouncing
```

## Development

### Build and Test

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run GUI in development mode
cargo run --release

# Run tests
cargo test
```

### Testing

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_test

# Run specific test
cargo test test_ocr_manager_creation

# Run with output
cargo test -- --nocapture
```

**Test Coverage:**
- 37 total tests
- Unit tests for OCR, utils, audio, capture, config
- Integration tests for pipeline validation
- Regression prevention for critical paths

### Code Quality

```bash
# Check code
cargo check

# Lint
cargo clippy

# Format
cargo fmt
```

## Technical Details

### Architecture

- **Thread-safe**: Uses Arc<AtomicBool> for state management
- **Zero-allocation loop**: Reuses buffers after initialization
- **Adaptive OCR**: Otsu's method for optimal thresholding
- **GPU-accelerated capture**: Platform-specific optimizations
- **Preloaded audio**: rodio loads MP3 into memory

### Platform-Specific Implementation

**macOS:**
- **Capture**: Metal framework via `scap` (IOSurface-backed)
- **GPU**: Metal acceleration for screen capture
- **Permissions**: Screen Recording (Privacy & Security)
- **Display**: Automatic Retina scaling support
- **Performance**: Excellent (5-15ms capture latency)

**Windows:**
- **Capture**: Windows.Graphics.Capture API via `scap`
- **GPU**: DirectX-backed capture (DXGI)
- **Permissions**: None required
- **Display**: Multi-monitor support
- **Performance**: Good (10-20ms capture latency)
- **Requirements**: Windows 10 1903+

**Linux:**
- **Capture**: X11/Wayland via `scap`
- **GPU**: Varies by compositor
- **Permissions**: None typically required
- **Display**: X11 or Wayland
- **Performance**: Varies (15-30ms typical)

### Performance Optimizations

- 60 FPS capture target
- Adaptive threshold calculation
- Reuse of image buffers
- Minimal temporary file usage
- Efficient debouncing
- Platform-native capture APIs
- GPU-accelerated screen capture
- Crop area applied at capture time (no post-processing)

## License

MIT

## Credits

Built with:
- [scap](https://github.com/CapSoftware/scap) - Screen capture
- [leptess](https://github.com/houqp/leptess) - Tesseract OCR wrapper
- [rodio](https://github.com/RustAudio/rodio) - Audio playback
- [rdev](https://github.com/Narsil/rdev) - Global keyboard hooks
- [Tesseract OCR](https://github.com/tesseract-ocr/tesseract) - Text recognition