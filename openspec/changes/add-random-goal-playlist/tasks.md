## 1. Implementation
- [ ] 1.1 Update configuration and AppState to track a goal playlist (multiple selected tracks) and an in-memory last played track index.
- [ ] 1.2 Update the Library tab UI to allow toggling tracks into/out of the goal playlist and to show a concise playlist summary.
- [ ] 1.3 Update detection setup and run loop to load playlist audio and select tracks via a random, no-immediate-repeat strategy.
- [ ] 1.4 Add a random-number dependency to Cargo.toml and wire it into the goal playback selection logic.
- [ ] 1.5 Ensure backward compatibility when only a single legacy selected track exists (treat it as a single-track playlist when no explicit playlist is stored).

## 2. Testing
- [ ] 2.1 Manual test with exactly one selected track to confirm that it can play for consecutive goals.
- [ ] 2.2 Manual test with two or more selected tracks to confirm that random playback works and the last track is not immediately repeated.
- [ ] 2.3 Add or update automated tests for playlist selection helper logic where feasible.
