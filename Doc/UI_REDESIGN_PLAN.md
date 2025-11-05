# FM Goal Musics - UI Redesign Plan

## Overview

This document outlines the comprehensive UI redesign for FM Goal Musics application. The goal is to transform the technical-looking interface into a modern, user-friendly application with a football/sport theme.

## Design Principles

1. **Football Theme**: Green pitch colors, stadium aesthetic, gold accents
2. **Rich & Detailed**: Feature-rich with visual elements, not minimal
3. **Card-Based Layout**: Visual grouping using panels and cards
4. **Progressive Disclosure**: Basic settings visible, advanced settings collapsed
5. **Logical Organization**: Related controls grouped together
6. **Better UX**: Clear, understandable text and intuitive placement

## Design System

### Color Palette

**Primary Colors:**
- `PITCH_GREEN` (#1B5E20) - Dark forest green, main accent color
- `PITCH_LIGHT` (#2E7D32) - Lighter green for hover states
- `GOAL_GOLD` (#FFD700) - Gold for important actions and highlights
- `STADIUM_DARK` (#1A1A1D) - Deep charcoal for backgrounds
- `WHITE_LINE` (#FFFFFF) - Pure white for text and lines

**Secondary Colors:**
- `GRASS_ACCENT` (#4CAF50) - Bright green for success states
- `SHADOW` (#0D0D0F) - Deep shadow for depth
- `SCOREBOARD_BG` (#2C2C30) - Medium gray for panels

**Semantic Colors:**
- `SUCCESS` = GRASS_ACCENT
- `WARNING` (#FFA726) - Orange for warnings
- `ERROR` (#EF5350) - Red for errors
- `INFO` (#42A5F5) - Blue for information

### Spacing System

- `SPACING_XS` = 4.0
- `SPACING_SM` = 8.0
- `SPACING_MD` = 12.0
- `SPACING_LG` = 16.0
- `SPACING_XL` = 24.0

### Component Styles

- **Card Frame**: Rounded corners (rounding: 8.0), subtle shadows, SCOREBOARD_BG fill
- **Buttons**: PITCH_GREEN fill, PITCH_LIGHT hover, white text, rounded
- **Headings**: PITCH_GREEN text, bold, larger font
- **Status Indicators**: Colored dots with labels

## Implementation Phases

### Phase 1: Reorganization (6 commits)

**Goal**: Fix organizational issues without changing visual design yet

#### 1.1 Move Capture Preview to Settings Tab
**File**: `src/gui/views/settings.rs`, `src/gui/views/team.rs`
**Changes**:
- Remove capture preview section from team.rs (lines 125-159)
- Add capture preview to settings.rs with capture region controls
- Update method signatures if needed

**Commit**: `refactor: move capture preview from team tab to settings tab`

#### 1.2 Reorganize Settings into Grouped Sections
**File**: `src/gui/views/settings.rs`
**Changes**:
- Group 1: "Display & Capture" (Monitor, Capture Region, Capture Preview)
- Group 2: "Detection Settings" (OCR Threshold, Debounce, Morphological Opening)
- Group 3: "Audio Settings" (Volume Controls, Sound Length)
- Group 4: "Updates" (Auto-check, Manual check)
- Group 5: "Advanced Settings" (Collapsible, default closed)

**Commit**: `refactor: reorganize settings tab into logical sections`

#### 1.3 Improve Library Tab Layout
**File**: `src/gui/views/library.rs`
**Changes**:
- Reorder buttons: Add → Preview → Remove
- Add preview button for ambiance sounds
- Group music files and ambiance into separate visual sections

**Commit**: `refactor: improve library tab button ordering and layout`

#### 1.4 Make Add Team UI Collapsible
**File**: `src/gui/mod.rs` (render_add_team_ui method)
**Changes**:
- Wrap "Add New Team" section in collapsing header
- Default to collapsed state
- Keep functionality unchanged

**Commit**: `refactor: make add team ui collapsible by default`

#### 1.5 Reorder Help Tab Sections
**File**: `src/gui/views/help.rs`
**Changes**:
- Move "Quick Start" to first position
- Set "Quick Start" to open by default
- Order: Quick Start → Library → Team Selection → Settings → teams.json → Troubleshooting

**Commit**: `refactor: reorder help sections with quick start first`

#### 1.6 Simplify Main Window Status Bar
**File**: `src/gui/mod.rs`
**Changes**:
- Replace text status with colored dot indicator
- Green dot = running, Gray dot = stopped
- Move technical info (OCR, capture stats) to footer or tooltip
- Keep clean, simple status bar

**Commit**: `refactor: simplify status bar with visual indicators`

---

### Phase 2: Design System Foundation (2 commits)

**Goal**: Create centralized theme system

#### 2.1 Create Theme Module
**File**: `src/gui/theme.rs` (NEW)
**Changes**:
- Define color constants (PITCH_GREEN, GOAL_GOLD, etc.)
- Define spacing constants
- Create `apply_theme()` function for egui context
- Create component styling functions:
  - `styled_button()` - Returns styled button response
  - `styled_primary_button()` - Gold accent button
  - `styled_heading()` - Styled heading text
  - `card_frame()` - Returns Frame with card styling
  - `section_frame()` - Returns Frame for sections
  - `status_dot()` - Renders colored status indicator

**Commit**: `feat: add football-themed design system`

#### 2.2 Integrate Theme System
**File**: `src/gui/mod.rs`
**Changes**:
- Add `mod theme;` and `use theme::*;`
- Call `theme::apply_theme(ctx)` in update() method
- Replace hardcoded colors in header with theme colors
- Update egui::Context styling with football colors

**Commit**: `feat: integrate theme system into main application`

---

### Phase 3: Visual Design Application (7 commits)

**Goal**: Apply football theme to all UI elements

#### 3.1 Redesign Main Window Header & Status
**File**: `src/gui/mod.rs`
**Changes**:
- Apply card_frame() to header section
- Style Start/Stop button with styled_primary_button()
- Add football-themed app title with icon
- Style status bar with theme colors
- Add subtle animations to status dot

**Commit**: `feat: apply football theme to main window header`

#### 3.2 Redesign Library Tab
**File**: `src/gui/views/library.rs`
**Changes**:
- Wrap sections in card_frame()
- Style all buttons with styled_button()/styled_primary_button()
- Apply themed headings with styled_heading()
- Add visual separation between music and ambiance sections
- Style file list with alternating row colors

**Commit**: `feat: apply football theme to library tab`

#### 3.3 Redesign Team Selection Tab
**File**: `src/gui/views/team.rs`
**Changes**:
- Wrap team selection in card_frame()
- Style dropdowns with theme colors
- Apply themed heading
- Style "Clear Selection" button
- Add visual feedback for selected team

**Commit**: `feat: apply football theme to team selection tab`

#### 3.4 Redesign Settings Tab (MOST IMPORTANT)
**File**: `src/gui/views/settings.rs`
**Changes**:
- Wrap each section in card_frame() with visual spacing
- Apply styled_heading() to all section headers
- Style all buttons with theme functions
- Add icons to section headers (🖥️ Display, 🔍 Detection, 🔊 Audio, 🔄 Updates, ⚙️ Advanced)
- Style sliders and drag values with theme colors
- Add subtle hover effects
- Make capture preview prominent with border

**Commit**: `feat: apply football theme to settings tab with card layout`

#### 3.5 Redesign Help Tab
**File**: `src/gui/views/help.rs`
**Changes**:
- Style collapsing headers with theme colors
- Wrap content in card frames
- Apply themed headings
- Add football-themed icons to sections
- Style code examples with monospace font on SCOREBOARD_BG
- Improve text readability with proper spacing

**Commit**: `feat: apply football theme to help tab`

#### 3.6 Add Animations & Polish
**File**: All view files
**Changes**:
- Add hover animations to buttons (scale, color transition)
- Add fade-in animations for collapsible sections
- Add celebration effect when goal is detected (optional)
- Add smooth transitions for status changes
- Polish spacing and alignment throughout

**Commit**: `feat: add animations and interaction polish`

#### 3.7 Improve Responsive Layout
**File**: All view files
**Changes**:
- Implement two-column layout for settings on wide windows
- Add responsive breakpoints
- Ensure all elements scale properly on smaller windows
- Test minimum window size requirements
- Add scrolling where needed

**Commit**: `feat: improve responsive layout across all tabs`

---

### Phase 4: Testing & Refinement (4 commits)

**Goal**: Ensure quality and prepare for release

#### 4.1 Manual Testing & Bug Fixes
**Changes**:
- Test all UI interactions on macOS
- Test responsive layout at different window sizes
- Test theme consistency across all tabs
- Fix any visual bugs or alignment issues
- Test with light/dark system theme

**Commit**: `fix: address ui bugs and visual inconsistencies`

#### 4.2 Update Tests
**Files**: `src/wizard/steps.rs`, `src/wizard/state.rs`, test files
**Changes**:
- Update any tests broken by UI changes
- Add new tests if needed
- Ensure all tests pass
- Run `cargo test`

**Commit**: `test: update tests for ui redesign`

#### 4.3 Update Documentation
**Files**: `Doc/FEATURES.md`, `Doc/UserGuide.md`, README.md
**Changes**:
- Update screenshots (if applicable)
- Document new UI features
- Update user guide with new section locations
- Add design system documentation

**Commit**: `docs: update documentation for ui redesign`

#### 4.4 Version Bump & Release Prep
**Files**: `Cargo.toml`, `CHANGELOG.md` (if exists)
**Changes**:
- Bump version to 0.3.0
- Update changelog with UI redesign notes
- Prepare release notes
- Merge to main, create tag v0.3.0

**Commit**: `chore: bump version to 0.3.0 for ui redesign release`

---

## Testing Checklist

- [ ] All tabs render correctly
- [ ] All buttons respond to clicks
- [ ] Theme colors applied consistently
- [ ] Status indicators work correctly
- [ ] Capture preview displays properly in Settings tab
- [ ] Team selection works with new theme
- [ ] Library file management works
- [ ] Settings are saved and loaded correctly
- [ ] Help sections are properly ordered
- [ ] Collapsible sections work smoothly
- [ ] Window resizing works properly
- [ ] No visual glitches or alignment issues
- [ ] Text is readable with new colors
- [ ] Hover effects work correctly
- [ ] All cargo tests pass

## Success Criteria

1. ✅ UI looks modern and friendly, not technical
2. ✅ Football theme is consistent throughout
3. ✅ Settings tab uses visual card layout
4. ✅ All sections are logically organized
5. ✅ User experience is intuitive
6. ✅ Text is clear and understandable
7. ✅ No functionality is broken
8. ✅ All tests pass
9. ✅ Code is well-documented

## Notes

- Can create new tabs/sections if needed during implementation
- Can change any text for better UX
- Should maintain backward compatibility with config files
- Should not break existing functionality
- Focus on user-friendliness while keeping all features intact
