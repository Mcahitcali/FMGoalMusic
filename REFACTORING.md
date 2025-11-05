# FMGoalMusic Architecture Refactoring

**Branch**: `refactor/architecture-redesign`
**Started**: 2025-11-04
**Status**: In Progress

## 🎯 Goals

Transform FMGoalMusic from a monolithic architecture to a clean, maintainable, event-driven system using:
- **MVU (Model-View-Update)** pattern for GUI (perfect for egui)
- **Event/Command Segregation** for clear module communication
- **Strategy Pattern** for extensibility (detectors, audio effects)
- **Proper tooling** (CI, structured errors, tracing)

### Key Targets
- Reduce `gui/mod.rs` from **1,981 LOC → <200 LOC** (90% reduction)
- Support future features: match detection, i18n, audio mixing, wizard
- Zero functional regressions
- All existing features work identically

## 🏗️ Architecture Overview

### Core Patterns
1. **MVU (Model-View-Update)** - Elm architecture, perfect for egui immediate mode
2. **Event/Command Segregation** - Events notify (past), Commands request (future)
3. **Strategy Pattern** - Different detection types, audio effects
4. **Composite Pattern** - Audio mixing multiple sources
5. **Repository Pattern** - State persistence
6. **Factory Pattern** - Create detectors, audio players

### Concurrency Model
- **Threading**: `std::thread` for background work (no tokio - overkill for GUI)
- **Channels**: `crossbeam-channel` for communication (better than std::mpsc)
- **Locks**: `parking_lot::Mutex` for shared state (faster, no poisoning, prevents UI freezes)

### Error Handling
- **`thiserror`** - For library/module errors (structured, typed)
- **`anyhow`** - For application errors (with context)

### Logging
- **`tracing`** - Replace `log` + `flexi_logger` (structured, spans, better async support)

## 📁 New Module Structure

```
src/
  main.rs
  gui_main.rs

  gui/
    mod.rs           # MVU orchestration (~200 LOC)
    model.rs         # Application state/model
    messages.rs      # Message enum for MVU
    update.rs        # Message handlers
    views/           # Pure rendering functions
      library.rs
      team_selection.rs
      settings.rs
      help.rs
      region_selector.rs

  messaging/         # Event/Command system
    mod.rs
    events.rs        # Event types (GoalDetected, ConfigChanged, etc.)
    commands.rs      # Command types (PlayAudio, StartDetection, etc.)
    bus.rs           # Event dispatcher
    executor.rs      # Command executor

  state/             # Centralized state management
    mod.rs
    app_state.rs     # Main application state
    process_state.rs # Detection process state
    config.rs        # Configuration with validation
    repository.rs    # Persistence

  audio/             # Audio system
    mod.rs
    manager.rs       # Audio coordinator
    player.rs        # Individual player
    mixer.rs         # Mix multiple sources
    source.rs        # Audio source types
    effects/
      fade.rs
      volume.rs
      limiter.rs

  detection/         # Detection abstraction
    mod.rs
    detector.rs      # Detector trait
    goal.rs          # Goal detection
    kickoff.rs       # Match start detection
    match_end.rs     # Match end + score detection
    pipeline.rs      # Chain of responsibility
    phrases.rs       # i18n phrase loading

  ocr/               # Low-level OCR (unchanged)
  capture.rs         # Screen capture (unchanged)
  teams.rs           # Team database (unchanged)
  team_matcher.rs    # Team matching (unchanged)
  utils.rs           # Utilities (unchanged)

  wizard/            # First-run setup
    mod.rs
    steps/
      welcome.rs
      language.rs
      monitor.rs
      team.rs
      test.rs

assets/              # External data
  i18n/
    en.json          # English phrases
    es.json          # Spanish phrases
    tr.json          # Turkish phrases
  teams/
    teams.json       # Team database (move from config/)
  audio/
    default/         # Default audio files
```

