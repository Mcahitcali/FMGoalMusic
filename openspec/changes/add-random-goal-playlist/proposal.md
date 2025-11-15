# Change: Multi-song random goal celebration playlist

## Why
FMGoalMusic currently allows configuring only a single goal celebration track. Users want to build a small playlist of songs that can play on goals, with random selection and a rule to avoid repeating the last song when multiple tracks are selected.

## What Changes
- Add support for selecting multiple tracks as a "goal celebration playlist" in the Library UI.
- Persist the goal playlist selection in configuration alongside the existing music library.
- Update the detection loop to choose a random track from the goal playlist when a goal is detected.
- Enforce a "no immediate repeat" rule when the playlist contains more than one track.
- Preserve existing behavior for single-track configurations (a single selected track can still play on consecutive goals).
- Introduce a small random-number dependency and minimal additional state to track the last played track.

## Impact
- Affected specs: gui (library / goal music configuration, dashboard summary, detection behavior)
- Affected code: src/state/app_state.rs, src/config.rs, src/gui/view.rs, src/gui/controller.rs, src/audio.rs, Cargo.toml
