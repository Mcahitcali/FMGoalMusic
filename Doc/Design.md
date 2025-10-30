### Filename and Display Name Rules

- **ASCII-only filenames**: When a user adds audio, the resulting WAV is saved under `config/musics/` using an ASCII slug.
- **Slug rules**:
  - Turkish letters mapped: ı→i, İ→I, ğ→g, Ğ→G, ş→s, Ş→S
  - General diacritics removed via Unicode decomposition (e.g., ö→o, ü→u, ç→c)
  - Spaces and non-alphanumerics → underscores
  - Collapses multiple underscores, trims edges
- **Display name**: Derived from the final WAV filename stem (no extension). What you see matches the on-disk name.

### Fonts

- Primary text is ASCII after slugging. For broader UI text (status/messages), the app may load a system font (e.g., Arial/Helvetica) at startup to improve glyph coverage on macOS.
# FM Goal Musics – Design Specification

## Design Philosophy
FM Goal Musics follows a **minimal, functional, performance-first** design approach. The interface prioritizes clarity, speed, and reliability over visual flourish. Both CLI and GUI versions maintain consistent visual language while respecting platform conventions.

## Design Principles

### 1. Performance First
- **Lightweight UI** – Minimal memory footprint, 60 FPS responsiveness
- **Clear Feedback** – Instant visual response to user actions
- **No Blocking Operations** – All heavy tasks run in background threads
- **Efficient Rendering** – Immediate mode GUI (egui) with optimized draw calls

### 2. Clarity & Simplicity
- **Single Purpose Per Screen** – Each view focuses on one task
- **Progressive Disclosure** – Advanced options hidden until needed
- **Clear Labeling** – Descriptive button text and input labels
- **Visual Hierarchy** – Important actions prominent, secondary actions subdued

### 3. Consistency
- **Unified Color Language** – Same colors mean same things across all views
- **Predictable Behavior** – Actions produce expected results
- **Platform Conventions** – Respect macOS/Windows UI patterns
- **Shared Terminology** – Same terms in CLI and GUI

### 4. Accessibility
- **High Contrast** – Clear visual separation between elements
- **Readable Text** – Minimum 14px font sizes
- **Status Indicators** – Color + icon + text for colorblind accessibility
- **Keyboard Navigation** – Full keyboard control in CLI, shortcuts in GUI

## Color Palette

### Primary Colors
```
Background (Dark):   #1e1e1e (RGB: 30, 30, 30)
Background (Light):  #f5f5f5 (RGB: 245, 245, 245)
Surface:             #2d2d2d (RGB: 45, 45, 45)
```

### Semantic Colors
```
Success (Green):     #4caf50 (RGB: 76, 175, 80)
  - Running state
  - Successful operations
  - Valid input

Warning (Yellow):    #ffb74d (RGB: 255, 183, 77)
  - Paused state
  - Attention needed
  - Non-critical alerts

Error (Red):         #f44336 (RGB: 244, 67, 54)
  - Stopped state
  - Error messages
  - Invalid input

Info (Blue):         #2196f3 (RGB: 33, 150, 243)
  - Detection counter
  - Information messages
  - Neutral actions
```

### Text Colors
```
Primary Text:        #ffffff (RGB: 255, 255, 255) - Main content
Secondary Text:      #b0b0b0 (RGB: 176, 176, 176) - Labels, descriptions
Disabled Text:       #6e6e6e (RGB: 110, 110, 110) - Inactive elements
Accent Text:         #90caf9 (RGB: 144, 202, 249) - Links, highlights
```

### UI Element Colors
```
Button Primary:      #2196f3 (RGB: 33, 150, 243)
Button Hover:        #1976d2 (RGB: 25, 118, 210)
Button Active:       #0d47a1 (RGB: 13, 71, 161)
Button Disabled:     #424242 (RGB: 66, 66, 66)

Input Background:    #3a3a3a (RGB: 58, 58, 58)
Input Border:        #5a5a5a (RGB: 90, 90, 90)
Input Focus:         #2196f3 (RGB: 33, 150, 243)
Input Error:         #f44336 (RGB: 244, 67, 54)

Selection BG:        #2196f3 (RGB: 33, 150, 243)
Selection Text:      #ffffff (RGB: 255, 255, 255)
```

