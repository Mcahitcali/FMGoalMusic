# Change: Update GUI Dashboard to Dark Red Football Theme

## Why
The current GUI is functional but lacks a cohesive modern visual language. We want a dashboard that matches the shared design (sidebar + cards) with a dark palette and red primary, improving clarity, navigation, and consistency.

## What Changes
- Adopt a dark-red theme inspired by the provided HTML/screenshot
- Update `theme.rs` tokens (primary, surfaces, accents) to match the color palette
- Redesign the Dashboard screen:
  - Sidebar with branding, Start/Stop Monitoring button, and navigation items
  - Header with "Dashboard" title and a live status chip (Running/Stopped)
  - Team Selection callout with "Configure" action
  - Two cards: "Goal Music" and "Other Music" with browse buttons
- Wire actions to existing controller/state (start/stop monitoring, navigation between tabs)
- Keep GPUI + gpui-component architecture (no framework change)
- Minor responsive behavior inside the GPUI layout rules

## Impact
- Affected specs: gui
- Affected code: `src/gui/theme.rs`, `src/gui/view.rs`, `src/gui/state.rs`
- Non-breaking: runtime behavior remains the same; UI structure and styling are updated
