# FM Goal Musics — GPUI Design Architecture

Version: 2.0 (GPUI refresh)\
Status: In progress – source of truth for every GUI change touching `src/gui/**`.

This document replaces the previous egui-oriented design guide. It reflects the gpui/gpui-component rewrite described in `ARCHITECTURE.md` and `openspec/changes/refactor-gui-to-gpui`. Treat it as the contract for layout, interaction, and theming.

---

## 1. Design Goals

1. **Instant feedback for real-time detection** – UI must acknowledge every action within a frame (≤16 ms) and never block the UI thread. All slow work (`rfd`, disk, audio preload) runs via `cx.defer_in`.
2. **Operational clarity** – Tabs are scoped to single responsibilities (Library, Team, Detection, Settings, Help). Controls live inside contextual panels so users never guess where a setting belongs.
3. **Hardware-friendly visuals** – Palette avoids pure dark/light extremes, emphasizing a mid-contrast “slate dusk” look that stays readable on SDR and HDR monitors.
4. **Spec-driven components** – Only gpui-component widgets (`Button`, `Slider`, `Switch`, `List`, `Stack`, etc.) are allowed unless a requirement proves otherwise. Custom primitives (`div`, `label`) are only used to compose these widgets.
5. **Backend stability** – The GUI orchestrates `GuiController` and state structures but never mutates detection/audio internals directly. Follow the MVU/Event flow described in `ARCHITECTURE.md`.

## 2. Layout System

```
┌──────────────────────────────────────────────────────────────────────────┐
│ Header (logo, detection controls, status, session summary, help link)    │
├──────────────────────────────────────────────────────────────────────────┤
│ Tab Bar (Library · Team · Detection · Settings · Help)                   │
├──────────────────────────────────────────────────────────────────────────┤
│ Content Area (grid-based panels, per-tab layout)                         │
└──────────────────────────────────────────────────────────────────────────┘
 Footer (capture region summary + hotkeys) – optional on narrow widths.
```

- **Header**: split layout with `flex` – left column shows product name + secondary status text; right column contains detection CTA(s) and quick links.
- **Tab Bar**: uses `gpui_component::tab::TabBar` with underline style. Tab labels include emoji prefixes (🎵, ⚽, 🛰, ⚙️, ℹ️) for instant recognition.
- **Content Grid**: default to two columns (list vs. inspector) ≥1280 px; collapse to stacked panels below 960 px.
- **Panels**: `div().border_1().rounded_lg().p_3()`; panel titles use `text_sm().uppercase().tracking_wider()` for clarity.
- **Footer**: status chips describing capture region `[x, y, w, h]`, selected monitor, and keyboard shortcuts (Cmd+1 pause, Cmd+Shift+R region tool).

## 3. Tab Specifications

### 3.1 Library

- **Purpose**: Manage celebration music & ambiance clips, preview audio, and map shortcuts.
- **Left Panel – Collections**
  - Two collapsible sections: `Celebration Tracks` (list) and `Ambiance Clip` (single-slot card, because backend stores one ambiance path today).
  - Each row uses `Button::ghost()` in list mode with `selected()` highlight for the active entry. The label shows display name + optional shortcut (e.g., `I Will Survive · ⇧1`).
  - Actions above the list:
    - `Add` opens native file dialog (`rfd::FileDialog::new().pick_files()`), supports multi-select, auto-converts to WAV via controller.
    - `Remove`, `Clear Selection`, `Preview` (or `Stop Preview`), `Reveal in Finder/Explorer`.
    - Buttons disabled when no selection is active.
- **Right Panel – Inspector**
  - Shows waveform placeholder, metadata (duration, sample rate if available), and `Set Shortcut` + `Play Preview`.
  - Ambiance subsection mirrors actions but is limited to a single file slot; includes `Switch` for enable/disable + `Slider` for mix volume.
  - Preview uses cached audio via `AudioManager` inside `PreviewSound`; stop preview resets state.
- **Status messaging**: Use `status_text` to display success/failure; never rely on modal alerts.

### 3.2 Team Selection

- **Left Sidebar**: League list (scrollable). Each league entry is a `Button::ghost().selected(...)`. Searching/filtering uses an `Input` at top.
- **Main Grid**: Responsive 3–5 columns of team cards. Each card shows team display name + key variations (chip list). Selecting a team updates controller + inspector panel.
- **Custom Team Drawer**:
  - Expandable panel toggled via `Button::ghost("Add Custom Team")`.
  - Fields: `Team Name`, `League` (dropdown when existing, input when new), `Variations` (multi-line).
  - Submit button validates via controller (`add_custom_team`), errors surface inline under input.

### 3.3 Detection

- **Capture Overview**: Show monitor identifier, capture region `[x, y, width, height]`, and screenshot placeholder.
- **Controls**:
  - Buttons for `Select Region`, `Reset Region`, `Monitor ▾`.
  - Sliders for `OCR Threshold` (Auto label when 0), `Debounce`, `Morphological Opening` switch.
  - `Detection Status` card summarizing `ProcessState`, detection count, last trigger timestamp.
- **Actions**: `Start`, `Pause`, `Stop` button group. Buttons dispatch through controller once backend wiring lands.

### 3.4 Settings

- **Audio Section**: `Slider` for music/ambiance volumes, `Slider` for playback length (in seconds). Display value as `XX s` or `XX %`.
- **Updates Section**: Switch for auto update checks, `Check Now` button.
- **Diagnostics Section**: “Open Logs” and “Reveal Config” actions using `open::that`.

