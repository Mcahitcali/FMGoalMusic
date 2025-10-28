# Enhanced Prompt — **FM Goal Musics (Rust, macOS + Windows)**

## 0) Role & Output Contract
You are a **senior Rust engineer** optimizing for **lowest end-to-end latency** and **stability**.  
**Respond with code-first outputs** and only add brief notes when explicitly requested.  
**Never** change files outside the ones listed in each step. **Do not** introduce new crates unless asked.

> **Deliverables format:**  
> - For code: provide **complete file content** with path headers like `// FILE: src/capture.rs`.  
> - For edits: show unified diffs or explicit before/after blocks.  
> - For commands: list exact `cargo` commands`.  
This tight format follows vibe-coding guidance to **specify output shape up front**.

---

## 1) Context (What we’re building)
**App:** FM Goal Musics — background utility that detects the word **“GOAL”** inside a **fixed screen region** and **instantly** plays a celebration sound.  
**Hard target:** total latency **< 100 ms** from frame capture to audible playback.  
**Config:** no UI; settings via `config.json` (capture region, audio path, threshold).  
**OS:** macOS (Screen Recording permission) + Windows.  
**Tech stack (fixed):**
- `scap` for GPU-assisted capture  
- `leptess` (Tesseract) for OCR  
- `rodio` (with `mp3`) for playback (preload into memory)  
- `rdev` for global hotkeys (F8 pause, F9 quit)  
- `serde`/`serde_json`, `image`, `tray-item`, `dirs`  
**Project layout:** `src/{main,capture,ocr,audio,config,utils}.rs`.

---

## 2) Tasks (Do this step-by-step, iteratively)
We will work in **small, verifiable increments** instead of a single mega prompt (vibe-coding best practice). After each step, **run**, **measure**, and **iterate**.

### Step A — Baseline, deterministic core
- Implement the capture→preprocess→OCR→trigger pipeline with **single allocation reuse** and `PSM_SINGLE_WORD`, whitelist `"GOAL"`.  
- Preload audio into memory; `AudioManager::play_sound()` must be **non-blocking**.  
- Add `Arc<AtomicBool>` for `is_running`/`is_paused`; hotkeys F8/F9 wired.  
- **Output:** full contents of `src/{capture,ocr,audio,config,utils,main}.rs` + `Cargo.toml`.

### Step B — Latency instrumentation
- Add **lightweight timing** (e.g., `instant::Instant`) around **capture**, **preprocess**, **OCR**, **sink play start**.  
- Expose a one-shot `--bench` CLI flag to print p50/p95 latency over 500 iterations.  
- **Output:** diff only for touched files + the CLI flag wiring in `main.rs`.

### Step C — False-positive control
- Add configurable **binary threshold** and optional **morphological open** behind flags.  
- Implement **debounce** (e.g., ignore repeat trigger for N ms).  
- **Output:** `config.json` schema update + code diffs.

### Step D — OS-specific fast paths
- macOS: prefer highest refresh path in `scap`; document Screen Recording permission string.  
- Windows: ensure capture API path is `Windows.Graphics.Capture`.  
- **Output:** comments + code guards; **no extra crates**.

### Step E — Test hooks (micro-TDD)
- Provide a **test image set** loader and a `fn detect_goal(img: GrayImage)->bool` that is unit-testable.  
- Add **two failing tests first**, then make them pass (TDD).

---

## 3) Constraints, Don’ts & Quality Gates
**Hard constraints**
- Keep the inner loop **allocation-free** after warm-up; reuse buffers.  
- Initialize `scap::Capturer` and `leptess::LepTess` **once** and reuse.  
- End-to-end loop cadence: **≤ 100 ms**; if paused, back off to ~250 ms.  
- **No streaming from disk** on trigger; audio must already be in RAM.

**Don’ts**
- Don’t modify unrelated modules.  
- Don’t add heavyweight logging; prefer **counters/timers**.  
- Don’t refactor interfaces unless prompted (keep API stable).  

**Quality gates**
- Provide **p50/p95** latency printout after `--bench` runs 500 frames.  
- Unit tests for `detect_goal` must pass.  
- Explain any crate feature flags you enable in **one line** per flag.

---

## 4) File Boundaries (exact files you may touch)
- `Cargo.toml`, `src/main.rs`, `src/{capture,ocr,audio,config,utils}.rs`, `config.json`  
- **Do not** create additional binaries, GUIs, or services.

---

## 5) Config Schema (authoritative)
```json
{
  "capture_region": [x, y, width, height],
  "audio_file_path": "path/to/goal.mp3",
  "ocr_threshold": 150,
  "debounce_ms": 800,
  "enable_morph_open": false,
  "bench_frames": 500
}
```
If `config.json` is missing, **create with defaults** in platform config dir (`dirs::config_dir()`), and print its path for the user to edit.

---

## 6) Performance Playbook (quick reference)
- **Preprocess:** grayscale → binary threshold only; avoid expensive filters unless guarded by a flag.  
- **OCR:** `PSM_SINGLE_WORD`, whitelist `"GOAL"`, uppercase normalization.  
- **Audio:** persistent `OutputStream` + prebuilt `Sink`; warm the decoder at init.  
- **Threading:** single producer (capture/ocr) + lightweight notifier to audio sink.

---

## 7) Interaction Rules (how we’ll iterate)
- After each step, **run benches**, paste the **p50/p95 table** only.  
- If p95 > 100 ms, propose **one** change at a time; re-bench.  
- When I say “Next”, proceed to the next Step section.

---

## 8) Security & Review Notes
- Avoid unsafe code unless justified; if used, annotate with rationale.  
- Call out any platform permissions (macOS Screen Recording).  
- Provide a **brief self-review checklist** with each change (race risks, panics, allocations).

---

## 9) Build & Run
- **Build (release):** `cargo build --release`  
- **Run (normal):** `./target/release/fm-goal-musics`  
- **Run (bench):** `./target/release/fm-goal-musics --bench`

---

### Appendix — References that informed this prompt (no need to include in LLM input)
- Graphite: clarity/specificity, “don’ts,” output format, iterative refinement
- Supabase: 3-part prompt structure, iteration rhythm, master checklists
- Medium (Z. Jameel): scene setting, constraint cues, small steps/TDD hooks
- Reddit r/vibecoding: community tips like “don’t code until I confirm,” and small-batch changes
