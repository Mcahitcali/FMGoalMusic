## ADDED Requirements

### Requirement: Dashboard Layout and Navigation
The GUI SHALL present a dashboard screen with a left sidebar and a main content area that matches the provided layout.

#### Scenario: Sidebar structure
- WHEN the app launches
- THEN the left sidebar shows branding (app title), a Start/Stop Monitoring button, and a vertical nav with items: Dashboard, Library, Team Selection, Detection, Settings, Help
- AND the current tab is highlighted (Dashboard when landing on the dashboard)

#### Scenario: Start/Stop Monitoring
- WHEN the user presses Start Monitoring (or Stop Monitoring)
- THEN it MUST call the existing controller methods to start/stop and update the status message

#### Scenario: Navigation actions
- WHEN the user clicks "Browse Library" on any card
- THEN the active tab MUST switch to Library
- WHEN the user clicks "Configure" in Team Selection callout
- THEN the active tab MUST switch to Team Selection

#### Scenario: Status chip
- WHEN detection is running
- THEN a green status chip labeled "Running" is shown in the top-right of the dashboard header
- WHEN not running
- THEN the status chip is hidden or rendered with a neutral/disabled look

### Requirement: Theme Palette and Styling
The GUI theme SHALL use a dark-red palette inspired by the provided design.

#### Scenario: Palette tokens
- WHEN the theme is initialized
- THEN the primary color MUST be approximately `#EA2831`
- AND sidebar/background surfaces SHOULD use values near `#121212` and `#1E1E1E`
- AND success chip SHOULD use a green near `#39FF14`
- AND rounded corners MUST be applied to cards and buttons

#### Scenario: Layout sizing
- GIVEN a window around 1180×760
- THEN the dashboard MUST show a team callout card and two content cards (Goal Music, Other Music) arranged in two columns on medium widths and one column on small widths
