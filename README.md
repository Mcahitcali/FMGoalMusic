# FM Goal Musics 🎯⚽🎵

A high-performance goal detection system for Football Manager that plays celebration music when "GOAL FOR" text appears on screen.

## Features

✅ **Real-time Screen Capture** - GPU-assisted capture with scap  
✅ **Adaptive OCR Detection** - Smart thresholding works with any color scheme  
✅ **Instant Audio Playback** - Preloaded MP3 with minimal latency  
✅ **Debounce Control** - Prevents multiple triggers for same goal  
✅ **Keyboard Control** - Cmd+1 to toggle detection on/off  
✅ **High Performance** - 60 FPS monitoring with <100ms response time  

## Installation

### Prerequisites

**macOS:**
```bash
brew install tesseract
```

**Linux:**
```bash
sudo apt-get install tesseract-ocr
```

### Build

```bash
cargo build --release
```

The binary will be at `target/release/fm-goal-musics`

## Configuration

On first run, a default config is created at:
- **macOS/Linux:** `./target/release/config/config.json` (next to executable)

### Config Structure

```json
{
  "capture_region": [400, 900, 1024, 50],
  "audio_file_path": "goal.mp3",
  "ocr_threshold": 80,
  "debounce_ms": 200,
  "enable_morph_open": false,
  "bench_frames": 500
}
```

**Parameters:**
- `capture_region`: `[x, y, width, height]` - Screen region to monitor
- `audio_file_path`: Path to MP3 file (relative to config directory)
- `ocr_threshold`: OCR threshold (auto-calculated via Otsu's method)
- `debounce_ms`: Minimum time between goal detections (200ms recommended)
- `enable_morph_open`: Morphological opening (future feature)
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

### Production Mode (Default)

```bash
cargo run --release
```

Or run the binary directly:
```bash
./target/release/fm-goal-musics
```

**Controls:**
- **Cmd+1**: Toggle detection on/off
- **Ctrl+C**: Quit application

### Test Mode

Test individual modules:
```bash
cargo run --release -- --test
```

This runs:
- Config loading test
- Audio playback test
- Screen capture test
- OCR detection test

## Performance

**Target:** <100ms total response time

Typical performance on modern hardware:
- **Capture:** 5-10ms
- **OCR:** 10-20ms
- **Total:** 30-50ms
- **FPS:** 60 FPS
- **Response:** <100ms from goal appearing to sound playing

## Troubleshooting

### "Permission denied" on macOS

Grant Screen Recording permission:
1. System Preferences > Security & Privacy
2. Privacy tab > Screen Recording
3. Add Terminal (or your terminal app)
4. Restart terminal

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
├── main.rs       # Main loop, keyboard control
├── config.rs     # Configuration management
├── audio.rs      # Audio preloading and playback
├── capture.rs    # Screen capture with scap
├── ocr.rs        # OCR with adaptive thresholding
└── utils.rs      # Shared utilities, timing, debouncing
```

## Development

### Build and Test

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Test mode
cargo run --release -- --test
```

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
- **GPU-accelerated**: scap uses Metal/Vulkan for capture
- **Preloaded audio**: rodio loads MP3 into memory

### Performance Optimizations

- 60 FPS capture target
- Adaptive threshold calculation
- Reuse of image buffers
- Minimal temporary file usage
- Efficient debouncing

## License

MIT

## Credits

Built with:
- [scap](https://github.com/CapSoftware/scap) - Screen capture
- [leptess](https://github.com/houqp/leptess) - Tesseract OCR wrapper
- [rodio](https://github.com/RustAudio/rodio) - Audio playback
- [rdev](https://github.com/Narsil/rdev) - Global keyboard hooks
- [Tesseract OCR](https://github.com/tesseract-ocr/tesseract) - Text recognition