## 📋 Refactoring Phases

### Phase 0: Tooling & Infrastructure ⏳
**Goal**: Set up safety nets before refactoring
**Time**: 3-4 hours
**Status**: Not Started

#### Tasks
- [ ] Add dependencies: `anyhow`, `thiserror`, `tracing`, `crossbeam-channel`, `parking_lot`
- [ ] Create `src/error.rs` with app error types using `thiserror`
- [ ] Migrate logging from `log` + `flexi_logger` to `tracing` + `tracing-subscriber`
- [ ] Replace `std::sync::Mutex` → `parking_lot::Mutex`
- [ ] Replace `std::sync::mpsc` → `crossbeam::channel`
- [ ] Set up CI/CD: `.github/workflows/ci.yml` (fmt, clippy, test, build)
- [ ] Configure clippy with strict lints
- [ ] **Test**: Build passes, CI works

#### Acceptance Criteria
- ✅ CI pipeline runs successfully
- ✅ All tests pass
- ✅ Structured errors in key modules
- ✅ Tracing logs with spans visible
- ✅ No `std::sync::Mutex` or `std::sync::mpsc` remaining

---

### Phase 1: State Management Foundation
**Goal**: Define the data model before building around it
**Time**: 4-5 hours
**Status**: Not Started

#### Tasks
- [ ] Create `src/state/` module
- [ ] `app_state.rs` - Main application state
- [ ] `process_state.rs` - Detection state machine (Stopped, Starting, Running, Stopping)
- [ ] `config.rs` - Move config logic, add validation methods
- [ ] `repository.rs` - Load/save config with error handling
- [ ] Remove `Arc<Mutex<AppState>>` sprawl, centralize in StateManager
- [ ] Use `parking_lot::RwLock` for read-heavy state access
- [ ] **Test**: Build passes, state loading works

#### Acceptance Criteria
- ✅ State module compiles
- ✅ Config validation works (volume 0-100, paths exist, etc.)
- ✅ State machine transitions are clear
- ✅ Repository loads/saves config correctly

---

### Phase 2: Event/Command System
**Goal**: Decouple modules with messaging
**Time**: 4-5 hours
**Status**: Not Started

#### Tasks
- [ ] Create `src/messaging/` module
- [ ] `events.rs` - Define event types (GoalDetected, MatchStarted, ConfigChanged, etc.)
- [ ] `commands.rs` - Define command types (PlayAudio, StartDetection, SaveConfig, etc.)
- [ ] `bus.rs` - Event dispatcher with pub/sub pattern using crossbeam-channel
- [ ] `executor.rs` - Command executor, emit events on completion
- [ ] Write unit tests for event bus
- [ ] **Test**: Build passes, event bus works in isolation

#### Acceptance Criteria
- ✅ Can subscribe to events
- ✅ Can dispatch events to multiple subscribers
- ✅ Can execute commands
- ✅ Commands emit completion events
- ✅ Non-blocking dispatch works

---

### Phase 3: Split GUI with MVU
**Goal**: Break down gui/mod.rs (1,981 LOC → ~200 LOC)
**Time**: 8-10 hours
**Status**: Not Started

#### Tasks
- [ ] Create `gui/model.rs` - Extract model from AppState
- [ ] Create `gui/messages.rs` - Define Message enum for MVU
- [ ] Create `gui/update.rs` - Message handler (emit commands, update model)
- [ ] Create `gui/views/library.rs` - Extract Library tab rendering
- [ ] Create `gui/views/team_selection.rs` - Extract Team tab rendering
- [ ] Create `gui/views/settings.rs` - Extract Settings tab rendering
- [ ] Create `gui/views/help.rs` - Extract Help tab rendering
- [ ] Create `gui/views/region_selector.rs` - Extract region selector
- [ ] Refactor `gui/mod.rs` to MVU loop: update → model → view → messages
- [ ] Wire event bus to convert events to messages
- [ ] **Test**: Build passes, all GUI features work

