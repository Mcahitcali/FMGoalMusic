# FMGoalMusic Roadmap

## Vision

**FMGoalMusic** transforms Football Manager gameplay into an immersive, stadium-like experience by automatically detecting goals and playing customized music and crowd sounds. Our mission is to make every FM session feel like you're in the stands of your favorite team's home stadium.

### Strategic Goals (2025)
1. **Stability First**: Fix critical bugs that block core functionality
2. **Platform Excellence**: Deliver professional, polished experience on Windows (primary platform)
3. **Global Reach**: Support multiple languages to expand user base internationally
4. **Immersive Audio**: Build comprehensive match atmosphere (kickoff, goals, final whistle)

---

## Release Milestones

### v0.1 - FOUNDATION + CRITICAL FIXES ✅ COMPLETED

**Theme**: Make the app production-ready with essential infrastructure

**Target**: 2-3 weeks (11-17 hours with AI assistance)
**Actual**: 3 weeks (~13 hours actual) ✅

**Shipped**:
- ✅ **Update checker (notify-only)** - v0.2.0: Users discover new releases automatically
- ✅ **teams.json user config directory** - v0.2.1: Fix critical bug preventing team customization
- ✅ **Multi-monitor simple selection** - v0.2.2: Unblock users with multiple displays (MVP)
- ✅ **Hide Windows console** - v0.2.3: Professional appearance on Windows
- ✅ **File logging with rotation** - v0.2.4: Enable user debugging and support

**Impact**:
- ✅ Unblocks customization (teams.json was in system folder requiring admin)
- ✅ Unblocks multi-monitor users (30-40% of user base)
- ✅ Enables future releases to be discovered (update checker)
- ✅ Professional polish for Windows majority

**Marketing message**: *"Now with auto-updates, multi-monitor support, and editable team database!"*

**Completed**: 2025-11-03

---

### v0.2 - UNBLOCK + ATTRACT

**Theme**: Fix remaining blockers + add exciting new capability

**Target**: 2-3 weeks after v0.1 (10-16 hours with AI assistance)

**Shipping**:
- ✅ **Match start crowd sound**: Play ambience when match kicks off (new feature!)
- ✅ **Enhanced multi-monitor**: Add display info, live preview, optional auto-detection

**Impact**:
- New audio feature drives user excitement and social sharing
- Multi-monitor experience becomes seamless (polish from v0.1 MVP)
- First step toward comprehensive match atmosphere

**Marketing message**: *"Match atmosphere is here! Crowd sounds at kickoff + enhanced multi-monitor"*

---

### v0.3 - GO INTERNATIONAL

**Theme**: Expand user base globally with language support

**Target**: 2-3 weeks after v0.2 (10-14 hours with AI assistance)

**Shipping**:
- ✅ **Dynamic OCR goal phrases (i18n)**: Support Spanish, Turkish, German, Italian, French
- ✅ **Match end crowd sound (score-aware)**: Cheer/boo/neutral based on result

**Impact**:
- Opens app to 5-10x larger addressable market (non-English FM players)
- Completes match atmosphere story (start, goals, end)
- Positions app as truly international product

**Marketing message**: *"¡Hola! Guten Tag! Merhaba! Now supports multiple languages + match end atmosphere!"*

---

### v0.4+ - FUTURE (3-4 weeks after v0.3)

**Themes Under Consideration**:
- Enhanced UX (system tray, hotkeys, first-run wizard)
- Team management UI (visual CRUD for teams/leagues)
- Advanced audio (reverb effects, dynamic crowd layers, chants)
- Player-specific goal music
- Idle playlist when not in match

**Decision Point**: After v0.3, reassess priorities based on:
- User feedback and feature requests
- Platform-specific needs (macOS catching up?)
- Market opportunities (partnerships, distributions)

---

## Timeline Overview

```
Week 1-3:   v0.1 - FOUNDATION + CRITICAL FIXES
            └─ Update checker, teams.json fix, multi-monitor MVP, console, logging

Week 4-6:   v0.2 - UNBLOCK + ATTRACT
            └─ Match start sound, enhanced multi-monitor

Week 7-9:   v0.3 - GO INTERNATIONAL
            └─ i18n OCR phrases, match end sound

Week 10+:   v0.4+ planning & execution
            └─ TBD based on user feedback
```

**Total to v0.3**: 6-9 weeks (realistic for part-time solo dev with AI assistance)

---

## Success Metrics

### v0.1 Success Criteria ✅ ALL MET
- ✅ Zero reports of "can't edit teams.json"
- ✅ Multi-monitor users report successful setup
- ✅ No console window complaints from Windows users
- ✅ Update checker notifies users of v0.2 when released

### v0.2 Success Criteria
- [ ] Users share clips of match start sound on social media
- [ ] Multi-monitor setup takes <1 minute
- [ ] Feature requests shift from "fix multi-monitor" to new capabilities

### v0.3 Success Criteria
- [ ] Downloads from non-English speaking countries increase 3x
- [ ] Community contributions of new language phrase packs
- [ ] "Match atmosphere" feature set considered complete by users

---

## Development Philosophy

1. **AI-Assisted Velocity**: Time estimates assume AI pair programming (Claude, GitHub Copilot, etc.)
2. **Batched Releases**: Ship meaningful milestones, not individual features
3. **Windows-First**: Prioritize majority platform, ensure parity later
4. **User Feedback Driven**: Roadmap adjusts based on real usage patterns
5. **Stability Over Features**: Fix critical bugs before adding new capabilities

---

## Deferred / Not Planned

See `IDEAS.md` for features intentionally deferred or not currently planned.

---

**Last Updated**: 2025-11-04
**Next Review**: After v0.2 ships

---

## Release History

### ✅ v0.1 - FOUNDATION + CRITICAL FIXES (Completed 2025-11-03)
- v0.2.0 - Update Checker: https://github.com/Mcahitcali/FMGoalMusic/releases/tag/v0.2.0
- v0.2.1 - teams.json Config Fix: https://github.com/Mcahitcali/FMGoalMusic/releases/tag/v0.2.1
- v0.2.2 - Multi-Monitor Support: https://github.com/Mcahitcali/FMGoalMusic/releases/tag/v0.2.2
- v0.2.3 - Hide Windows Console: https://github.com/Mcahitcali/FMGoalMusic/releases/tag/v0.2.3
- v0.2.4 - File Logging with Rotation: https://github.com/Mcahitcali/FMGoalMusic/releases/tag/v0.2.4