### 3.5 Help

- Cards for “Quick Start”, “Keyboard Shortcuts”, “Troubleshooting”, “Support Links”.
- Each card uses `Button::ghost()` rows linking to docs (`open::that`).

## 4. Component Guidelines

| Intent                 | Component                                             | Notes |
|------------------------|-------------------------------------------------------|-------|
| Primary actions        | `Button::primary()`                                   | Use `Button::danger()` for destructive remove/reset. |
| Secondary/list rows    | `Button::ghost().list_item(true)`                     | Always provide stable IDs via tuples `("music", idx)`. |
| Toggles                | `gpui_component::switch::Switch`                      | Reflect controller state immediately; disable during blocking ops. |
| Numeric inputs         | `gpui_component::slider::Slider`                      | For ms/percent/time values; show helper text with formatted values. |
| Text inputs            | `gpui_component::input::Input`                        | Use `.cleanable(true)` and focus handling from gpui. |
| Chips/badges           | `gpui_component::badge::Badge` or `div()` with `text-xs`. | Use for status (Running/Paused/Stopped). |
| Lists                  | Compose with `div().flex().flex_col()` or use `List` once stabilized. |

General rules:
- IDs must be deterministic & stable across frames (`Button::new(("team", idx))`).
- Avoid manual hover styling; rely on theme tokens.
- All callbacks route through `cx.listener` and call controller methods; never mutate controller state directly without using these APIs.

## 5. Theming

Palette aims for mid-tone slate (neither full dark nor light).

| Token                    | Hex      | Usage                                      |
|--------------------------|----------|--------------------------------------------|
| Background               | #262c34  | Window base, body background               |
| Surface                  | #2f3540  | Panels, cards                              |
| Elevated Surface         | #383f4d  | Popovers, inspectors                       |
| Foreground               | #f5f7fb  | Primary text                               |
| Muted Foreground         | #b7bfcd  | Secondary text                             |
| Border                   | #3d4452  | Panel borders, separators                   |
| Primary                  | #4c7cf4  | CTAs, selection, slider accent             |
| Primary Hover            | #5d8cff  | Hover states                               |
| Primary Active           | #365ed1  | Active/pressed                             |
| Accent                   | #f48c4c  | Highlight chips, warnings                   |
| Success                  | #55b685  | Running status                             |
| Warning                  | #f6c343  | Attention chips                            |
| Danger                   | #f05d70  | Errors, destructive buttons                |
| Tab Bar Background       | #2b303a  | Tab strip                                  |
| Tab Active               | #36404e  | Active tab                                 |
| Tab Inactive             | #a9b2c4  | Tab labels                                 |

Implementation notes:
- Use `gpui::rgb` + `ActiveTheme` mapping (see `src/gui/theme.rs`).
- Text must maintain ≥4.5:1 contrast; primary (#f5f7fb) on surface (#2f3540) is 5:1.
- Avoid translucent overlays unless absolutely necessary; prefer solid backgrounds for clarity.

## 6. Typography & Spacing

- Font stack: rely on platform defaults (SF Pro, Segoe UI, Ubuntu). `gpui_component` uses system fonts; no custom font loading required.
- Sizes: `text_xl` for headers (~20 px), `text_lg` (~18 px) for panel titles, `text_md` (~16 px) for body, `text_sm` (~14 px) for helper text.
- Spacing scale: multiples of 4 px. Default panel padding `p_3` (12 px), gap between controls `gap_2` (8 px).

## 7. Interaction Rules

1. **File dialogs**: always dispatch with `cx.defer_in(window, move |this, window, cx| { … })` to avoid blocking the render loop.
2. **Controller calls**: wrap in `if let Err(err) = controller.action(...) { status_text = err.into(); } else { refresh_status(); }`.
3. **Selections**: clicking a list row sets controller selection and updates `status_text`. Use `.selected(selected_index == Some(idx))`.
4. **Preview audio**: toggling preview on launches (or stops) `AudioManager`. Only one preview (music or ambiance) may play at a time.
5. **Async operations**: for long tasks (update check, add music conversions) show ephemeral status text (“Converting ‘song.mp3’…”).

## 8. Accessibility

- Focus states: ensure `Focusable` implementations and `focus_handle` usage. Buttons auto-handle focus outlines; avoid removing them.
- Keyboard shortcuts: document in Help tab and render helper text near relevant controls.
- Colorblind safety: pair color-coded chips with labels (e.g., `Running` text + green dot).
- Minimum touch target 40×40 px for all interactive elements.

## 9. Responsiveness

- Breakpoints: `lg ≥ 1280px` (two-column layout), `md 960–1279px` (single column + collapsible inspector), `sm < 960px` (stacked cards).
- Tab bar wraps to multiple lines if width < 720 px; maintain readability by keeping emoji + short labels.

## 10. Implementation Checklist

- [ ] Theme installed via `theme::install(cx)` inside `App::run`.
- [ ] Main view separated into helper methods per section (`render_library_tab`, `render_team_tab`, etc.).
- [ ] All list selections highlight even when unfocused.
- [ ] No usage of `webview` or custom GPU components; rely solely on gpui-component building blocks.
- [ ] Status text updated after every controller mutation.
- [ ] `cargo fmt` and `cargo clippy` remain clean.

---

Adhere to this document when implementing or reviewing any GUI code. If a change requires deviating from the rules above, update this document first (spec-first workflow).
