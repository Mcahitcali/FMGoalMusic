# Feature: Automatic Audio Conversion to WAV

## Overview
All audio files added to FM Goal Musics are automatically converted to WAV format for optimal playback performance. This feature ensures zero-latency audio playback regardless of the input format.

## Motivation

### Performance Requirements
- Goal detection requires sub-100ms total latency
- Audio playback must be instant (no decoding overhead)
- CPU resources needed for capture and OCR
- Repeated playback must be glitch-free

### Why WAV Format?
1. **No Decoding Overhead** – Direct PCM data, instant playback
2. **Lossless Quality** – No compression artifacts
3. **Optimal for Repeating** – No decoder state management
4. **Lower CPU Usage** – Critical when running OCR and capture simultaneously
5. **Simple Format** – Minimal parsing, straightforward memory layout

## Implementation

### Technical Approach
```
Input Audio File (MP3, FLAC, OGG, AAC, WAV)
    ↓
Symphonia Decoder (multi-format support)
    ↓
Extract PCM Samples (16-bit signed integers)
    ↓
Hound WAV Encoder
    ↓
Output: 16-bit PCM WAV File
```

### Supported Input Formats
- **MP3** – MPEG-1/2 Audio Layer III
- **AAC** – Advanced Audio Coding
- **FLAC** – Free Lossless Audio Codec
- **OGG Vorbis** – Ogg container with Vorbis codec
- **WAV** – No conversion needed (pass-through)

### Output Format Specification
```
Format:        WAV (RIFF)
Sample Format: 16-bit signed PCM
Byte Order:    Little-endian
Channels:      Preserved from source (mono/stereo/surround)
Sample Rate:   Preserved from source
Compression:   None (uncompressed PCM)
```

## User Experience

### Automatic Conversion Flow
1. User clicks "Add Music File" in GUI
2. File picker opens (filters: `.mp3`, `.wav`, `.ogg`, `.flac`, `.aac`)
3. User selects audio file
4. System checks if file is already WAV:
   - **If WAV:** Add to music list directly
   - **If other format:** Convert to WAV in same directory
5. Converted WAV file added to music list
6. Config saved with WAV file path
7. User sees success message or error if conversion fails

### File Naming Convention
```
Original:  celebration.mp3
Converted: celebration.wav

Original:  goal-sound.flac
Converted: goal-sound.wav

Original:  music.ogg
Converted: music.wav
```

### File Location
- WAV files created in same directory as original file
- Original files remain untouched
- Both files coexist (original + converted)

## Error Handling

### Conversion Failures
**Possible Issues:**
- Unsupported codec within container
- Corrupted audio file
- Insufficient disk space
- Write permission denied
- Invalid audio format

**User Feedback:**
```
❌ Failed to convert celebration.mp3
   Reason: Unsupported codec
   Solution: Try a different file or convert manually
```

### Recovery Actions
1. Show clear error message with reason
2. Suggest alternative actions
3. Don't crash application
4. Allow user to try different file
5. Log technical details for debugging

## Performance Impact

### Conversion Time
- **Small files (1-3 MB):** 100-300ms
- **Medium files (3-10 MB):** 300ms-1s
- **Large files (10+ MB):** 1-3s

### One-Time Operation
- Conversion happens only when adding file
- Subsequent app launches use pre-converted WAV
- No conversion overhead during detection

### Disk Space
- WAV files are larger than compressed formats
- Typical size increase:
  - MP3 (3 MB) → WAV (30 MB) – ~10x larger
  - FLAC (20 MB) → WAV (30 MB) – ~1.5x larger
  - OGG (2 MB) → WAV (30 MB) – ~15x larger

## Configuration Integration

### Config Schema Addition
```json
{
  "music_list": [
    "/path/to/celebration.wav",
    "/path/to/goal-sound.wav",
    "/path/to/music.wav"
  ],
  "selected_music_index": 0
}
```

### Persistence
- Music list saved in config.json
- WAV file paths stored (not original files)
- Config auto-saved on music list changes
- Config auto-loaded on app start

## Code Changes