#### Acceptance Criteria
- ✅ gui/mod.rs is <250 LOC
- ✅ Each view is pure rendering function
- ✅ All tabs render correctly
- ✅ Detection start/stop works
- ✅ Music/team selection works
- ✅ Settings save correctly
- ✅ Region selector works

---

### Phase 4: Audio System Redesign
**Goal**: Support multiple simultaneous audio sources
**Time**: 6-8 hours
**Status**: Not Started

#### Tasks
- [ ] Create `src/audio/` module structure
- [ ] `manager.rs` - Audio coordinator managing multiple players
- [ ] `player.rs` - Individual audio player
- [ ] `mixer.rs` - Mix multiple audio sources
- [ ] `source.rs` - AudioSource enum (GoalMusic, Ambiance, CrowdCheer, Commentator)
- [ ] `effects/fade.rs` - Fade in/out effect
- [ ] `effects/volume.rs` - Volume control
- [ ] `effects/limiter.rs` - Time limit effect
- [ ] Integrate with event system (subscribe to GoalDetected, etc.)
- [ ] **Test**: Build passes, audio plays correctly

#### Acceptance Criteria
- ✅ Can play multiple audio sources simultaneously
- ✅ Effects chain works (fade in → volume → fade out)
- ✅ Music + ambiance play together
- ✅ Event-driven playback works

---

### Phase 5: Detection Abstraction
**Goal**: Add goal/kickoff/match-end detection easily
**Time**: 6-8 hours
**Status**: Not Started

#### Tasks
- [ ] Create `src/detection/` module
- [ ] `detector.rs` - Detector trait definition
- [ ] `goal.rs` - GoalDetector implementation
- [ ] `kickoff.rs` - KickoffDetector implementation (for v0.2)
- [ ] `match_end.rs` - MatchEndDetector implementation (for v0.3)
- [ ] `pipeline.rs` - Detection pipeline (run all detectors)
- [ ] `phrases.rs` - Load phrases from external JSON
- [ ] Integrate with event system (emit GoalDetected, MatchStarted, etc.)
- [ ] **Test**: Build passes, goal detection works

#### Acceptance Criteria
- ✅ Goal detection works (existing behavior)
- ✅ Kickoff detection ready (for v0.2)
- ✅ Match end detection ready (for v0.3)
- ✅ Easy to add new detector (<100 LOC)
- ✅ i18n phrase loading prepared

---

### Phase 6: First-Run Wizard
**Goal**: Easy onboarding for new users
**Time**: 4-6 hours
**Status**: Not Started

#### Tasks
- [ ] Create `src/wizard/` module
- [ ] `steps/welcome.rs` - Welcome screen
- [ ] `steps/language.rs` - Language selection
- [ ] `steps/monitor.rs` - Monitor selection (auto-detect + manual)
- [ ] `steps/team.rs` - Favorite team selection
- [ ] `steps/test.rs` - Test detection
- [ ] Wizard state machine (WizardStep enum)
- [ ] Integration with main app (show on first run)
- [ ] **Test**: Build passes, wizard flow works

#### Acceptance Criteria
- ✅ Wizard shows on first run
- ✅ All steps work correctly
- ✅ Config saved on completion
- ✅ Can re-run from Settings

---

### Phase 7: Externalize Data
**Goal**: Make data easy to edit and contribute
**Time**: 2-3 hours
**Status**: Not Started

#### Tasks
- [ ] Create `assets/i18n/` directory
- [ ] `assets/i18n/en.json` - English phrases
- [ ] `assets/i18n/es.json` - Spanish phrases (for v0.3)
- [ ] `assets/i18n/tr.json` - Turkish phrases (for v0.3)
- [ ] Move `config/teams.json` → `assets/teams/teams.json`
- [ ] Update loading logic (embed defaults, load from filesystem, merge)
- [ ] **Test**: Build passes, phrases load correctly

