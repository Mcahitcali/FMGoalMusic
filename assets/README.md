# Assets Directory

This directory contains externalized data files that are embedded into the application binary at compile time using `include_str!`.

## Structure

```
assets/
├── i18n/           # Internationalization files
│   ├── en.json     # English detection phrases
│   ├── tr.json     # Turkish (Türkçe)
│   ├── es.json     # Spanish (Español)
│   ├── fr.json     # French (Français)
│   ├── de.json     # German (Deutsch)
│   ├── it.json     # Italian (Italiano)
│   └── pt.json     # Portuguese (Português)
└── wizard/         # First-run wizard data
    └── steps.json  # Wizard step definitions
```

## I18n Files

Detection phrase files for different languages. Each file contains:

- `language`: Full language name
- `code`: ISO 639-1 language code
- `detection`: Phrases for detecting game events
  - `goal_phrases`: Phrases for goal detection (e.g., "GOAL!", "GOL!")
  - `kickoff_phrases`: Phrases for kickoff detection (e.g., "Kick Off")
  - `match_end_phrases`: Phrases for match end detection (e.g., "Full Time")

### Example (en.json)

```json
{
  "language": "English",
  "code": "en",
  "detection": {
    "goal_phrases": ["GOAL!", "Goal!"],
    "kickoff_phrases": ["Kick Off", "Kick-Off"],
    "match_end_phrases": ["Full Time", "FT"]
  }
}
```

## Adding New Languages

To add a new language:

1. Create a new JSON file in `assets/i18n/` (e.g., `nl.json` for Dutch)
2. Follow the structure of existing files
3. Add the language to `src/detection/i18n.rs` `Language` enum
4. Add the embedded file constant in `src/detection/i18n_loader.rs`
5. Update the `load_phrases()` function to handle the new language

## Wizard Steps

The `wizard/steps.json` file defines the steps in the first-run wizard:

- `id`: Unique identifier for the step
- `title`: Display title
- `description`: Step description
- `skippable`: Whether the step can be skipped

## Embedding

All assets are embedded at compile time using Rust's `include_str!` macro, so:

- ✅ No runtime file I/O required
- ✅ Assets are part of the binary
- ✅ No external files to distribute
- ✅ Fast loading (already in memory)
- ✅ Fallback to hardcoded values if loading fails

## Modifying Assets

After modifying any JSON file, rebuild the application:

```bash
cargo build --release
```

The changes will be embedded in the new binary.

## Validation

All JSON files are validated at compile time. If a file contains invalid JSON, the build will fail.

Run tests to validate all assets:

```bash
cargo test
```
