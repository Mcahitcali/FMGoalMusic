# Ideas & Deferred Features

This document contains features that are **intentionally deferred** or **not yet validated**. Items here are:
- Too complex for current development capacity (solo part-time dev)
- Lower priority vs bug fixes and core features
- Require user validation before committing resources
- "Nice-to-have" but not essential for product success

Items may graduate to `FEATURES.md` after user feedback indicates strong demand.

---

## Deferred (Intentionally Postponed)

These are good ideas but deferred due to complexity/capacity trade-offs.

### System Tray / Minimize to Background

**Why Deferred**: Medium-high complexity (3-4 hours), lower priority than bug fixes

**Description**:
Allow app to run in background with system tray icon. On close, prompt: "Exit app" or "Keep running in background".

**Complexity**:
- egui/eframe tray integration is tricky
- Platform-specific tray icon handling
- State management for background mode

**User Workaround**: Minimize window (works today)

**Reconsider When**: After v0.3, if users frequently request it

---

### Teams/Leagues CRUD UI

**Why Deferred**: ✅ v0.1 fixed the underlying bug (teams.json location - v0.2.1), allowing manual editing

**Description**:
Visual interface to add/edit/remove teams, leagues, and name variations without editing JSON.

**Rationale for Deferral**:
- ✅ v0.1 fixed teams.json location bug (users CAN edit it safely now - see v0.2.1)
- Power users comfortable editing JSON
- UI complexity: 6-10 hours
- Higher priority: language support (expands market more than teams UI)

**Scope if Implemented**:
- Table/list view of teams by league
- Add/edit/remove teams
- Manage name variations per team
- Merge base dataset + user overrides
- Import/export teams.json

**Time Estimate**: 6-10 hours with AI

**Reconsider When**: After v0.3, if user feedback shows friction with JSON editing

---

### Player-Specific Goal Music

**Why Deferred**: High complexity, requires player name OCR (unreliable)

**Description**:
Map individual player names to specific goal music. When Player X scores, play their custom track.

**Complexity**:
- Player name OCR is error-prone (small text, variations)
- Requires per-player configuration UI
- Fallback logic when player not recognized
- Database of players per team

**User Workaround**: Team-specific music (current feature)

**Time Estimate**: 8-12 hours with AI

**Reconsider When**: If OCR accuracy significantly improves or users heavily request it

---

### Idle Playlist (Background Music When No Match)

**Why Deferred**: Complex integration, unclear user demand

**Description**:
Play music playlist when not in a match. Auto-pause on kickoff/goal.

**Scope**:
- Local playlist support (m3u/pls files)
- Optional integration with Spotify/YouTube Music APIs
- Match state detection (is a match active?)
- Auto-pause/resume logic

**Complexity**:
- Audio mixing (playlist + match sounds)
- External API integration
- Licensing concerns (if bundling music)

**Time Estimate**: 10-15 hours with AI (local only), 20+ hours (with streaming integration)

**Reconsider When**: Strong user demand via feature requests/votes

---

## Unvalidated Ideas (Need User Feedback)

These ideas sound interesting but lack user validation. Needs evidence of demand before prioritizing.

### Dynamic Crowd Bed by Score

**Description**:
Continuous background crowd ambience that changes intensity based on match score (louder when winning, quieter when losing).

**Scope**:
- Multiple ambience layers (quiet, medium, loud)
- Crossfade transitions based on score changes
- Low CPU overhead (looping audio)

**Questions**:
- Do users want continuous background sound?
- Would this be distracting vs immersive?
- CPU impact acceptable?

**Time Estimate**: 6-8 hours with AI

**Validation Needed**: User poll, prototype feedback

---

### Team Chants During Play

**Description**:
Randomly trigger team-specific chants during match (with cooldowns to avoid spam).

**Scope**:
- Random/periodic triggers
- Cooldown logic (e.g., max once per 5 minutes)
- Team-specific audio assets
- Don't overlap with goal music

**Questions**:
- Where to source team chants? (licensing?)
- User preference: authentic chants vs generic?
- Frequency: how often is too often?

**Time Estimate**: 4-6 hours with AI (excluding asset sourcing)

**Validation Needed**: User interest survey, asset availability

---

### Goal/VAR Commentator Voice

**Description**:
Play pre-recorded commentator voice clips on goals and VAR decisions.

**Scope**:
- Support for voice packs (community-contributed)
- Trigger on goal: "GOOOAL!"
- Trigger on VAR/no-goal: "It's been ruled out!"
- Optional TTS for dynamic commentary later
- Volume ducking of background sounds