## Typography

### Font Families
```
Primary Font:        System Default
  - macOS: SF Pro Text, SF Pro Display
  - Windows: Segoe UI
  - Linux: Ubuntu, sans-serif

Monospace Font:      System Monospace
  - macOS: SF Mono, Menlo
  - Windows: Consolas, Courier New
  - Linux: Ubuntu Mono, Courier
```

### Font Sizes
```
Heading Large:       24px (1.5rem) - Section titles
Heading Medium:      20px (1.25rem) - Subsection titles
Heading Small:       16px (1rem) - Group labels

Body Large:          16px (1rem) - Primary content
Body Medium:         14px (0.875rem) - Standard text
Body Small:          12px (0.75rem) - Secondary info, captions

Monospace:           13px (0.8125rem) - Code, logs, coordinates
```

### Font Weights
```
Regular:             400 - Body text
Medium:              500 - Emphasis, labels
Semibold:            600 - Buttons, headings
Bold:                700 - Important headings
```

## Layout & Spacing

### Grid System
```
Base Unit:           8px
Small Space:         8px (1 unit)
Medium Space:        16px (2 units)
Large Space:         24px (3 units)
XLarge Space:        32px (4 units)

Container Padding:   16px
Section Margin:      24px
Element Spacing:     8px
```

### Component Dimensions
```
Button Height:       32px (Standard), 40px (Large)
Input Height:        32px
List Item Height:    36px
Header Height:       48px
Sidebar Width:       280px
Min Window Width:    600px
Min Window Height:   500px
```

### Border Radius
```
Small:               4px - Inputs, small buttons
Medium:              8px - Cards, panels
Large:               12px - Modals, overlays
```

## GUI Components

### Main Window Layout
```
┌─────────────────────────────────────────┐
│ Title Bar                               │
├─────────────────────────────────────────┤
│                                         │
│  Music Management Section               │
│  ├─ Music List (scrollable)            │
│  ├─ Add/Remove Buttons                 │
│  └─ Selected Music Display             │
│                                         │
├─────────────────────────────────────────┤
│  🔄 NEW: Team Selection Section         │
│  ├─ League Dropdown                    │
│  ├─ Team Dropdown (filtered by league) │
│  ├─ Selected Team Display              │
│  └─ Clear Selection Button             │
│                                         │
├─────────────────────────────────────────┤
│                                         │
│  Configuration Section                  │
│  ├─ Capture Region (X, Y, W, H)        │
│  ├─ Region Selector Button             │
│  ├─ OCR Threshold                      │
│  ├─ Debounce Time                      │
│  └─ Morphological Opening Toggle       │
│                                         │
├─────────────────────────────────────────┤
│                                         │
│  Process Control Section                │
│  ├─ Start/Pause/Stop Buttons           │
│  ├─ Status Indicator                   │
│  └─ Detection Counter                  │
│                                         │
└─────────────────────────────────────────┘
```

### Updated: Tabbed Layout

```
┌─────────────────────────────────────────┐
│ Title Bar                               │
├─────────────────────────────────────────┤
│ Status Bar: status message · detections │
│            · display res · window size  │
├─────────────────────────────────────────┤
│ Tab Bar: [Library | Audio | Detection | │
│          Settings | Help]               │
├─────────────────────────────────────────┤
│ Active Tab Content                      │
│  • Library: music list, add/remove      │
│  • Audio: volumes, lengths, ambiance    │
│  • Detection: team selection, controls, │
│             capture preview             │
│  • Settings: capture region, OCR,       │
│             debounce, morphology        │
│  • Help: quick tips                     │
└─────────────────────────────────────────┘
```

Tabs group related controls for clarity and responsiveness. Status bar and window metrics remain always visible at the top.

