# Ambiance Sounds Feature

## Purpose
Add football ambiance sounds to enhance the game experience with realistic crowd reactions and stadium atmosphere.

## Overview
This feature introduces a separate audio layer for ambiance sounds that play alongside music tracks, with independent volume control for each audio type.

## Core Features

### 1. Ambiance Sound System
- Play ambiance sounds alongside music tracks
- Support for multiple ambiance sound types:
  - Goal crowd cheer
  - Fan chants
  - Stadium atmosphere
  - Match start sounds
  - Continuous ambient noise

### 2. Dual Volume Control
- **Music Volume**: Control the volume of goal music tracks
- **Ambiance Volume**: Control the volume of ambiance sounds independently
- Both volumes adjustable from 0-100%

### 3. Sound Triggering
- Goal scored: Play both goal music + crowd cheer sound
- Ambiance sounds play at lower volume to complement music
- Proper mixing of multiple audio sources

## Implementation Details

### Audio Architecture
- Extend audio system to support multiple simultaneous audio streams
- Add ambiance audio player separate from music player
- Implement volume mixing for both streams

### Configuration
- Add `ambiance_volume` to config alongside `music_volume`
- Store ambiance sound file paths in config
- Support multiple ambiance sound categories

### UI Updates
- Separate volume sliders for Music and Ambiance
- Clear labeling for each audio type
- Simplified, modern design
- Visual feedback for volume changes

## File Structure
```
config/
  sounds/
    goal_crowd_cheer.wav       # Goal celebration sound
    fan_chants.wav             # Future: Fan singing/chanting
    stadium_ambient.wav        # Future: Background stadium noise
    match_start.wav            # Future: Match beginning sound
```

## Technical Implementation

### Phase 1 (Current)
1. Create ambiance audio player
2. Add volume controls for both music and ambiance
3. Implement goal crowd cheer sound
4. Update UI with dual volume sliders

### Phase 2 (Future)
1. Add more ambiance sound types
2. Implement sound categories
3. Add contextual sound triggering
4. Enhanced mixing capabilities

## User Experience
- Clean, intuitive volume controls
- Clear separation between music and ambiance
- Realistic football match atmosphere
- Non-intrusive ambiance that enhances music

## Status
- **Phase**: Implementation
- **Branch**: feature/ambiance-sounds
- **Priority**: High