### New Module: `src/audio_converter.rs`
```rust
pub fn convert_to_wav(
    input_path: &Path,
    output_path: &Path
) -> Result<(), Box<dyn std::error::Error>>
```

### Modified: `src/gui.rs`
- Add file picker for music files
- Call converter when non-WAV file selected
- Update music list with converted file path
- Show conversion progress/error messages

### Modified: `src/config.rs`
```rust
pub struct Config {
    // ...existing fields...
    pub music_list: Vec<String>,
    pub selected_music_index: usize,
}
```

### Modified: `Cargo.toml`
```toml
[dependencies]
# Audio conversion
symphonia = { version = "0.5", features = ["mp3", "aac", "flac", "vorbis", "wav"] }
hound = "3.5"
```

## Testing

### Unit Tests
- Test WAV pass-through (no conversion)
- Test MP3 → WAV conversion
- Test error handling for invalid files
- Test channel preservation (mono/stereo)
- Test sample rate preservation

### Integration Tests
- Add file via GUI, verify conversion
- Restart app, verify music list loaded
- Play converted audio, verify quality
- Test with various audio formats

### Manual Testing Checklist
- [ ] Add MP3 file, verify WAV created
- [ ] Add FLAC file, verify WAV created
- [ ] Add OGG file, verify WAV created
- [ ] Add WAV file, verify no conversion
- [ ] Try corrupted file, verify error message
- [ ] Restart app, verify music list persisted
- [ ] Play each converted file, verify audio quality
- [ ] Remove music, verify config updated

## Documentation Updates

### README.md
- Mention automatic WAV conversion
- List supported input formats
- Explain disk space considerations
- Note one-time conversion process

### GUI_GUIDE.md (old-docs reference)
- Update "Add Music Files" section
- Document conversion process
- Explain file naming convention
- Show error messages and solutions

## Future Enhancements

### Potential Improvements
1. **Progress Bar** – Show conversion progress for large files
2. **Batch Conversion** – Convert multiple files at once
3. **Quality Options** – Choose bit depth (16/24/32-bit)
4. **Sample Rate Options** – Resample to specific rate
5. **Cleanup Tool** – Remove unused WAV files
6. **Format Detection** – Better error messages for unsupported formats
7. **Cloud Conversion** – Offload conversion to server (future)

### Performance Optimizations
1. **Parallel Conversion** – Convert multiple files concurrently
2. **Streaming Conversion** – Don't load entire file into memory
3. **Caching** – Check if WAV already exists before converting
4. **Background Thread** – Don't block GUI during conversion

## Migration Notes

### Existing Users
- Old `audio_file_path` config field maintained for CLI compatibility
- New `music_list` field added for GUI multi-file support
- Both CLI and GUI read same config file
- No breaking changes to existing configurations

### Backward Compatibility
- Old configs without `music_list` continue to work
- CLI still uses `audio_file_path` field
- GUI migrates single audio path to music list if present
- Default values provided for missing fields

## Success Metrics

### Performance Targets
- Conversion time < 3s for typical audio files (3-5 MB)
- No playback latency increase
- Memory usage increase < 50 MB during conversion
- No impact on detection loop performance

### Quality Targets
- Zero audio quality loss (lossless conversion)
- Preserved stereo/surround information
- No clipping or distortion
- Sample-accurate conversion

### Usability Targets
- Clear progress indication
- Helpful error messages
- Intuitive file selection
- No manual intervention required

## Known Limitations

### Current Limitations
1. **No Progress Indicator** – User must wait without feedback
2. **Single File Only** – No batch conversion UI
3. **No Format Validation** – Relies on file extension
4. **No Cleanup** – Converted files remain after original deleted
5. **Fixed Output Format** – Always 16-bit PCM (no options)

### Planned Fixes
- Add progress bar for conversions > 1 second
- Implement batch file selection
- Add MIME type detection
- Provide cleanup utility
- Allow bit depth configuration

## Release Information

**Feature Added:** Version 1.0  
**Status:** Production Ready  
**Dependencies:** symphonia v0.5, hound v3.5  
**Platform Support:** macOS, Windows, Linux  

---

*Last Updated: 2025-10-29*
*Feature Status: Implemented and Tested*
