## Context
Users want to configure multiple goal celebration songs and have FMGoalMusic pick a random one when a goal is detected, without repeating the last song when multiple tracks are available. The current implementation assumes a single selected track and wires a single audio buffer into the detection loop.

## Goals / Non-Goals
- Goals:
  - Allow multi-track selection for goal celebrations using the existing music library.
  - Implement random selection with a simple "no immediate repeat" rule when the playlist has more than one track.
  - Keep configuration backwards compatible with existing single-track setups.
- Non-Goals:
  - Building a full playlist management UI (ordering, reordering, complex metadata).
  - Implementing advanced scheduling or weighted randomness.

## Decisions
- Decision: Represent the goal playlist as a list of indices into the existing `music_list` (e.g., `Vec<usize>`), plus an in-memory `last_played_index: Option<usize>` in AppState that is not persisted.
- Decision: Keep the existing `selected_music_index` for backward compatibility and treat it as an implicit single-track playlist whenever the explicit goal playlist is empty.
- Decision: Use the `rand` crate to pick random indices from the playlist rather than rolling a custom RNG.
- Decision: Use a single `AudioManager` instance in the detection loop and swap its underlying audio buffer before each play, instead of allocating a separate audio manager per track.

## Risks / Trade-offs
- Risk: Slightly more complex detection loop logic.
  - Mitigation: Isolate playlist selection into a small helper function and keep the rest of the loop unchanged.
- Risk: Config shape grows with additional fields.
  - Mitigation: Add new fields with sensible defaults and avoid removing existing ones so legacy configs continue to work.

## Migration Plan
- New fields are added to configuration and AppState with defaults.
- If a config has no explicit goal playlist but has a `selected_music_index`, the system will treat that as a single-track playlist.
- No manual user migration is required; users may optionally add more tracks to the playlist through the updated Library UI.

## Open Questions
- None identified at this time.