### Button Styles

#### Primary Button
```
State:      Default         Hover           Active          Disabled
BG Color:   #2196f3         #1976d2         #0d47a1         #424242
Text:       #ffffff         #ffffff         #ffffff         #6e6e6e
Border:     None            None            None            None
Shadow:     0 2px 4px       0 4px 8px       0 1px 2px       None
            rgba(0,0,0,0.2) rgba(0,0,0,0.3) rgba(0,0,0,0.1)
```

#### Secondary Button
```
State:      Default         Hover           Active          Disabled
BG Color:   #3a3a3a         #4a4a4a         #2a2a2a         #2d2d2d
Text:       #ffffff         #ffffff         #ffffff         #6e6e6e
Border:     1px #5a5a5a     1px #6a6a6a     1px #4a4a4a     1px #3d3d3d
```

#### Icon Button
```
Size:       32x32px (Standard), 40x40px (Large)
Icon Size:  16px (Standard), 20px (Large)
BG:         Transparent (hover: #3a3a3a)
```

### Status Indicators

#### Running State
```
Icon:       🟢 (Green Circle)
Text:       "Running"
Color:      #4caf50 (Success Green)
Animation:  Subtle pulse (1s cycle)
```

#### Paused State
```
Icon:       🟡 (Yellow Circle)
Text:       "Paused"
Color:      #ffb74d (Warning Yellow)
Animation:  None
```

#### Stopped State
```
Icon:       🔴 (Red Circle)
Text:       "Stopped"
Color:      #f44336 (Error Red)
Animation:  None
```

### Input Fields

#### Text Input
```
Height:         32px
Padding:        8px 12px
BG Color:       #3a3a3a
Border:         1px solid #5a5a5a
Focus Border:   2px solid #2196f3
Error Border:   2px solid #f44336
Font Size:      14px
Border Radius:  4px
```

#### Number Input
```
Same as Text Input, plus:
Width:          80px (coordinates), 120px (general)
Alignment:      Right-aligned text
Step Controls:  +/- buttons (optional)
```

### List Components

#### Music List Item
```
Height:         36px
Padding:        8px 12px
BG (Unselected): Transparent
BG (Selected):   #2196f3
BG (Hover):      #3a3a3a
Text Color:      #ffffff (unselected), #ffffff (selected)
Font Size:       14px
Border:          None
```

#### Scrollbar
```
Width:          8px
Track Color:    #2d2d2d
Thumb Color:    #5a5a5a (default), #6a6a6a (hover)
Border Radius:  4px
```

### Region Selector Overlay

#### Full-Screen Overlay
```
BG Color:       rgba(0, 0, 0, 0.7) - Semi-transparent black
Selection Box:  2px solid #f44336 (Red)
Dimension Text: 16px white text with black shadow
Cursor:         Crosshair
```

#### Selection Rectangle
```
Border:         2px solid #f44336
Fill:           rgba(244, 67, 54, 0.2) - Transparent red
Shadow:         0 0 8px rgba(244, 67, 54, 0.5)
```

## CLI Design

### Color Scheme (Terminal)
```
Success:        Green (ANSI 32)
Warning:        Yellow (ANSI 33)
Error:          Red (ANSI 31)
Info:           Blue (ANSI 34)
Highlight:      Cyan (ANSI 36)
Dim:            Gray (ANSI 90)
```

### Output Format

#### Status Messages
```
✓ Success message          (Green checkmark)
⚠ Warning message          (Yellow warning)
✗ Error message            (Red X)
ℹ Info message             (Blue info)
```

#### Progress Indicators
```
▸ Loading...               (Right arrow + animation)
● Running detection        (Bullet point)
⏸ Paused                   (Pause symbol)
```

