# Team Selection Feature

## Feature Overview
Play goal sound only for a user-selected team. The application will detect "GOAL FOR [team_name]" text and match it against the selected team's variations before playing the celebration sound.

## Problem Statement
Currently, the application plays celebration sound for every goal scored in the game, regardless of which team scored. Users want to celebrate only when their selected team scores a goal.

## Solution
1. **Team Database**: JSON file containing leagues, teams, display names, and name variations
2. **Team Selection UI**: Allow users to select their favorite team from the GUI
3. **Team Name Matching**: Extract team name from "GOAL FOR [team_name]" and match against selected team's variations
4. **Conditional Playback**: Play sound only if detected team matches selected team

## Technical Requirements

### JSON Structure
```json
{
  "Premier League": {
    "manchester_united": {
      "display_name": "Manchester Utd",
      "variations": [
        "Man United",
        "Manchester Utd",
        "Manchester United FC"
      ]
    },
    "team_key": {
      "display_name": "Display Name",
      "variations": ["variation1", "variation2"]
    }
  },
  "Other League": {
    "team_key": {
      "display_name": "Display Name",
      "variations": ["variation1", "variation2"]
    }
  }
}
```

### Detection Pattern
- **Current detection**: "GOAL FOR" text
- **New detection**: "GOAL FOR [team_name]" where team_name is extracted and matched

### Matching Strategy (Implemented)
- Normalize to lowercase ASCII and collapse spaces
- Match if any variation equals the detected normalized string
- Or, token-subset match: all tokens in a variation must be present in the detected tokens
  - Example: Detected `"FC. INTERNAZIONALE MILANO!"` → normalized `"fc internazionale milano"`
    - Variation `"FC Internazionale"` matches via token-subset

### Configuration Changes
Add to `config.json`:
```json
{
  "selected_team": {
    "league": "Premier League",
    "team_key": "manchester_united",
    "display_name": "Manchester Utd"
  }
}
```

## Implementation Plan

### Step 1: Team Data Module
**File**: `src/teams.rs`
- Create struct for team data: `TeamDatabase`, `League`, `Team`
- Load teams from JSON file
- Provide search/query methods
- Parse and normalize team variations for matching

### Step 2: Enhanced OCR Detection
**File**: `src/ocr.rs` (modify)
- Extract full text instead of just detecting "GOAL"
- Parse "GOAL FOR [team_name]" pattern
- Return both detection status and team name

### Step 3: Team Matching Logic
**File**: `src/team_matcher.rs`
- Implement fuzzy matching for team name variations
- Case-insensitive comparison
- Handle special characters and spaces
- Match detected team against selected team's variations

### Step 4: Configuration Update
**File**: `src/config.rs` (modify)
- Add `selected_team` field to Config struct
- Support optional team selection (backward compatible)
- Persist selected team between sessions

### Step 5: GUI Updates
**File**: `src/gui.rs` (modify)
- Add team selection dropdown/list
- League and team browsing
- Display currently selected team
- Update configuration on team selection

### Step 6: Detection Loop Update
**File**: `src/main.rs` and `src/gui_main.rs` (modify)
- Check if team is selected
- If selected: match detected team before playing sound
- If not selected: play sound for all goals (backward compatible)

## User Interface Design

### GUI Components
1. **Team Selection Panel**
   - League dropdown
   - Team list (filtered by league)
   - Search box for quick team lookup
   - Clear selection button

2. **Status Display**
   - Show currently selected team
   - Update on selection change

### CLI Support
- Add command-line argument to select team: `--team "Premier League:Manchester Utd"`
- Display selected team on startup

## Backward Compatibility
- If no team is selected: behave as before (play for all goals)
- Existing configurations without `selected_team`: continue working
- Team selection is optional feature

## Edge Cases
1. **No team selected**: Play sound for all goals
2. **Team name not found in variations**: No sound played (miss)
3. **Malformed JSON**: Load with error handling, disable team selection
4. **Multiple matches**: Use first match (prioritize exact matches)

## Testing Requirements
1. Unit tests for team matching logic
2. Unit tests for OCR text extraction
3. Integration tests with sample "GOAL FOR [team]" images
4. Test various team name formats and variations
5. Test backward compatibility (no team selected)

## Performance Considerations
- Load team database once at startup
- Cache team variations in memory
- Matching should add < 1ms to detection latency
- Use efficient string matching (avoid regex if possible)

## File Locations
- **Team Database**: `config/teams.json` or embedded in binary
- **Configuration**: Existing `config.json` with new `selected_team` field
- **Module Files**: `src/teams.rs`, `src/team_matcher.rs`

## Success Criteria
- [x] Team database loaded successfully
- [x] User can select team from GUI
- [x] Detected team name extracted from OCR
- [x] Team matching works with variations
- [x] Sound plays only for selected team
- [x] Backward compatible with existing behavior
- [x] No significant performance impact (< 1ms added latency)

## Future Enhancements
1. **Multiple Team Selection**: Support celebrating multiple teams
2. **Team Statistics**: Track goals per team
3. **Custom Sounds per Team**: Different celebration for different teams
4. **Auto-Update Team Database**: Download latest team names from server
5. **Smart Matching**: ML-based team name recognition

---

*Created: 2025-10-29*
*Status: Completed*
