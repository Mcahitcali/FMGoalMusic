# Testing Guide

## Overview
The `main.rs` file now contains separate test functions for each module. You can enable/disable tests by commenting/uncommenting the function calls in `main()`.

## Test Functions

### 1. `test_config()`
Tests configuration loading and display.

**What it tests:**
- Config file loading from platform directory
- Default config creation if missing
- Config value display

**Enable:**
```rust
test_config(&cfg);
```

---

### 2. `test_audio()`
Tests audio preloading and playback.

**What it tests:**
- Audio file loading into memory
- AudioManager initialization
- Optional: Sound playback (commented by default)

**Enable:**
```rust
test_audio(&cfg);
```

**To test playback:** Uncomment lines 59-63 in the function.

---

### 3. `test_capture()`
Tests screen capture functionality.

**What it tests:**
- Screen capture permission
- Capturer initialization
- Region capture
- Screenshot saving to `test_screenshot.png`

**Enable:**
```rust
test_capture(&cfg);
```

**Requirements:**
- macOS: Screen Recording permission
- Capture region must be within screen bounds

---

### 4. `test_ocr()`
Tests OCR text detection on captured screenshot.

**What it tests:**
- Tesseract OCR initialization
- Text detection from image
- "GOAL" keyword detection
- Binary threshold preprocessing

**Enable:**
```rust
test_ocr(&cfg);
```

**Requirements:**
- Tesseract installed (`brew install tesseract`)
- `test_screenshot.png` must exist (run `test_capture()` first)

---

## Current Configuration

Currently **only OCR test is enabled**:

```rust
fn main() {
    // ...
    
    // test_config(&cfg);
    // test_audio(&cfg);
    // test_capture(&cfg);
    test_ocr(&cfg);  // ← Only this is active
}
```

## Running Tests

### Test OCR Only (Current)
```bash
cargo run
```

### Test All Modules
Uncomment all test functions in `main()`:
```rust
test_config(&cfg);
test_audio(&cfg);
test_capture(&cfg);
test_ocr(&cfg);
```

Then run:
```bash
cargo run
```

### Test Specific Module
Comment all except the one you want:
```rust
// test_config(&cfg);
test_audio(&cfg);  // Only test audio
// test_capture(&cfg);
// test_ocr(&cfg);
```

---

## Test Workflow

### First Time Setup
1. **Run capture test** to create screenshot:
   ```rust
   test_capture(&cfg);
   ```

2. **Verify screenshot** contains text you want to detect

3. **Run OCR test** to detect text:
   ```rust
   test_ocr(&cfg);
   ```

### Testing OCR with "GOAL" Text
1. Open a text editor
2. Type "GOAL" in large, clear font
3. Position window in capture region (default: top-left 200x100 pixels)
4. Run `test_capture(&cfg)` to capture it
5. Run `test_ocr(&cfg)` to detect it

---

## Output Examples

### Config Test
```
=== Config Test ===
  Capture region: [0, 0, 200, 100]
  Audio file: goal.mp3
  OCR threshold: 150
  Debounce: 800ms
  Config path: /path/to/config.json
```

### Audio Test
```
=== Audio Test ===
✓ Audio manager initialized
  Audio file: /path/to/goal.mp3
```

### Capture Test
```
=== Capture Test ===
✓ Capture manager initialized
✓ Screenshot saved: test_screenshot.png
  Size: 400x200
```

### OCR Test
```
=== OCR Test ===
✓ OCR manager initialized

Testing test_screenshot.png:
  Detected text: 'GOAL'
  ✓ GOAL detected!
```

---

## Troubleshooting

### OCR: "test_screenshot.png not found"
**Solution:** Run `test_capture()` first to create the screenshot.

### OCR: "Failed to initialize OCR"
**Solution:** Install Tesseract:
```bash
# macOS
brew install tesseract

# Linux
sudo apt-get install tesseract-ocr
```

### Capture: "Permission denied"
**Solution:** Grant Screen Recording permission:
- macOS: System Preferences > Security & Privacy > Privacy > Screen Recording
- Add your terminal app to the list

### Audio: "Failed to initialize audio"
**Solution:** Place a `goal.mp3` file in the config directory:
```
target/debug/config/goal.mp3
```

---

## Next Steps

After testing individual modules, proceed to:
- **Step A.6:** Implement `utils.rs` (shared utilities)
- **Step A.7:** Implement full `main.rs` (wire everything together)
- **Step B:** Add latency instrumentation
