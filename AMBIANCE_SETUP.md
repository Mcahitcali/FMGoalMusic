# Ambiance Sounds Setup Guide

## Overview
The ambiance sounds feature allows you to play crowd reactions (like cheers) alongside your goal music for a more immersive Football Manager experience.

## Quick Setup

### 1. Add Your Goal Cheer Sound

1. Launch the FM Goal Musics application
2. Navigate to the **🎺 Ambiance Sounds** section in the GUI
3. Click **➕ Add Goal Cheer Sound**
4. Select your crowd cheer WAV file (e.g., `goal_crowd_cheer.wav`)
   - **Note:** Currently only WAV format is supported for ambiance sounds
   - You can place your WAV files in `config/sounds/` directory

### 2. Adjust Volumes

Use the **🔊 Volume Controls** section to set:

- **🎵 Music**: Volume for your goal music tracks (0-100%)
  - Default: 100%
  - Controls the main celebration music
  
- **🔉 Ambiance**: Volume for crowd cheer sounds (0-100%)
  - Default: 60%
  - Typically lower than music so it enhances rather than overwhelms

### 3. Test It Out

1. Select a goal music from your music list
2. Click **▶️ Start Detection**
3. When a goal is detected:
   - Your selected music will play at **Music** volume
   - The crowd cheer will play simultaneously at **Ambiance** volume

## File Organization

### Recommended Structure
```
config/
├── sounds/
│   ├── goal_crowd_cheer.wav      ← Your crowd cheer sound
│   ├── fan_chants.wav            ← (Future: fan singing)
│   └── stadium_ambient.wav       ← (Future: background noise)
└── musics/
    └── YourMusic.wav              ← Your goal celebration music
```

## Tips for Best Experience

### Finding Good Crowd Cheer Sounds
- Search for "football crowd cheer" or "stadium crowd goal reaction"
- Ensure the file is in WAV format
- Recommended length: 3-8 seconds
- The sound should complement your music, not overpower it

### Volume Balance
- Start with Music: 100%, Ambiance: 60%
- If ambiance is too quiet: increase to 70-80%
- If ambiance overwhelms music: decrease to 40-50%
- Use the **▶️ Preview** button to test your music (note: preview doesn't play ambiance)

### Performance Notes
- Both music and ambiance are preloaded into memory
- No performance impact during goal detection
- Playback latency remains under 100ms

## Current Limitations

1. **Ambiance Format**: Only WAV files supported (MP3, OGG not yet supported for ambiance)
2. **Single Ambiance Sound**: Currently only supports one goal cheer sound
3. **No Ambiance Preview**: Preview button only plays music, not ambiance

## Future Enhancements

Planned features (not yet implemented):
- Multiple ambiance categories (fan chants, stadium atmosphere, match start sounds)
- Support for MP3/OGG ambiance files
- Contextual ambiance (different sounds for home/away goals)
- Ambiance preview functionality

## Troubleshooting

### Ambiance Not Playing
1. Check that you've added a goal cheer sound file
2. Verify the file path is correct (shown in GUI)
3. Ensure ambiance volume is above 0%
4. Check the file exists at the specified path

### Volume Not Saving
- Volumes are saved automatically when you adjust the sliders
- Check that config file is writable at: `config/config.json`

### File Not Found Error
- Ensure your WAV file exists at the path shown in the GUI
- Try removing and re-adding the ambiance sound file
- Check file permissions (should be readable)

## Example Configuration

Your `config/config.json` will include:
```json
{
  "music_volume": 1.0,
  "ambiance_volume": 0.6,
  "goal_ambiance_path": "config/sounds/goal_crowd_cheer.wav"
}
```

## Need Help?

If you encounter issues:
1. Check the console output for error messages
2. Verify file paths and formats
3. Try rebuilding: `cargo build --release`
4. Check that the file mentioned in the error actually exists

---

*Last Updated: 2025-10-30*
*Feature Version: 1.0*