#### Benchmark Table
```
╔═══════════════════════════════════════════════════════════════╗
║           FM Goal Musics - Latency Benchmark Report          ║
╚═══════════════════════════════════════════════════════════════╝

Sample Size: 500 iterations

┌─────────────────┬──────────┬──────────┬──────────┬──────────┐
│ Stage           │   Mean   │   p50    │   p95    │   p99    │
├─────────────────┼──────────┼──────────┼──────────┼──────────┤
│ Capture         │  12.3 ms │  11.8 ms │  15.2 ms │  18.4 ms │
│ Preprocess      │   2.1 ms │   2.0 ms │   2.8 ms │   3.5 ms │
│ OCR             │  18.5 ms │  17.2 ms │  23.1 ms │  28.6 ms │
│ Audio Trigger   │   0.8 ms │   0.7 ms │   1.2 ms │   1.8 ms │
├─────────────────┼──────────┼──────────┼──────────┼──────────┤
│ TOTAL           │  33.7 ms │  31.7 ms │  42.3 ms │  52.3 ms │
└─────────────────┴──────────┴──────────┴──────────┴──────────┘
```

## Icons & Symbols

### GUI Icons
```
➕ Add          - Add music file
🗑️ Remove       - Remove selected music
▶️ Start        - Start detection
⏸️ Pause        - Pause detection
⏹️ Stop         - Stop detection
🎯 Target       - Region selector
⚙️ Settings     - Configuration
ℹ️ Info         - Information
🎵 Music        - Audio file
🟢 Running      - Active state
🟡 Paused       - Paused state
🔴 Stopped      - Inactive state
```

### CLI Symbols
```
✓  Success checkmark
✗  Error cross
⚠  Warning triangle
ℹ  Information
▸  Right arrow (progress)
●  Bullet point (list)
⏸  Pause symbol
```

## Animation & Transitions

### GUI Animations
```
Button Hover:       150ms ease-in-out
State Change:       200ms ease-in-out
Modal Fade In:      250ms ease-out
List Selection:     100ms ease-in-out
Overlay Appear:     300ms fade
```

### Animation Guidelines
- Keep animations subtle and functional
- Disable animations on low-performance systems
- No animations during active detection (performance priority)
- Use animations only for state changes and feedback

## Responsive Behavior

### Window Resizing
```
Min Width:      600px
Min Height:     500px
Max Width:      None (expands with content)
Max Height:     None (expands with content)

Behavior:
- Music list expands vertically
- Configuration section maintains fixed height
- Horizontal elements stack at narrow widths
```

### Scaling
```
DPI Scaling:    Automatic (respects system settings)
Retina:         2x rendering on high-DPI displays
Text Scaling:   Follows system font size preferences
```

## Accessibility Features

### Visual Accessibility
- High contrast mode support
- Colorblind-safe palette (use icons + text, not color alone)
- Minimum text size 12px
- Clear focus indicators (2px blue outline)

### Interaction Accessibility
- Keyboard navigation in GUI
- Screen reader compatible labels
- Tooltip descriptions on hover
- Clear error messages with solutions

## Platform-Specific Design

### macOS
```
Window Style:       Native macOS title bar
Buttons:            Rounded corners (8px)
Shadows:            Subtle depth (0 2px 8px rgba(0,0,0,0.15))
Scrollbars:         Overlay style (hidden when not scrolling)
```

### Windows
```
Window Style:       Standard Windows chrome
Buttons:            Squared corners (4px)
Shadows:            Pronounced depth (0 4px 12px rgba(0,0,0,0.25))
Scrollbars:         Always visible
```

### Linux
```
Window Style:       GTK/Qt compliant
Buttons:            Follows desktop environment theme
Shadows:            Minimal (0 2px 4px rgba(0,0,0,0.1))
Scrollbars:         Desktop environment default
```

## Brand Identity

### Application Name
```
Full Name:      FM Goal Musics
Short Name:     FM Goal Musics
Abbreviation:   FGM (internal use)
```

### Tagline
```
"Instant goal celebrations for Football Manager"
```

### Visual Identity
- No custom logo (uses system icons and emoji)
- Focus on functionality over branding
- Clean, technical aesthetic
- Performance-oriented presentation

---

*Last Updated: 2025-10-29*
*Version: 1.0*
