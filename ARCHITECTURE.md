# FMGoalMusic Architecture

**Status**: Refactoring in Progress (see REFACTORING.md for details)

## Overview

FMGoalMusic is a desktop application that detects goals in Football Manager gameplay via OCR and plays customized music. The architecture follows **MVU (Model-View-Update)** pattern with **Event/Command** messaging.

## Core Patterns

1. **MVU (Model-View-Update)** - Elm architecture for GUI
   - Model: Application state
   - View: Pure rendering functions (gui/views/)
   - Update: Message handlers (gui/update.rs)
   - Messages: User actions + events (gui/messages.rs)

2. **Event/Command Segregation**
   - Events: Notifications (past tense) - "GoalDetected", "ConfigChanged"
   - Commands: Requests (imperative) - "PlayAudio", "StartDetection"

3. **Strategy Pattern** - Pluggable detection types and audio effects

4. **Composite Pattern** - Mix multiple audio sources

## Module Structure

```
src/
├── gui/                    # User interface (MVU pattern)
│   ├── mod.rs             # MVU orchestration (~200 LOC)
│   ├── model.rs           # Application state
│   ├── messages.rs        # Message enum
│   ├── update.rs          # Message handlers
│   └── views/             # Pure rendering
│       ├── library.rs
│       ├── team_selection.rs
│       ├── settings.rs
│       ├── help.rs
│       └── region_selector.rs
│
├── messaging/              # Event/Command system
│   ├── events.rs          # Event types
│   ├── commands.rs        # Command types
│   ├── bus.rs             # Event dispatcher
│   └── executor.rs        # Command executor
│
├── state/                  # State management
│   ├── app_state.rs       # Main app state
│   ├── process_state.rs   # Detection state machine
│   ├── config.rs          # Configuration + validation
│   └── repository.rs      # Persistence
│
├── audio/                  # Audio system
│   ├── manager.rs         # Coordinator
│   ├── player.rs          # Individual player
│   ├── mixer.rs           # Mix sources
│   ├── source.rs          # Source types
│   └── effects/           # Audio effects
│       ├── fade.rs
│       ├── volume.rs
│       └── limiter.rs
│
├── detection/              # Detection abstraction
│   ├── detector.rs        # Detector trait
│   ├── goal.rs            # Goal detection
│   ├── kickoff.rs         # Match start
│   ├── match_end.rs       # Match end + score
│   ├── pipeline.rs        # Detection chain
│   └── phrases.rs         # i18n phrases
│
├── ocr/                    # OCR implementation
│   ├── mod.rs
│   ├── detection.rs
│   ├── preprocessing.rs
│   └── text_extraction.rs
│
├── wizard/                 # First-run setup
│   └── steps/
│
├── capture.rs              # Screen capture
├── teams.rs                # Team database
├── team_matcher.rs         # Team matching
└── utils.rs                # Utilities
```

## Data Flow

### User Action Flow (MVU)
```
User clicks button
  → View emits Message
    → Update handles Message
      → Update sends Command to executor
        → Executor performs action
          → Executor emits Event
            → Event converted to Message
              → Update updates Model
                → View re-renders with new Model
```

### Detection Flow (Event-Driven)
```
Detection thread
  → Captures screen
    → Runs OCR
      → Detection pipeline checks all detectors
        → Detector emits GoalDetected event
          → Audio handler receives event
            → Sends PlayAudio command
              → Audio manager plays music
```

## Concurrency Model

- **Threading**: `std::thread` for background work (detection loop)
- **Channels**: `crossbeam-channel` for inter-thread communication
- **Locks**: `parking_lot::Mutex` / `parking_lot::RwLock` for shared state
- **No async runtime** (tokio) - not needed for GUI app

## Error Handling

- **`thiserror`** - Module/library errors (typed, structured)
- **`anyhow`** - Application errors (with context chains)
- Errors propagate via `Result<T, E>` and are logged + shown in UI

## Logging

- **`tracing`** - Structured logging with spans
- Logs written to rotating files in user config directory
- File rotation: 10MB max, keep last 5 files

## State Management

- Centralized in `state/` module
- Process state machine: Stopped → Starting → Running → Stopping
- Config validation on load/save
- State changes emit events for reactive updates
- Uses `parking_lot::RwLock` for read-heavy access

## Audio System

- Multiple simultaneous sources (music, ambiance, crowd, commentator)
- Effect chain: source → fade in → volume → limit → fade out
- Event-driven playback (subscribe to detection events)
- Mixer combines multiple audio streams

## Detection System

- Pluggable detector trait for extensibility
- Detection pipeline runs all enabled detectors
- Detectors:
  - **GoalDetector** - Detects "GOAL FOR {team}"
  - **KickoffDetector** - Detects match start (00:00, "Kick Off")
  - **MatchEndDetector** - Detects FT and parses score
- i18n support via external phrase files

## External Data

```
assets/
├── i18n/              # Language phrase files
│   ├── en.json
│   ├── es.json
│   └── tr.json
├── teams/
│   └── teams.json     # Team database
└── audio/
    └── default/       # Default audio files
```

## Testing Strategy

- **Unit tests**: Pure functions (detectors, audio effects, state logic)
- **Integration tests**: Module interactions (event bus, state management)
- **GUI tests**: Message handlers (update functions)
- **End-to-end**: Detection flow with mocked OCR

## Performance Considerations

- OCR runs in background thread (doesn't block UI)
- 16ms detection loop interval (configurable)
- Debouncing prevents duplicate detections (3s window)
- RwLock for read-heavy state access (UI reads frequently)
- Audio decoding happens once at load time

## Extensibility

### Adding a New Detection Type
1. Implement `Detector` trait in `detection/`
2. Add to detection pipeline
3. Define new Event type in `messaging/events.rs`
4. Add audio handler if needed
~100 LOC total

### Adding a New Audio Source
1. Add variant to `AudioSource` enum
2. Subscribe to relevant event
3. Add UI controls if needed
~50 LOC total

### Adding a New Language
1. Create `assets/i18n/{lang}.json`
2. Add phrases for goal/kickoff/match-end
3. Users can select in Settings
~30 minutes, no code changes

## Design Principles

1. **Separation of Concerns** - Each module has single responsibility
2. **Pure Functions** - Views are pure rendering, no side effects
3. **Explicit over Implicit** - Prefer std::thread over async magic
4. **Type Safety** - Use enums and structs over primitives
5. **Event-Driven** - Modules communicate via events, not direct calls
6. **Community-Friendly** - External data files for easy contributions

## Related Documents

- `REFACTORING.md` - Detailed refactoring plan and progress
- `Doc/ROADMAP.md` - Product roadmap
- `Doc/FEATURES.md` - Feature specifications
- `README.md` - User documentation

---

**Last Updated**: 2025-11-04
**Version**: Refactoring in progress