#### Acceptance Criteria
- ✅ Phrases loaded from JSON files
- ✅ Can override embedded defaults
- ✅ Community can add translations easily

---

## ⏱️ Time Tracking

| Phase | Estimated | Actual | Status |
|-------|-----------|--------|--------|
| 0. Tooling | 3-4h | - | Not Started |
| 1. State | 4-5h | - | Not Started |
| 2. Events | 4-5h | - | Not Started |
| 3. GUI MVU | 8-10h | - | Not Started |
| 4. Audio | 6-8h | - | Not Started |
| 5. Detection | 6-8h | - | Not Started |
| 6. Wizard | 4-6h | - | Not Started |
| 7. Data | 2-3h | - | Not Started |
| **Total** | **37-49h** | **-** | **0%** |

## ✅ Success Criteria

### Technical
- [ ] CI passes (fmt, clippy, test, build)
- [ ] No file over 500 LOC
- [ ] gui/mod.rs: 1,981 LOC → <200 LOC
- [ ] All business logic has no GUI dependencies
- [ ] Can add new detection type in <100 LOC
- [ ] Can add new audio source in <50 LOC
- [ ] i18n phrases loaded from external JSON

### Functional
- [ ] Zero functional regressions
- [ ] All existing features work identically:
  - [ ] Music selection and playback
  - [ ] Team selection and matching
  - [ ] Region selection
  - [ ] Goal detection with debouncing
  - [ ] Ambiance playback
  - [ ] Settings (volume, threshold, morph, etc.)
  - [ ] Config persistence
  - [ ] Multi-monitor selection
  - [ ] Update checker
  - [ ] File logging

## 🎁 Expected Benefits

### Immediate (After Phase 3)
- 90% reduction in gui/mod.rs size (1,981 → ~200 LOC)
- CI catches issues automatically
- Structured errors with context
- Better logging with tracing spans
- Clear separation of concerns

### Medium-term (After Phase 5)
- Add kickoff detection in 2 hours (v0.2 feature)
- Add match end detection in 2 hours (v0.3 feature)
- Add Spanish translation in 30 minutes (v0.3 feature)
- Audio mixing ready for multiple sources

### Long-term (After Phase 7)
- All v0.2-v0.4 features become straightforward
- Community can contribute translations easily
- Testing becomes possible (pure functions, DI)
- System tray integration simple (just events)
- First-run wizard improves adoption

## 📝 Notes

### Design Decisions

**Why MVU instead of MVC?**
- egui is immediate mode - MVU (Elm architecture) is the natural fit
- Pure rendering functions are easier to reason about
- No need for controllers - update function handles all logic

**Why Event/Command separation?**
- Events = notifications (past tense, broadcast)
- Commands = requests (imperative, targeted)
- Clear semantics improve code clarity and debugging

**Why no tokio?**
- Desktop GUI app, not async I/O bound
- std::thread is simpler and more explicit
- Avoids async complexity where not needed

**Why parking_lot over std::sync?**
- 1-2x faster
- No poisoning (simpler error handling)
- Smaller memory footprint
- Prevents potential UI freezes

### Risks & Mitigation

**Risk**: Breaking existing functionality during refactoring
**Mitigation**: Test build after every phase, commit working states

**Risk**: Refactoring taking longer than estimated
**Mitigation**: Break phases into smaller commits, can pause at any phase

**Risk**: Performance regression
**Mitigation**: Benchmark key paths (OCR loop, audio playback) before/after

## 🔗 Related Documents

- `Doc/ROADMAP.md` - Product roadmap (v0.2-v0.4 features)
- `Doc/BUGS.md` - Known bugs and fixes
- `Doc/FEATURES.md` - Feature specifications
- `README.md` - Project overview

---

**Last Updated**: 2025-11-04
**Next Review**: After each phase completion
