## ADDED Requirements

### Requirement: Goal Celebration Playlist Selection
The GUI SHALL allow users to build a goal celebration playlist from the existing music library.

#### Scenario: Add track to playlist
- **WHEN** the user toggles a track as part of the goal playlist in the Library tab
- **THEN** the track SHALL be added to the stored goal playlist
- AND the Library UI SHALL visually indicate that the track is in the playlist

#### Scenario: Remove track from playlist
- **WHEN** the user untoggles a track from the goal playlist
- **THEN** the track SHALL be removed from the stored goal playlist
- AND the Library UI SHALL update its visual indication accordingly

#### Scenario: Playlist summary
- **WHEN** one or more tracks are in the goal playlist
- **THEN** the dashboard or Library view SHALL show a concise summary of the playlist (for example, the number of selected tracks and/or representative track names)

### Requirement: Random Non-Repeating Goal Playback
The system SHALL play a random track from the goal playlist when a goal is detected, without immediately repeating the last played track when multiple tracks are available.

#### Scenario: Multiple tracks, no immediate repeat
- **GIVEN** a goal playlist with at least two tracks
- AND a track was played for the previous goal
- **WHEN** the next goal is detected
- **THEN** the system SHALL choose a random track from the playlist that is not the last played track
- AND play that track as the goal celebration

#### Scenario: Single track playlist
- **GIVEN** a goal playlist with exactly one track
- **WHEN** consecutive goals are detected
- **THEN** the same track MAY be played for each goal (no no-repeat constraint)

#### Scenario: Legacy single selection fallback
- **GIVEN** a configuration where no explicit goal playlist is stored
- AND a single legacy selected track exists
- **WHEN** a goal is detected
- **THEN** the system SHALL treat the legacy selection as a single-track playlist and play that track
