# Folder Structure

## Application Directory Layout

```
fm-goal-musics/
├── fm-goal-musics          # Executable (macOS/Linux)
├── fm-goal-musics.exe      # Executable (Windows)
└── config/
    ├── config.json         # Configuration file
    └── goal.mp3            # Goal celebration audio file
```

## Config Directory

The `config/` folder is located **next to the executable** in the same directory.

### Location Examples:

**Development (cargo run):**
```
target/debug/config/
├── config.json
└── goal.mp3
```

**Release Build:**
```
target/release/config/
├── config.json
└── goal.mp3
```

**Production Deployment:**
```
/path/to/app/config/
├── config.json
└── goal.mp3
```

## Setup Instructions

1. **Build the application:**
   ```bash
   cargo build --release
   ```

2. **The config folder will be auto-created** on first run at:
   ```
   target/release/config/config.json
   ```

3. **Add your goal.mp3 file:**
   ```bash
   cp /path/to/your/goal.mp3 target/release/config/goal.mp3
   ```

4. **Edit config.json** to customize settings:
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

## Audio File Path

The `audio_file_path` in config.json is **relative to the config directory**.

Examples:
- `"goal.mp3"` → `config/goal.mp3`
- `"sounds/goal.mp3"` → `config/sounds/goal.mp3`
- You can also use absolute paths if needed

## Benefits of This Approach

✓ **Portable** - Everything stays together  
✓ **Simple** - Easy to find and edit  
✓ **No system permissions** - No need for system config directories  
✓ **Easy deployment** - Just copy the folder  
✓ **Multiple instances** - Can run different configs in different folders