**Questions**:
- Licensing for commentator voices?
- User preference: commentary vs pure crowd sounds?
- Which languages/commentators to prioritize?

**Time Estimate**: 6-10 hours with AI (assuming assets available)

**Validation Needed**: User interest, asset licensing clarity

---

### In-App Audio Editor (Trim, Fade, Preview)

**Description**:
Built-in audio editor to trim goal music, adjust fade-in/out, preview before saving.

**Scope**:
- Waveform visualization
- Trim start/end
- Fade controls
- Real-time preview
- Save edited version

**Questions**:
- Is Audacity/external tools sufficient for users?
- Does complexity justify value?

**Time Estimate**: 15-20 hours with AI (complex GUI)

**Validation Needed**: User pain points with current workflow

---

### Social Sharing (Auto-Clip Recording)

**Description**:
Automatically record 10-second clip when goal detected, offer to share on social media.

**Scope**:
- Video capture (not just screenshot)
- Detect goal → start recording retrospectively (buffer last 5s) + record next 5s
- Simple trim UI
- Export to file or share directly

**Questions**:
- Privacy concerns (recording gameplay)?
- Video encoding performance impact?
- Sufficient demand vs complexity?

**Time Estimate**: 12-18 hours with AI

**Validation Needed**: User interest, privacy model clarity

---

### FM Workshop/Modding Integration

**Description**:
Integrate with FM modding community to auto-download team logos, kits, or sound packs.

**Scope**:
- Browse community sound packs
- One-click import
- Share your config with community

**Questions**:
- Is there existing FM workshop infrastructure?
- Legal/licensing considerations?
- Demand from modding community?

**Time Estimate**: 15-25 hours with AI

**Validation Needed**: Community engagement, API availability

---

## Adoption & Growth Ideas

Ideas to improve user acquisition and retention.

### Starter Packs (One-Click Import)

**Description**:
Pre-configured packages: "Premier League Pack", "La Liga Pack" with team sounds, logos, configurations.

**Scope**:
- ZIP files with config + audio
- Import wizard
- Community contributions

**Time Estimate**: 4-6 hours with AI (import mechanism)

**Status**: Good idea, consider for v0.4+

---

### Optional Privacy-Friendly Telemetry

**Description**:
Opt-in crash/error reporting to help prioritize bug fixes.

**Scope**:
- Crash counts, error types only (no PII)
- Opt-in during first run
- Dashboard for developer insights

**Time Estimate**: 6-8 hours with AI

**Status**: Consider for v0.4+ (after core features stable)

---

### Video Tutorials & Onboarding

**Description**:
Embedded video tutorials for setup, OCR region selection, team configuration.

**Scope**:
- Record tutorial videos
- Embed in Help tab or first-run wizard
- YouTube playlist

**Time Estimate**: 4-6 hours (recording + editing)

**Status**: Good idea, low priority vs features

---

## Quick Reference

### Why These Are Deferred

**Complexity vs Capacity**:
- Solo part-time developer with AI assistance
- 5-10 hours/week capacity
- Focus on high-impact, achievable goals

**Validation First**:
- Don't build features users don't want
- Prioritize based on actual user requests
- v0.1-v0.3 are foundation; iterate after

**Strategic Focus**:
- Fix bugs that block users (P0)
- Expand market (i18n in v0.3)
- Core audio experience (match atmosphere in v0.2-v0.3)
- Nice-to-haves come later

---

## How to Graduate Ideas to Features

1. **User Validation**: Collect evidence of demand (GitHub issues, feedback, polls)
2. **Scope Definition**: Write detailed spec with acceptance criteria
3. **Complexity Assessment**: Estimate time, dependencies, risks
4. **Priority Ranking**: Compare vs other planned features
5. **Capacity Check**: Does it fit in upcoming release timeline?

If all checks pass → Move to `FEATURES.md` and assign to release milestone

---

**Last Updated**: 2025-11-04
**Next Review**: After v0.3 ships

---

## Note on v0.1 Milestone

✅ **v0.1 COMPLETED** (2025-11-03): All planned features shipped across 5 releases (v0.2.0-v0.2.4). This unblocked several deferred features mentioned in this document:
- teams.json now editable by users (v0.2.1) - enables manual team customization without CRUD UI
- File logging implemented (v0.2.4) - foundation for future telemetry/diagnostics
- Multi-monitor support added (v0.2.2) - users can now select correct display
