// GUI module for FM Goal Musics
//
// This module contains the main GUI application logic.
// Transitioning to MVU (Model-View-Update) architecture.

mod state;
pub mod model;
pub mod messages;
pub mod update;

// Re-export public types
pub use state::{AppState, MusicEntry, ProcessState};

use state::{save_capture_image, AppTab, CapturePreview, PreviewAudio};

use display_info::DisplayInfo;
use eframe::egui;
use image::ImageBuffer;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use parking_lot::Mutex;
use crossbeam_channel::{self, unbounded, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use crate::audio::AudioManager;
use crate::audio_converter;
use crate::capture::{CaptureManager, CaptureRegion};
use crate::config::Config;
use crate::ocr::OcrManager;
use crate::utils::Debouncer;

// Region selector is implemented inline to avoid creating nested native
// windows that can crash on some platforms when called from UI callbacks.

/// Main GUI application
enum DetectionCommand {
    StopAudio,
}

/// Update modal display state - mirrors UpdateCheckResult for UI display
enum UpdateModalState {
    UpdateAvailable {
        latest_version: String,
        current_version: String,
        release_notes: String,
        download_url: String,
    },
    UpToDate {
        current_version: String,
    },
    Error {
        message: String,
    },
}

pub struct FMGoalMusicsApp {
    state: Arc<Mutex<AppState>>,
    detection_thread: Option<thread::JoinHandle<()>>,
    selecting_region: bool,
    region_selector: Option<RegionSelectState>,
    hide_window_for_capture: bool,
    capture_delay_frames: u8,
    preview_audio: Option<PreviewAudio>,
    preview_playing: bool,
    screen_resolution: Option<(u32, u32)>,
    capture_preview: CapturePreview,
    latest_capture: Arc<Mutex<Option<(ImageBuffer<image::Rgba<u8>, Vec<u8>>, std::time::Instant)>>>,
    capture_dirty: Arc<AtomicBool>,
    cached_audio_data: Option<(PathBuf, Arc<Vec<u8>>)>,
    detection_cmd_tx: Option<Sender<DetectionCommand>>,
    team_database: Option<crate::teams::TeamDatabase>,
    selected_league: Option<String>,
    selected_team_key: Option<String>,
    active_tab: AppTab,

    // Update checker fields
    update_check_rx: Option<Receiver<crate::update_checker::UpdateCheckResult>>,
    update_modal_state: Option<UpdateModalState>,
}

impl FMGoalMusicsApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Configure fonts to better support extended Latin characters (e.g., Turkish)
        Self::configure_fonts(_cc);
        let screen_resolution = DisplayInfo::all().ok().and_then(|infos| {
            let primary = infos.iter().find(|d| d.is_primary);
            let target = if let Some(display) = primary {
                Some(display)
            } else {
                infos.first()
            };
            target.map(|display| (display.width as u32, display.height as u32))
        });

        // Load team database
        let team_database = match crate::teams::TeamDatabase::load() {
            Ok(db) => {
                let db_path = crate::teams::TeamDatabase::database_path_display();
                log::info!("[fm-goal-musics] Team database loaded successfully from: {}", db_path);
                Some(db)
            }
            Err(e) => {
                log::error!("[fm-goal-musics] ERROR: Failed to load team database: {}", e);
                log::error!("[fm-goal-musics] Team selection will not be available.");
                log::error!("[fm-goal-musics] Expected location: {}", crate::teams::TeamDatabase::database_path_display());
                None
            }
        };

        let mut app = Self {
            state: Arc::new(Mutex::new(AppState::default())),
            detection_thread: None,
            selecting_region: false,
            region_selector: None,
            hide_window_for_capture: false,
            capture_delay_frames: 0,
            preview_audio: None,
            preview_playing: false,
            screen_resolution,
            capture_preview: CapturePreview::default(),
            latest_capture: Arc::new(Mutex::new(None)),
            capture_dirty: Arc::new(AtomicBool::new(false)),
            cached_audio_data: None,
            detection_cmd_tx: None,
            team_database,
            selected_league: None,
            selected_team_key: None,
            active_tab: AppTab::Library,
            update_check_rx: None,
            update_modal_state: None,
        };

        // Load config and restore music list
        match Config::load() {
            Ok(config) => {
                let mut state = app.state.lock();
                state.capture_region = config.capture_region;
                state.ocr_threshold = config.ocr_threshold;
                state.debounce_ms = config.debounce_ms;
                state.enable_morph_open = config.enable_morph_open;
                state.selected_music_index = config.selected_music_index;
                state.selected_team = config.selected_team.clone();
                state.music_volume = config.music_volume;
                state.ambiance_volume = config.ambiance_volume;
                state.goal_ambiance_path = config.goal_ambiance_path.clone();
                state.ambiance_enabled = config.ambiance_enabled;
                state.music_length_ms = config.music_length_ms;
                state.ambiance_length_ms = config.ambiance_length_ms;
                state.auto_check_updates = config.auto_check_updates;
                state.skipped_version = config.skipped_version.clone();
                state.selected_monitor_index = config.selected_monitor_index;

                // Initialize update checker if enabled
                if config.auto_check_updates {
                    let skipped_version = config.skipped_version.clone();
                    let (tx, rx) = unbounded();
                    app.update_check_rx = Some(rx);

                    // Spawn background thread to check for updates
                    thread::spawn(move || {
                        let result = crate::update_checker::check_for_updates();

                        // For startup checks, only send if update is available and not skipped
                        // Don't show "up to date" or errors on startup (silent)
                        if let crate::update_checker::UpdateCheckResult::UpdateAvailable { ref latest_version, .. } = result {
                            if !crate::update_checker::should_skip_version(latest_version, &skipped_version) {
                                let _ = tx.send(result);
                            }
                        }
                    });
                }

                // Initialize selected league and team from config
                if let Some(ref team) = config.selected_team {
                    app.selected_league = Some(team.league.clone());
                    app.selected_team_key = Some(team.team_key.clone());
                }
                
                // Convert config music entries to GUI entries; derive display name from file stem
                state.music_list = config.music_list.iter().map(|entry| {
                    let path = PathBuf::from(&entry.path);
                    let name = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| entry.name.clone());
                    MusicEntry {
                        name,
                        path,
                        shortcut: entry.shortcut.clone(),
                    }
                }).collect();
                
                log::info!("✓ Loaded {} music files from config", state.music_list.len());
            }
            Err(e) => {
                log::warn!("⚠ Failed to load config: {}", e);
                // Use default screen-based capture region
                if let Some((screen_w, screen_h)) = screen_resolution {
                    let mut state = app.state.lock();
                    if state.capture_region == [0, 0, 200, 100] {
                        let capture_height = (screen_h / 4).max(1);
                        let capture_y = screen_h.saturating_sub(capture_height);
                        state.capture_region = [0, capture_y, screen_w, capture_height];
                    }
                }
            }
        }
        
        app
    }

    fn configure_fonts(cc: &eframe::CreationContext<'_>) {
        // Try to load a system font with wide Unicode coverage (Turkish supported)
        // Common macOS locations; fallback to default if none found
        let candidates = [
            "/Library/Fonts/HelveticaNeue.dfont",
            "/Library/Fonts/Helvetica.ttf",
            "/Library/Fonts/Arial.ttf",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
        ];

        for path in candidates.iter() {
            if let Ok(bytes) = std::fs::read(path) {
                let mut fonts = egui::FontDefinitions::default();
                fonts.font_data.insert(
                    "ui_override".to_owned(),
                    egui::FontData::from_owned(bytes).into(),
                );
                // Put our font first in the proportional family for UI text
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "ui_override".to_owned());
                cc.egui_ctx.set_fonts(fonts);
                break;
            }
        }
    }

    fn start_region_selection(&mut self) {
        log::info!("═══════════════════════════════════════════════════════");
        log::info!("🎯 REGION SELECTION BUTTON CLICKED!");
        log::info!("   Platform: {}", std::env::consts::OS);
        log::info!("   Starting region selection process...");
        log::info!("═══════════════════════════════════════════════════════");

        self.selecting_region = true;
        self.region_selector = Some(RegionSelectState::default());
        self.hide_window_for_capture = false;
        self.capture_delay_frames = 0;

        log::info!("   ✓ Region selector state initialized");
    }

    fn stop_preview(&mut self) {
        if let Some(audio) = &self.preview_audio {
            audio.manager.stop();
        }
        self.preview_audio = None;
        self.preview_playing = false;
    }

    fn get_or_load_audio_data(&mut self, path: &Path) -> Result<Arc<Vec<u8>>, String> {
        if let Some((cached_path, data)) = &self.cached_audio_data {
            if cached_path.as_path() == path {
                return Ok(Arc::clone(data));
            }
        }

        let bytes = fs::read(path).map_err(|e| format!("Failed to read audio file '{}': {}", path.display(), e))?;
        log::info!("✓ Preloaded audio file: {} ({} bytes)", path.display(), bytes.len());
        let arc = Arc::new(bytes);
        self.cached_audio_data = Some((path.to_path_buf(), Arc::clone(&arc)));
        Ok(arc)
    }

    fn refresh_capture_preview(&mut self, ctx: &egui::Context) {
        if !self.capture_dirty.swap(false, Ordering::SeqCst) {
            return;
        }

        let maybe_capture = {
            let mut slot = self.latest_capture.lock();
            slot.take()
        };

        if let Some((buffer, timestamp)) = maybe_capture {
            let width = buffer.width();
            let height = buffer.height();

            if width == 0 || height == 0 {
                return;
            }

            let raw = buffer.into_raw();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [width as usize, height as usize],
                &raw,
            );

            self.capture_preview.width = width;
            self.capture_preview.height = height;
            self.capture_preview.timestamp = Some(timestamp);

            // Store a copy for last_image if needed for saving
            self.capture_preview.last_image = Some(color_image.clone());

            if let Some(texture) = self.capture_preview.texture.as_mut() {
                texture.set(color_image, egui::TextureOptions::LINEAR);
            } else {
                let tex = ctx.load_texture(
                    "capture_preview",
                    color_image,
                    egui::TextureOptions::LINEAR,
                );
                self.capture_preview.texture = Some(tex);
            }
        }
    }

    fn save_config(&self) {
        let state = self.state.lock();
        
        let new_config = Config {
            capture_region: state.capture_region,
            ocr_threshold: state.ocr_threshold,
            debounce_ms: state.debounce_ms,
            enable_morph_open: state.enable_morph_open,
            bench_frames: 500,
            music_list: state.music_list.iter().map(|entry| {
                crate::config::MusicEntry {
                    name: entry.name.clone(),
                    path: entry.path.to_string_lossy().to_string(),
                    shortcut: entry.shortcut.clone(),
                }
            }).collect(),
            selected_music_index: state.selected_music_index,
            selected_team: state.selected_team.clone(),
            music_volume: state.music_volume,
            ambiance_volume: state.ambiance_volume,
            goal_ambiance_path: state.goal_ambiance_path.clone(),
            ambiance_enabled: state.ambiance_enabled,
            music_length_ms: state.music_length_ms,
            ambiance_length_ms: state.ambiance_length_ms,
            auto_check_updates: state.auto_check_updates,
            skipped_version: state.skipped_version.clone(),
            selected_monitor_index: state.selected_monitor_index,
        };

        // Load previous config to compare changes
        let mut changes = Vec::new();
        if let Ok(old_config) = Config::load() {
            // Compare each field and log differences
            if old_config.capture_region != new_config.capture_region {
                changes.push(format!("capture_region: {:?} -> {:?}", old_config.capture_region, new_config.capture_region));
            }
            if old_config.ocr_threshold != new_config.ocr_threshold {
                changes.push(format!("ocr_threshold: {} -> {}", old_config.ocr_threshold, new_config.ocr_threshold));
            }
            if old_config.debounce_ms != new_config.debounce_ms {
                changes.push(format!("debounce_ms: {} -> {}", old_config.debounce_ms, new_config.debounce_ms));
            }
            if old_config.enable_morph_open != new_config.enable_morph_open {
                changes.push(format!("enable_morph_open: {} -> {}", old_config.enable_morph_open, new_config.enable_morph_open));
            }
            if old_config.music_list.len() != new_config.music_list.len() || 
               old_config.music_list.iter().zip(new_config.music_list.iter()).any(|(old, new)| {
                   old.name != new.name || old.path != new.path || old.shortcut != new.shortcut
               }) {
                changes.push(format!("music_list: {} entries -> {} entries", old_config.music_list.len(), new_config.music_list.len()));
            }
            if old_config.selected_music_index != new_config.selected_music_index {
                changes.push(format!("selected_music_index: {:?} -> {:?}", old_config.selected_music_index, new_config.selected_music_index));
            }
            if old_config.selected_team != new_config.selected_team {
                changes.push(format!("selected_team: {:?} -> {:?}", old_config.selected_team, new_config.selected_team));
            }
            if (old_config.music_volume - new_config.music_volume).abs() > f32::EPSILON {
                changes.push(format!("music_volume: {:.2} -> {:.2}", old_config.music_volume, new_config.music_volume));
            }
            if (old_config.ambiance_volume - new_config.ambiance_volume).abs() > f32::EPSILON {
                changes.push(format!("ambiance_volume: {:.2} -> {:.2}", old_config.ambiance_volume, new_config.ambiance_volume));
            }
            if old_config.goal_ambiance_path != new_config.goal_ambiance_path {
                changes.push(format!("goal_ambiance_path: {:?} -> {:?}", old_config.goal_ambiance_path, new_config.goal_ambiance_path));
            }
            if old_config.ambiance_enabled != new_config.ambiance_enabled {
                changes.push(format!("ambiance_enabled: {} -> {}", old_config.ambiance_enabled, new_config.ambiance_enabled));
            }
            if old_config.music_length_ms != new_config.music_length_ms {
                changes.push(format!("music_length_ms: {} -> {}", old_config.music_length_ms, new_config.music_length_ms));
            }
            if old_config.ambiance_length_ms != new_config.ambiance_length_ms {
                changes.push(format!("ambiance_length_ms: {} -> {}", old_config.ambiance_length_ms, new_config.ambiance_length_ms));
            }
            if old_config.auto_check_updates != new_config.auto_check_updates {
                changes.push(format!("auto_check_updates: {} -> {}", old_config.auto_check_updates, new_config.auto_check_updates));
            }
            if old_config.skipped_version != new_config.skipped_version {
                changes.push(format!("skipped_version: {:?} -> {:?}", old_config.skipped_version, new_config.skipped_version));
            }
            if old_config.selected_monitor_index != new_config.selected_monitor_index {
                changes.push(format!("selected_monitor_index: {} -> {}", old_config.selected_monitor_index, new_config.selected_monitor_index));
            }
        }

        if let Err(e) = new_config.save() {
            log::warn!("⚠ Failed to save config: {}", e);
        } else {
            if changes.is_empty() {
                log::info!("✓ Config saved (no changes detected)");
            } else {
                log::info!("✓ Config saved with changes:");
                for change in changes {
                    log::info!("  - {}", change);
                }
            }
        }
    }

    fn check_for_updates_manually(&mut self) {
        log::info!("[update-checker] Manual update check requested");

        let (tx, rx) = unbounded();
        self.update_check_rx = Some(rx);

        // Spawn background thread to check for updates
        thread::spawn(move || {
            let result = crate::update_checker::check_for_updates();
            // For manual checks, always send result (all cases: update, up-to-date, error)
            let _ = tx.send(result);
        });
    }

    fn add_music_file(&mut self, path: PathBuf) {
        // Convert to WAV if not already WAV
        let final_path = match audio_converter::convert_to_wav(&path) {
            Ok(wav_path) => wav_path,
            Err(e) => {
                let mut state = self.state.lock();
                state.status_message = format!("Failed to convert audio: {}", e);
                return;
            }
        };

        let name = final_path
            .file_stem()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let mut state = self.state.lock();
        state.music_list.push(MusicEntry {
            name,
            path: final_path,
            shortcut: None,
        });
        
        // Save config
        drop(state);
        self.save_config();
    }

    fn start_detection(&mut self) {
        self.stop_preview();
        if let Some(handle) = self.detection_thread.take() {
            let _ = handle.join();
        }

let (music_path, music_name, capture_region, ocr_threshold, debounce_ms, enable_morph_open, selected_team, music_volume, ambiance_volume, ambiance_path, ambiance_enabled, music_length_ms, ambiance_length_ms, selected_monitor_index) = {
            let mut state = self.state.lock();

            if state.process_state != ProcessState::Stopped {
                state.status_message = "Already running!".to_string();
                return;
            }

            let selected_idx = match state.selected_music_index {
                Some(idx) => idx,
                None => {
                    state.status_message = "Please select a music file first!".to_string();
                    return;
                }
            };

            let entry = match state.music_list.get(selected_idx).cloned() {
                Some(entry) => entry,
                None => {
                    state.status_message = "Selected music entry not found.".to_string();
                    return;
                }
            };

            let capture_region = state.capture_region;
            let ocr_threshold = state.ocr_threshold;
            let debounce_ms = state.debounce_ms;
            let enable_morph_open = state.enable_morph_open;
            let selected_team = state.selected_team.clone();
            let music_volume = state.music_volume;
            let ambiance_volume = state.ambiance_volume;
            let ambiance_path = state.goal_ambiance_path.clone();
            let ambiance_enabled = state.ambiance_enabled;
            let music_length_ms = state.music_length_ms;
            let ambiance_length_ms = state.ambiance_length_ms;
            let selected_monitor_index = state.selected_monitor_index;

            log::info!(
                "[fm-goal-musics] Starting detection\n  music='{}'\n  region=[{}, {}, {}, {}]\n  ocr_threshold={}\n  debounce_ms={}\n  morph_open={}",
                entry.name,
                capture_region[0], capture_region[1], capture_region[2], capture_region[3],
                ocr_threshold,
                debounce_ms,
                enable_morph_open
            );

            if let Some(ref team) = selected_team {
                log::info!("  team_selection={} ({})", team.display_name, team.league);
            }

            state.process_state = ProcessState::Running {
                since: Instant::now(),
            };
            state.status_message = format!("Starting detection with '{}'", entry.name);
            state.detection_count = 0;

(entry.path.clone(), entry.name.clone(), capture_region, ocr_threshold, debounce_ms, enable_morph_open, selected_team, music_volume, ambiance_volume, ambiance_path, ambiance_enabled, music_length_ms, ambiance_length_ms, selected_monitor_index)
        };

        let audio_data = match self.get_or_load_audio_data(&music_path) {
            Ok(data) => data,
            Err(err) => {
                let mut state = self.state.lock();
                state.status_message = err;
                state.process_state = ProcessState::Stopped;
                return;
            }
        };

        // Preload ambiance audio if enabled (to avoid delay during playback)
        let ambiance_data: Option<Arc<Vec<u8>>> = if ambiance_enabled {
            if let Some(ref path) = ambiance_path {
                match std::fs::read(path) {
                    Ok(bytes) => {
                        log::info!("[fm-goal-musics] Preloaded ambiance sound: {} ({} bytes)", path, bytes.len());
                        Some(Arc::new(bytes))
                    },
                    Err(e) => {
                        log::warn!("[fm-goal-musics] Warning: Failed to read ambiance file: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        let state_clone = Arc::clone(&self.state);
        let latest_capture = Arc::clone(&self.latest_capture);
        let capture_dirty = Arc::clone(&self.capture_dirty);
        let (cmd_tx, cmd_rx) = unbounded();
        self.detection_cmd_tx = Some(cmd_tx);

        let handle = thread::spawn(move || {
            let notify_error = |message: String| {
                let mut st = state_clone.lock();
                st.status_message = message;
                st.process_state = ProcessState::Stopped;
            };

            let audio_manager = match AudioManager::from_preloaded(Arc::clone(&audio_data)) {
                Ok(manager) => {
                    manager.set_volume(music_volume);
                    manager
                },
                Err(e) => {
                    notify_error(format!("Audio init failed: {}", e));
                    return;
                }
            };
            
            // Initialize ambiance manager from preloaded data
            let ambiance_manager: Option<AudioManager> = if let Some(ref data) = ambiance_data {
                match AudioManager::from_preloaded(Arc::clone(data)) {
                    Ok(manager) => {
                        manager.set_volume(ambiance_volume);
                        log::info!("[fm-goal-musics] Ambiance audio manager initialized");
                        Some(manager)
                    },
                    Err(e) => {
                        log::warn!("[fm-goal-musics] Warning: Failed to initialize ambiance manager: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            let mut capture_manager = match CaptureManager::new(CaptureRegion::from_array(capture_region), selected_monitor_index) {
                Ok(manager) => manager,
                Err(e) => {
                    notify_error(format!("Capture init failed: {}", e));
                    return;
                }
            };

            let mut ocr_manager = match OcrManager::new_with_options(ocr_threshold, enable_morph_open) {
                Ok(manager) => manager,
                Err(e) => {
                    notify_error(format!("OCR init failed: {}", e));
                    return;
                }
            };
            
            // Load team database and create matcher if team is selected
            let team_matcher = if let Some(ref sel_team) = selected_team {
                match crate::teams::TeamDatabase::load() {
                    Ok(db) => {
                        match db.find_team(&sel_team.league, &sel_team.team_key) {
                            Some(team) => {
                                log::info!("[fm-goal-musics] Team matcher initialized for: {}", sel_team.display_name);
                                Some(crate::team_matcher::TeamMatcher::new(&team))
                            }
                            None => {
                                log::warn!("[fm-goal-musics] Warning: Selected team not found in database");
                                None
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("[fm-goal-musics] ERROR: Failed to load team database in detection thread: {}", e);
                        log::error!("[fm-goal-musics] Team-specific goal detection will not work.");
                        log::error!("[fm-goal-musics] Check that teams.json exists at: {}", crate::teams::TeamDatabase::database_path_display());
                        None
                    }
                }
            } else {
                None
            };

            let mut debouncer = Debouncer::new(debounce_ms);
            let mut last_detection_time: Option<std::time::Instant> = None;

            {
                let mut st = state_clone.lock();
                if team_matcher.is_some() {
                    st.status_message = format!("Monitoring for goals by selected team... Playing '{}' when detected", music_name);
                } else {
                    st.status_message = format!("Monitoring for all goals... Playing '{}' when detected", music_name);
                }
            }

            loop {
                while let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        DetectionCommand::StopAudio => {
                            audio_manager.stop();
                            let mut st = state_clone.lock();
                            st.status_message = "Goal audio stopped".to_string();
                        }
                    }
                }

                let process_state = {
                    let state = state_clone.lock();
                    state.process_state
                };

                match process_state {
                    ProcessState::Stopped => break,
                    ProcessState::Starting | ProcessState::Stopping => {
                        thread::sleep(Duration::from_millis(120));
                        continue;
                    }
                    ProcessState::Running { .. } => {}
                }

                // Skip expensive capture/OCR during debounce cooldown
                if let Some(last_time) = last_detection_time {
                    let elapsed = last_time.elapsed();
                    if elapsed < Duration::from_millis(debounce_ms) {
                        // Still in cooldown - sleep and skip expensive operations
                        let remaining_ms = debounce_ms.saturating_sub(elapsed.as_millis() as u64);
                        let sleep_ms = remaining_ms.min(500); // Wake up periodically to check state
                        thread::sleep(Duration::from_millis(sleep_ms));
                        continue;
                    }
                }

                let image = match capture_manager.capture_region() {
                    Ok(img) => img,
                    Err(e) => {
                        notify_error(format!("Capture error: {}", e));
                        break;
                    }
                };

                {
                    let mut slot = latest_capture.lock();
                    *slot = Some((image.clone(), std::time::Instant::now()));
                    capture_dirty.store(true, Ordering::SeqCst);
                }

                // Perform OCR - use team detection if team matcher is available
                let should_play_sound = if let Some(ref matcher) = team_matcher {
                    // Team selection enabled - extract and match team name
                    match ocr_manager.detect_goal_with_team(&image) {
                        Ok(Some(detected_team)) => {
                            if matcher.matches(&detected_team) {
                                log::info!("[fm-goal-musics] 🎯 GOAL FOR SELECTED TEAM: {}", detected_team);
                                true
                            } else {
                                log::info!("[fm-goal-musics] Goal detected for: {} (not selected team)", detected_team);
                                false
                            }
                        }
                        Ok(None) => false,
                        Err(e) => {
                            log::error!("[fm-goal-musics] OCR error: {}", e);
                            false
                        }
                    }
                } else {
                    // No team selection - use original detection
                    match ocr_manager.detect_goal(&image) {
                        Ok(detected) => detected,
                        Err(e) => {
                            log::error!("[fm-goal-musics] OCR error: {}", e);
                            false
                        }
                    }
                };

                if should_play_sound && debouncer.should_trigger() {
                    // Update last detection time to start cooldown period
                    last_detection_time = Some(std::time::Instant::now());

                    // Play ambiance first (crowd reaction) with fade-in and length limit
                    if let Some(ref ambiance) = ambiance_manager {
                        if ambiance_length_ms > 0 {
                            if let Err(e) = ambiance.play_sound_with_fade_and_limit(200, ambiance_length_ms) {
                                log::error!("[fm-goal-musics] Failed to play ambiance: {}", e);
                            }
                        } else {
                            // No length limit, use regular fade-in
                            if let Err(e) = ambiance.play_sound_with_fade(200) {
                                log::error!("[fm-goal-musics] Failed to play ambiance: {}", e);
                            }
                        }
                    }

                    // Play music immediately after with fade-in and length limit
                    let music_result = if music_length_ms > 0 {
                        audio_manager.play_sound_with_fade_and_limit(200, music_length_ms)
                    } else {
                        // No length limit, use regular fade-in
                        audio_manager.play_sound_with_fade(200)
                    };

                    match music_result {
                        Ok(()) => {
                            let mut st = state_clone.lock();
                            st.detection_count += 1;
                            let ambiance_msg = if ambiance_manager.is_some() {
                                " + crowd cheer"
                            } else {
                                ""
                            };
                            st.status_message = format!(
                                "Goal detected! Played '{}'{} (total: {})",
                                music_name,
                                ambiance_msg,
                                st.detection_count
                            );
                        }
                        Err(e) => {
                            log::error!("[fm-goal-musics] Failed to play music: {}", e);
                            let mut st = state_clone.lock();
                            st.status_message = format!("Failed to play music: {}", e);
                        }
                    }
                }

                thread::sleep(Duration::from_millis(16));
            }
        });

        self.detection_thread = Some(handle);
    }

    fn stop_detection(&mut self) {
        self.detection_cmd_tx = None;
        self.stop_preview();
        let mut state = self.state.lock();
        state.process_state = ProcessState::Stopped;
        state.status_message = "Stopped".to_string();
        drop(state);
    }

//     fn pause_detection(&mut self) {
//         let mut state = self.state.lock();
//         if matches!(state.process_state, ProcessState::Running { .. }) {
//             state.process_state = ProcessState::Paused;
//             state.status_message = "Paused".to_string();
//         } else if state.process_state == ProcessState::Paused {
//             state.process_state = ProcessState::Running;
//             state.status_message = "Resumed".to_string();
//         }
//     }

    fn stop_detection_audio(&mut self) {
        if let Some(tx) = &self.detection_cmd_tx {
            let _ = tx.send(DetectionCommand::StopAudio);
        }
    }
}

impl eframe::App for FMGoalMusicsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for update results from background thread
        if let Some(rx) = &self.update_check_rx {
            if let Ok(result) = rx.try_recv() {
                // Convert UpdateCheckResult to UpdateModalState and show modal
                self.update_modal_state = Some(match result {
                    crate::update_checker::UpdateCheckResult::UpdateAvailable {
                        latest_version,
                        current_version,
                        release_notes,
                        download_url,
                    } => UpdateModalState::UpdateAvailable {
                        latest_version,
                        current_version,
                        release_notes,
                        download_url,
                    },
                    crate::update_checker::UpdateCheckResult::UpToDate { current_version } => {
                        UpdateModalState::UpToDate { current_version }
                    }
                    crate::update_checker::UpdateCheckResult::Error { message } => {
                        UpdateModalState::Error { message }
                    }
                });
            }
        }

        // Render update modal based on check result
        if let Some(modal_state) = &self.update_modal_state {
            let mut close_modal = false;
            let mut skip_version = false;
            let mut latest_version_to_skip: Option<String> = None;

            match modal_state {
                UpdateModalState::UpdateAvailable {
                    latest_version,
                    current_version,
                    release_notes,
                    download_url,
                } => {
                    egui::Window::new("🔄 Update Available")
                        .collapsible(false)
                        .resizable(false)
                        .default_width(500.0)
                        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                        .show(ctx, |ui| {
                            ui.heading("A new version is available!");

                            ui.separator();

                            // Version comparison
                            ui.horizontal(|ui| {
                                ui.label("Current version:");
                                ui.label(current_version);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Latest version:");
                                ui.colored_label(egui::Color32::GREEN, latest_version);
                            });

                            ui.separator();

                            // Changelog
                            ui.heading("What's New:");
                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    ui.label(release_notes);
                                });

                            ui.separator();

                            // Action buttons
                            ui.horizontal(|ui| {
                                if ui.button("📥 Download Update").clicked() {
                                    if let Err(e) = open::that(download_url) {
                                        log::error!("[update-checker] Failed to open browser: {}", e);
                                    }
                                    close_modal = true;
                                }

                                if ui.button("⏭️ Skip This Version").clicked() {
                                    skip_version = true;
                                    latest_version_to_skip = Some(latest_version.clone());
                                    close_modal = true;
                                }

                                if ui.button("⏰ Remind Me Later").clicked() {
                                    close_modal = true;
                                }
                            });
                        });
                }
                UpdateModalState::UpToDate { current_version } => {
                    egui::Window::new("✅ Up to Date")
                        .collapsible(false)
                        .resizable(false)
                        .default_width(400.0)
                        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                        .show(ctx, |ui| {
                            ui.heading("You're running the latest version!");

                            ui.separator();

                            ui.horizontal(|ui| {
                                ui.label("Current version:");
                                ui.colored_label(egui::Color32::GREEN, current_version);
                            });

                            ui.separator();

                            if ui.button("✓ OK").clicked() {
                                close_modal = true;
                            }
                        });
                }
                UpdateModalState::Error { message } => {
                    egui::Window::new("⚠️ Update Check Failed")
                        .collapsible(false)
                        .resizable(false)
                        .default_width(450.0)
                        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                        .show(ctx, |ui| {
                            ui.heading("Failed to check for updates");

                            ui.separator();

                            ui.label("Error:");
                            ui.colored_label(egui::Color32::RED, message);

                            ui.separator();

                            ui.label("Please check your internet connection and try again later.");

                            ui.separator();

                            if ui.button("✓ OK").clicked() {
                                close_modal = true;
                            }
                        });
                }
            }

            // Handle modal actions outside the closure
            if skip_version {
                if let Some(version) = latest_version_to_skip {
                    let mut state = self.state.lock();
                    state.skipped_version = Some(version);
                    drop(state);
                    self.save_config();
                }
            }

            if close_modal {
                self.update_modal_state = None;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Batch read state once per frame to minimize lock contention
            let (is_stopped, is_running, status_color, status_message, detection_count) = {
                let state = self.state.lock();
                let is_stopped = state.process_state == ProcessState::Stopped;
                let is_running = matches!(state.process_state, ProcessState::Running { .. });
                let status_color = match state.process_state {
                    ProcessState::Running { .. } => egui::Color32::GREEN,
                    ProcessState::Stopped => egui::Color32::RED,
                    ProcessState::Starting | ProcessState::Stopping => egui::Color32::YELLOW,
                };
                (is_stopped, is_running, status_color, state.status_message.clone(), state.detection_count)
            };

            // Header with title and Start/Stop button
            ui.horizontal(|ui| {
                // Start/Stop Detection button on the left
                let button_text = if is_running { "⏹️ Stop Detection" } else { "▶️ Start Detection" };
                let button_color = if is_running { egui::Color32::from_rgb(244, 67, 54) } else { egui::Color32::from_rgb(76, 175, 80) };

                if ui.add(egui::Button::new(button_text).fill(button_color)).clicked() {
                    if is_running {
                        self.stop_detection();
                    } else if is_stopped {
                        self.start_detection();
                    }
                }

                ui.separator();
                ui.heading("⚽ FM Goal Musics");
            });
            ui.separator();

            // Status bar
            {
                let window_rect = ctx.input(|i| i.viewport_rect());
                let window_width = window_rect.width().round() as i32;
                let window_height = window_rect.height().round() as i32;
                ui.horizontal(|ui| {
                    ui.label("Status:");
                    ui.colored_label(status_color, &status_message);
                    ui.label(format!("| Detections: {}", detection_count));
                    if let Some((sw, sh)) = self.screen_resolution {
                        ui.label(format!("| Display: {}x{}", sw, sh));
                    }
                    ui.label(format!("| Window: {}x{}", window_width, window_height));
                });
            }

            ui.separator();
            ui.horizontal_wrapped(|ui| {
                for tab in AppTab::ALL {
                    let selected = self.active_tab == tab;
                    if ui.selectable_label(selected, tab.label()).clicked() {
                        self.active_tab = tab;
                    }
                }
            });
            ui.separator();

            // Library tab
            if self.active_tab == AppTab::Library {
                ui.heading("🎵 Music Files");
                
                ui.horizontal(|ui| {
                    if ui.button("➕ Add Music File").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Audio", &["mp3", "wav", "ogg"])
                            .pick_file()
                        {
                            self.add_music_file(path);
                        }
                    }
                    
                    if ui.button("🗑️ Remove Selected").clicked() {
                        let mut state = self.state.lock();
                        if let Some(idx) = state.selected_music_index {
                            state.music_list.remove(idx);
                            state.selected_music_index = None;
                        }
                        drop(state);
                        self.stop_preview();
                        self.save_config();
                    }

                    // Preview button
                    let preview_active = self.preview_playing;
                    let preview_label = if preview_active {
                        "🔇 Stop Preview"
                    } else {
                        "▶️ Preview"
                    };

                    if ui.button(preview_label).clicked() {
                        if preview_active {
                            self.stop_preview();
                        } else {
                            let selected_path = {
                                let state = self.state.lock();
                                state
                                    .selected_music_index
                                    .and_then(|idx| state.music_list.get(idx))
                                    .map(|entry| entry.path.clone())
                            };

                            match selected_path {
                                Some(path) => {
                                    let needs_reload = self
                                        .preview_audio
                                        .as_ref()
                                        .map_or(true, |p| p.path.as_path() != path.as_path());

                                    let audio_data = match self.get_or_load_audio_data(&path) {
                                        Ok(data) => data,
                                        Err(err) => {
                                            let mut st = self.state.lock();
                                            st.status_message = err;
                                            return;
                                        }
                                    };

                                    if needs_reload {
                                        self.stop_preview();
                                        match AudioManager::from_preloaded(Arc::clone(&audio_data)) {
                                            Ok(manager) => {
                                                self.preview_audio = Some(PreviewAudio {
                                                    manager,
                                                    path: path.clone(),
                                                });
                                            }
                                            Err(e) => {
                                                let mut st = self.state.lock();
                                                st.status_message = format!("Preview init failed: {}", e);
                                                return;
                                            }
                                        }
                                    }

                                    if let Some(preview) = self.preview_audio.as_ref() {
                                        preview.manager.stop();
                                        match preview.manager.play_sound() {
                                            Ok(()) => {
                                                self.preview_playing = true;
                                                let mut st = self.state.lock();
                                                st.status_message = "Preview playing...".to_string();
                                            }
                                            Err(e) => {
                                                let mut st = self.state.lock();
                                                st.status_message = format!("Preview failed: {}", e);
                                            }
                                        }
                                    }
                                }
                                None => {
                                    let mut st = self.state.lock();
                                    st.status_message = "Select a music file to preview.".to_string();
                                }
                            }
                        }
                    }
                });

                ui.separator();

                // Music list display
                let selection_changed = egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        let mut state = self.state.lock();
                        let mut new_selection = state.selected_music_index;
                        
                        for (idx, entry) in state.music_list.iter().enumerate() {
                            let is_selected = state.selected_music_index == Some(idx);
                            
                            ui.horizontal(|ui| {
                                if ui.selectable_label(is_selected, &entry.name).clicked() {
                                    new_selection = Some(idx);
                                }
                                
                                if let Some(shortcut) = &entry.shortcut {
                                    ui.label(format!("({})", shortcut));
                                }
                            });
                        }
                        
                        let changed = new_selection != state.selected_music_index;
                        if changed {
                            state.selected_music_index = new_selection;
                        }
                        changed
                    }).inner;
                
                if selection_changed {
                    self.save_config();
                }

                ui.separator();

                // Ambiance Sounds section
                ui.heading("🎺 Ambiance Sounds");
                
                ui.horizontal(|ui| {
                    let mut state = self.state.lock();
                    if ui.checkbox(&mut state.ambiance_enabled, "Enable Ambiance").changed() {
                        drop(state);
                        self.save_config();
                    }
                });
                
                ui.horizontal(|ui| {
                    if ui.button("➕ Add Goal Cheer Sound").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Audio", &["wav"])
                            .pick_file()
                        {
                            let mut state = self.state.lock();
                            state.goal_ambiance_path = Some(path.to_string_lossy().to_string());
                            drop(state);
                            self.save_config();
                        }
                    }
                    
                    if ui.button("🗑️ Remove Cheer Sound").clicked() {
                        let mut state = self.state.lock();
                        state.goal_ambiance_path = None;
                        drop(state);
                        self.save_config();
                    }
                });
                
                {
                    let state = self.state.lock();
                    if let Some(ref path) = state.goal_ambiance_path {
                        let display_name = PathBuf::from(path)
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| path.clone());
                        ui.label(format!("✓ Crowd cheer: {}", display_name));
                    } else {
                        ui.label("ℹ No crowd cheer sound selected");
                    }
                }
            }

            // Team Selection tab
            if self.active_tab == AppTab::TeamSelection {
                ui.separator();

                // Team Selection section
                ui.heading("⚽ Team Selection");
            
            if let Some(ref db) = self.team_database {
                ui.label("Select your team to play sound only for their goals:");
                
                // League dropdown
                let leagues = db.get_leagues();
                let mut league_changed = false;
                
                ui.horizontal(|ui| {
                    ui.label("League:");
                    egui::ComboBox::from_label("")
                        .selected_text(self.selected_league.as_deref().unwrap_or("-- Select League --"))
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(self.selected_league.is_none(), "-- Select League --").clicked() {
                                self.selected_league = None;
                                self.selected_team_key = None;
                                league_changed = true;
                            }
                            for league in &leagues {
                                if ui.selectable_label(self.selected_league.as_ref() == Some(league), league).clicked() {
                                    self.selected_league = Some(league.clone());
                                    self.selected_team_key = None;
                                    league_changed = true;
                                }
                            }
                        });
                });
                
                // Team dropdown (only if league is selected)
                if let Some(ref league) = self.selected_league {
                    if let Some(teams) = db.get_teams(league) {
                        ui.horizontal(|ui| {
                            ui.label("Team:");
                            egui::ComboBox::from_label(" ")
                                .selected_text(
                                    self.selected_team_key.as_ref()
                                        .and_then(|key| teams.iter().find(|(k, _)| k == key))
                                        .map(|(_, team)| team.display_name.as_str())
                                        .unwrap_or("-- Select Team --")
                                )
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(self.selected_team_key.is_none(), "-- Select Team --").clicked() {
                                        self.selected_team_key = None;
                                        league_changed = true;
                                    }
                                    for (key, team) in &teams {
                                        if ui.selectable_label(self.selected_team_key.as_ref() == Some(key), &team.display_name).clicked() {
                                            self.selected_team_key = Some(key.clone());
                                            league_changed = true;
                                        }
                                    }
                                });
                        });
                    }
                }
                
                // Update state and save if team selection changed
                if league_changed {
                    let mut state = self.state.lock();
                    
                    if let (Some(ref league), Some(ref team_key)) = (&self.selected_league, &self.selected_team_key) {
                        if let Some(team) = db.find_team(league, team_key) {
                            state.selected_team = Some(crate::config::SelectedTeam {
                                league: league.clone(),
                                team_key: team_key.clone(),
                                display_name: team.display_name.clone(),
                            });
                        }
                    } else {
                        state.selected_team = None;
                    }
                    
                    drop(state);
                    self.save_config();
                }
                
                // Display current selection
                {
                    let state = self.state.lock();
                    if let Some(ref team) = state.selected_team {
                        ui.label(format!("✓ Selected: {} ({})", team.display_name, team.league));
                    } else {
                        ui.label("ℹ No team selected - will play for all goals");
                    }
                }
                
                if ui.button("🗑️ Clear Selection").clicked() {
                    self.selected_league = None;
                    self.selected_team_key = None;
                    let mut state = self.state.lock();
                    state.selected_team = None;
                    drop(state);
                    self.save_config();
                }
            } else {
                ui.label("⚠ Team database not available");
            }

            ui.separator();

            // Capture preview
            self.refresh_capture_preview(ctx);
            if let Some(texture) = &self.capture_preview.texture {
                ui.group(|ui| {
                    ui.heading("📷 Capture Preview");
                    let aspect = texture.size()[0] as f32 / texture.size()[1] as f32;
                    let max_width = ui.available_width().min(400.0);
                    let desired_size = egui::Vec2::new(max_width, max_width / aspect);
                    ui.image(egui::load::SizedTexture::new(texture.id(), desired_size));

                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "Resolution: {}x{}",
                            self.capture_preview.width, self.capture_preview.height
                        ));

                        if let Some(ts) = self.capture_preview.timestamp {
                            let age = Instant::now().saturating_duration_since(ts);
                            ui.label(format!("Age: {:.1}s", age.as_secs_f32()));
                        }

                        if ui.button("Save frame...").clicked() {
                            if let Some(img) = &self.capture_preview.last_image {
                                if let Err(e) = save_capture_image(img) {
                                    let mut st = self.state.lock();
                                    st.status_message = format!("Failed to save capture: {}", e);
                                } else {
                                    let mut st = self.state.lock();
                                    st.status_message = "Saved capture preview to disk".to_string();
                                }
                            }
                        }
                    });
                });
            }
            } // end Detection tab

            // Settings tab
            if self.active_tab == AppTab::Settings {
                ui.separator();

                // Configuration section
                ui.heading("⚙️ Configuration");
            
            // Capture region controls
            {
                let mut state = self.state.lock();
                ui.horizontal(|ui| {
                    ui.label("Capture Region:");
                    ui.add(egui::DragValue::new(&mut state.capture_region[0]).prefix("X: "));
                    ui.add(egui::DragValue::new(&mut state.capture_region[1]).prefix("Y: "));
                    ui.add(egui::DragValue::new(&mut state.capture_region[2]).prefix("W: "));
                    ui.add(egui::DragValue::new(&mut state.capture_region[3]).prefix("H: "));
                });
            }
            
            // Visual selector button (separate scope to avoid borrow issues)
            ui.horizontal(|ui| {
                if ui.button("🎯 Select Region Visually").clicked() {
                    self.start_region_selection();
                }
                if ui.button("🔄 Reset Region").clicked() {
                    if let Some((screen_w, screen_h)) = self.screen_resolution {
                        let mut state = self.state.lock();
                        let capture_height = (screen_h / 4).max(1);
                        let capture_y = screen_h.saturating_sub(capture_height);
                        state.capture_region = [0, capture_y, screen_w, capture_height];
                    }
                }
            });
            ui.label("💡 Recommended: Use visual selector for accurate coordinates on HiDPI/Retina displays");

            // Monitor selection (multi-monitor support)
            ui.horizontal(|ui| {
                ui.label("Display:");
                let mut state = self.state.lock();

                // Get available monitors count
                let monitor_count = xcap::Monitor::all()
                    .map(|monitors| monitors.len())
                    .unwrap_or(1);

                // Show dropdown with monitor indices
                let prev_index = state.selected_monitor_index;
                egui::ComboBox::from_label("")
                    .selected_text(format!("Monitor {} ({})",
                        state.selected_monitor_index + 1,
                        if state.selected_monitor_index == 0 { "Primary" } else { "Secondary" }))
                    .show_ui(ui, |ui| {
                        for i in 0..monitor_count {
                            let label = format!("Monitor {} ({})",
                                i + 1,
                                if i == 0 { "Primary" } else { "Secondary" });
                            ui.selectable_value(&mut state.selected_monitor_index, i, label);
                        }
                    });

                // Save config if changed
                if state.selected_monitor_index != prev_index {
                    drop(state);
                    self.save_config();
                }
            });
            ui.label("💡 Select which monitor to capture from (0 = primary display)");

            // Other configuration options
            {
                let mut state = self.state.lock();
                
                ui.horizontal(|ui| {
                    ui.label("OCR Threshold:");
                    ui.add(egui::Slider::new(&mut state.ocr_threshold, 0..=255).text("(0 = auto)"));
                });

                ui.horizontal(|ui| {
                    ui.label("Debounce (ms):");
                    ui.add(egui::DragValue::new(&mut state.debounce_ms).speed(100));
                });

                ui.checkbox(&mut state.enable_morph_open, "Enable Morphological Opening (noise reduction)");
            }
                
            ui.separator();
            
            // Volume Controls
            ui.heading("🔊 Volume Controls");
            
            ui.horizontal(|ui| {
                ui.label("🎵 Music:");
                let mut state = self.state.lock();
                let mut music_vol_percent = (state.music_volume * 100.0) as i32;
                if ui.add(egui::Slider::new(&mut music_vol_percent, 0..=100).suffix("%")).changed() {
                    state.music_volume = (music_vol_percent as f32) / 100.0;
                    drop(state);
                    self.save_config();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("🔉 Ambiance:");
                let mut state = self.state.lock();
                let mut ambiance_vol_percent = (state.ambiance_volume * 100.0) as i32;
                if ui.add(egui::Slider::new(&mut ambiance_vol_percent, 0..=100).suffix("%")).changed() {
                    state.ambiance_volume = (ambiance_vol_percent as f32) / 100.0;
                    drop(state);
                    self.save_config();
                }
            });

            ui.separator();

            // Sound Length Controls
            ui.heading("⏱️ Sound Length Controls");
            
            ui.horizontal(|ui| {
                ui.label("🎵 Music Length:");
                let mut state = self.state.lock();
                let mut music_length_seconds = (state.music_length_ms as f32) / 1000.0;
                if ui.add(egui::Slider::new(&mut music_length_seconds, 0.0..=60.0).suffix(" seconds").step_by(1.0)).changed() {
                    state.music_length_ms = (music_length_seconds * 1000.0) as u64;
                    drop(state);
                    self.save_config();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("🔉 Ambiance Length:");
                let mut state = self.state.lock();
                let mut ambiance_length_seconds = (state.ambiance_length_ms as f32) / 1000.0;
                if ui.add(egui::Slider::new(&mut ambiance_length_seconds, 0.0..=60.0).suffix(" seconds").step_by(1.0)).changed() {
                    state.ambiance_length_ms = (ambiance_length_seconds * 1000.0) as u64;
                    drop(state);
                    self.save_config();
                }
            });

            ui.separator();

            // Update Checker
            ui.heading("🔄 Updates");

            ui.horizontal(|ui| {
                let mut state = self.state.lock();
                if ui.checkbox(&mut state.auto_check_updates, "Check for updates on startup").changed() {
                    drop(state);
                    self.save_config();
                }
            });

            if ui.button("🔍 Check for Updates Now").clicked() {
                self.check_for_updates_manually();
            }

            } // end Settings tab

            if self.active_tab == AppTab::Help {
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("📖 How to Use FM Goal Musics");
                
                ui.collapsing("🎵 Library Tab", |ui| {
                    ui.label("• Click '➕ Add Music File' to add celebration music (MP3, WAV, OGG)");
                    ui.label("• Select a music file from the list");
                    ui.label("• Click '▶️ Preview' to test the selected music");
                    ui.label("• Click '🗑️ Remove Selected' to remove unwanted music");
                    ui.label("");
                    ui.label("Ambiance Sounds:");
                    ui.label("• Enable 'Ambiance' checkbox for crowd cheer effects");
                    ui.label("• Click '➕ Add Goal Cheer Sound' to add a WAV crowd sound");
                    ui.label("• This plays alongside your music for extra atmosphere");
                });
                
                ui.collapsing("⚽ Team Selection Tab", |ui| {
                    ui.label("• Select a League from the dropdown");
                    ui.label("• Select your Team from the filtered list");
                    ui.label("• Goal music will only play for your selected team's goals");
                    ui.label("• Leave unselected to play for all goals");
                    ui.label("");
                    ui.label("Capture Preview:");
                    ui.label("• Shows real-time capture of the configured screen region");
                    ui.label("• Use 'Save frame...' to export the current captured frame");
                });
                
                ui.collapsing("⚙️ Settings Tab", |ui| {
                    ui.label("Configuration:");
                    ui.label("• Capture Region: X, Y, Width, Height of screen area to monitor");
                    ui.label("• Click '🎯 Select Region Visually' to drag-select on screen");
                    ui.label("• OCR Threshold: 0 = auto (recommended), 1-255 = manual");
                    ui.label("• Debounce: Cooldown between detections (default 8000ms)");
                    ui.label("• Morphological Opening: Reduces noise (adds 5-10ms latency)");
                    ui.label("");
                    ui.label("Volume & Length:");
                    ui.label("• Music Volume: 0-100% playback volume for celebration music");
                    ui.label("• Ambiance Volume: 0-100% playback volume for crowd cheer");
                    ui.label("• Music Length: How long to play music (0 = full track)");
                    ui.label("• Ambiance Length: How long to play crowd sound (0 = full)");
                });
                
                ui.collapsing("🏁 Quick Start", |ui| {
                    ui.label("1. Add at least one music file in Library tab");
                    ui.label("2. (Optional) Select your team in Team Selection tab");
                    ui.label("3. Configure capture region in Settings tab");
                    ui.label("4. Click '▶️ Start Detection' button (top-left)");
                    ui.label("5. Play Football Manager and watch for goals!");
                    ui.label("");
                    ui.label("✅ Status bar shows detection state (Green = Running)");
                    ui.label("✅ Detection count increments with each goal detected");
                });
                
                ui.collapsing("🔧 Configuring teams.json", |ui| {
                    ui.label("The teams database is automatically created on first run at:");
                    ui.label("  macOS: ~/Library/Application Support/FMGoalMusic/teams.json");
                    ui.label("  Windows: %APPDATA%/FMGoalMusic/teams.json");
                    ui.label("  Linux: ~/.config/FMGoalMusic/teams.json");
                    ui.label("");
                    ui.label("You can safely edit this file to add your favorite teams!");
                    ui.label("");
                    ui.label("Structure:");
                    ui.label("  {");
                    ui.label("    \"Premier League\": {");
                    ui.label("      \"manchester_united\": {");
                    ui.label("        \"display_name\": \"Manchester Utd\",");
                    ui.label("        \"variations\": [\"Man Utd\", \"Man United\", \"MUFC\"]");
                    ui.label("      }");
                    ui.label("    }");
                    ui.label("  }");
                    ui.label("");
                    ui.label("• Add your leagues and teams with variations");
                    ui.label("• Variations help match different OCR results");
                    ui.label("• Restart the app after editing teams.json");
                });
                
                ui.collapsing("❓ Troubleshooting", |ui| {
                    ui.label("Music not playing:");
                    ui.label("• Check music file is selected in Library");
                    ui.label("• Verify 'Start Detection' is active (button shows 'Stop')");
                    ui.label("• Confirm capture region covers goal text area");
                    ui.label("");
                    ui.label("No goals detected:");
                    ui.label("• Use 'Capture Preview' to verify region captures goal text");
                    ui.label("• Try OCR Threshold = 0 (auto) first");
                    ui.label("• Increase debounce if detecting multiple times");
                    ui.label("");
                    ui.label("Team selection not working:");
                    ui.label("• Verify team exists in teams.json with correct variations");
                    ui.label("• Check Team Selection tab shows '✓ Selected: [team]'");
                    ui.label("• Ensure OCR is reading team name correctly");
                });
                });
            }

        });

        // Region selector overlay window (implemented inline)
        if self.selecting_region {
            log::info!("📸 Region selector active in update loop");

            // Initialize on first show
            if let Some(sel) = &mut self.region_selector {
                // Only minimize/hide if we haven't captured yet
                if !sel.initialized && !self.hide_window_for_capture {
                    log::info!("   Preparing window for screenshot capture...");

                    // On Windows, hiding the main window closes the app
                    // So we minimize it instead
                    #[cfg(target_os = "windows")]
                    {
                        log::info!("   (Windows: Minimizing window instead of hiding)");
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }

                    // On macOS/Linux, we can safely hide the window
                    #[cfg(not(target_os = "windows"))]
                    {
                        log::info!("   (macOS/Linux: Hiding window)");
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                    }

                    self.hide_window_for_capture = true;
                    self.capture_delay_frames = 2;
                    ctx.request_repaint();
                    return;
                }
                if !sel.initialized {
                    if self.capture_delay_frames > 0 {
                        log::info!("   Waiting for window to minimize/hide... (frames left: {})", self.capture_delay_frames);
                        self.capture_delay_frames = self.capture_delay_frames.saturating_sub(1);
                        ctx.request_repaint();
                        return;
                    }

                    log::info!("   Window minimized/hidden, starting screenshot capture...");
                    let capture_result = sel.capture_fullscreen();

                    // Restore window visibility
                    #[cfg(target_os = "windows")]
                    {
                        log::info!("   Restoring window (Windows: un-minimize)...");
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    }

                    #[cfg(not(target_os = "windows"))]
                    {
                        log::info!("   Restoring window (macOS/Linux: show)...");
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    }

                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    // Don't reset hide_window_for_capture to false - keep it true to prevent re-entering minimize logic
                    // self.hide_window_for_capture = false;
                    self.capture_delay_frames = 0;

                    match capture_result {
                        Err(e) => {
                            // Make window visible again (already done above, but ensure it's restored)
                            #[cfg(target_os = "windows")]
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));

                            #[cfg(not(target_os = "windows"))]
                            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));

                            // Log to console for debugging
                            log::error!("❌ Region selection capture failed: {}", e);
                            log::error!("   This error occurred during screen capture for region selection");

                            let mut st = self.state.lock();
                            st.status_message = format!("Region selection failed: {}", e);

                            self.selecting_region = false;
                            self.region_selector = None;
                            return;
                        }
                        Ok(()) => {
                            log::info!("✓ Screenshot captured successfully for region selection");
                            sel.initialized = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                            sel.fullscreen_on = true;
                        }
                    }
                }
            }

            if let Some(sel) = &mut self.region_selector {
                // Local flags to avoid borrowing self inside the closure
                let mut selection_done: Option<[u32; 4]> = None;
                let mut cancel_requested: bool = false;

                egui::Window::new("Select Region")
                    .title_bar(true)
                    .resizable(true)
                    .collapsible(false)
                    .default_size(egui::vec2(1000.0, 700.0))
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(ctx, |ui| {
                        // Create texture once we have pixels and a live egui Context
                        if sel.texture.is_none() {
                            if let Some(pixels) = sel.pixels_rgba.as_ref() {
                                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                    [sel.img_w, sel.img_h],
                                    pixels,
                                );
                                sel.texture = Some(ui.ctx().load_texture(
                                    "region_selector_screenshot",
                                    color_image,
                                    egui::TextureOptions::default(),
                                ));
                            }
                        }

                        if let Some(texture) = &sel.texture {
                            let available = ui.available_size();
                            let img_size = texture.size_vec2();
                            sel.scale = (available.x / img_size.x).min(available.y / img_size.y).max(0.1);
                            let display_size = img_size * sel.scale;

                            let response = ui.add(
                                egui::Image::new(texture)
                                    .fit_to_exact_size(display_size)
                                    .sense(egui::Sense::click_and_drag()),
                            );

                            if response.drag_started() {
                                if let Some(pos) = response.interact_pointer_pos() {
                                    sel.start = Some(pos);
                                    sel.current = Some(pos);
                                }
                            }
                            if response.dragged() {
                                if let Some(pos) = response.interact_pointer_pos() {
                                    sel.current = Some(pos);
                                }
                            }
                            if response.drag_stopped() {
                                if let (Some(s), Some(c)) = (sel.start, sel.current) {
                                    // Get image position offset within the window
                                    let img_min = response.rect.min;

                                    // Convert absolute positions to image-relative positions
                                    let rel_x1 = (s.x - img_min.x).max(0.0);
                                    let rel_y1 = (s.y - img_min.y).max(0.0);
                                    let rel_x2 = (c.x - img_min.x).max(0.0);
                                    let rel_y2 = (c.y - img_min.y).max(0.0);

                                    let x1 = rel_x1.min(rel_x2);
                                    let y1 = rel_y1.min(rel_y2);
                                    let x2 = rel_x1.max(rel_x2);
                                    let y2 = rel_y1.max(rel_y2);

                                    // Step 1: Convert from display coordinates to physical screen pixels
                                    let phys_x = x1 / sel.scale;
                                    let phys_y = y1 / sel.scale;
                                    let phys_w = (x2 - x1) / sel.scale;
                                    let phys_h = (y2 - y1) / sel.scale;

                                    // Step 2: Convert from physical pixels to logical pixels
                                    // (divide by display scale: 2.0 for Retina)
                                    let logical_x = (phys_x / sel.display_scale) as u32;
                                    let logical_y = (phys_y / sel.display_scale) as u32;
                                    let logical_w = (phys_w / sel.display_scale).max(1.0) as u32;
                                    let logical_h = (phys_h / sel.display_scale).max(1.0) as u32;

                                    // Step 3: Clamp to logical screen bounds
                                    let x = logical_x.min(sel.logical_w.saturating_sub(1));
                                    let y = logical_y.min(sel.logical_h.saturating_sub(1));
                                    let w = logical_w.min(sel.logical_w - x);
                                    let h = logical_h.min(sel.logical_h - y);

                                    selection_done = Some([x, y, w, h]);
                                }
                            }

                            // Draw selection rectangle overlay
                            if let (Some(s), Some(c)) = (sel.start, sel.current) {
                                let rect = egui::Rect::from_two_pos(s, c);
                                ui.painter().rect_stroke(
                                    rect,
                                    0.0,
                                    egui::Stroke::new(2.0, egui::Color32::RED),
                                    egui::StrokeKind::Inside,
                                );
                            }
                        } else {
                            ui.label("Capturing screenshot...");
                        }

                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                cancel_requested = true;
                            }
                            ui.label("Click and drag over the screenshot, then release to confirm.");
                        });
                    });

                // Apply results outside of the closure to avoid borrowing self and sel simultaneously
                if let Some([x, y, w, h]) = selection_done {
                    // Get dimensions while we still have the mutable borrow
                    let (phys_w, phys_h) = (sel.img_w, sel.img_h);
                    let (logical_w, logical_h) = (sel.logical_w, sel.logical_h);
                    let display_scale = sel.display_scale;
                    let fullscreen = sel.fullscreen_on;

                    // Update state
                    let mut st = self.state.lock();
                    st.capture_region = [x, y, w, h];
                    st.status_message = format!(
                        "Region selected: [{}, {}, {}, {}] (Logical: {}x{}, Physical: {}x{}, Scale: {:.1}x)",
                        x, y, w, h, logical_w, logical_h, phys_w, phys_h, display_scale
                    );

                    // Restore window state
                    if fullscreen {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                    }
                    self.selecting_region = false;
                    self.region_selector = None;
                } else if cancel_requested {
                    if sel.fullscreen_on {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                    }
                    self.selecting_region = false;
                    self.region_selector = None;
                }
            }
        }

        // Request repaint for smooth updates
        ctx.request_repaint();
    }
}

// Inline region selector state
#[derive(Default)]
struct RegionSelectState {
    initialized: bool,
    // Deferred GPU texture (created inside egui callbacks)
    texture: Option<egui::TextureHandle>,
    // Raw pixels captured from screen (physical resolution on Retina)
    img_w: usize,
    img_h: usize,
    pixels_rgba: Option<Vec<u8>>,
    // Logical screen dimensions (what the OS reports)
    logical_w: u32,
    logical_h: u32,
    // Display scale factor (e.g., 2.0 for Retina)
    display_scale: f32,
    // UI interaction state
    scale: f32,
    start: Option<egui::Pos2>,
    current: Option<egui::Pos2>,
    fullscreen_on: bool,
}

impl RegionSelectState {
    fn capture_fullscreen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Use xcap to capture the entire screen
        use xcap::Monitor;

        log::info!("🔍 Starting fullscreen capture for region selection...");

        // Get all monitors
        let monitors = Monitor::all()
            .map_err(|e| {
                log::error!("❌ Failed to enumerate monitors: {}", e);
                format!("Failed to enumerate monitors: {}", e)
            })?;

        log::info!("   Found {} monitor(s)", monitors.len());

        if monitors.is_empty() {
            log::error!("❌ No monitors found!");
            return Err("No monitors found".into());
        }

        // Get primary monitor (first in the list)
        let monitor = monitors.into_iter().next()
            .ok_or("Failed to get primary monitor")?;

        // Get logical dimensions (what the OS reports, e.g., 1512x982 on Retina)
        let logical_w = monitor.width().unwrap_or(0);
        let logical_h = monitor.height().unwrap_or(0);

        log::info!("   Monitor logical size: {}x{}", logical_w, logical_h);

        log::info!("   Attempting to capture screen...");

        // Capture full screen with permission error handling
        let image = monitor.capture_image().map_err(|e| -> Box<dyn std::error::Error> {
            let error_msg = format!("{}", e);

            log::error!("❌ Screen capture failed: {}", error_msg);

            #[cfg(target_os = "macos")]
            if error_msg.contains("permission") || error_msg.contains("denied") || error_msg.contains("authorization") {
                return format!(
                    "Screen Recording permission required.\n\
                    \n\
                    To grant permission on macOS:\n\
                    1. Open System Preferences/Settings > Privacy & Security\n\
                    2. Click 'Screen Recording'\n\
                    3. Enable permission for this application\n\
                    4. Restart the application and try again\n\
                    \n\
                    Original error: {}", e
                ).into();
            }

            #[cfg(target_os = "windows")]
            if error_msg.contains("permission") || error_msg.contains("denied") || error_msg.contains("access") {
                return format!(
                    "Screen capture access denied on Windows.\n\
                    \n\
                    This might be due to:\n\
                    1. Windows Privacy settings blocking screen capture\n\
                    2. Antivirus software blocking the capture\n\
                    3. Running in a restricted environment\n\
                    \n\
                    Try running the application as Administrator.\n\
                    \n\
                    Original error: {}", e
                ).into();
            }

            format!("Failed to capture screen: {}", e).into()
        })?;

        log::info!("   ✓ Screen captured successfully");

        // Get dimensions and pixel data (physical resolution on Retina)
        let w = image.width();
        let h = image.height();
        let rgba = image.into_raw();

        // Calculate display scale factor (e.g., 2.0 for Retina)
        // Physical pixels / Logical pixels
        let display_scale = if logical_w > 0 {
            w as f32 / logical_w as f32
        } else {
            1.0
        };

        // Defer texture creation to the active egui Context inside the selector window.
        self.img_w = w as usize;
        self.img_h = h as usize;
        self.logical_w = logical_w;
        self.logical_h = logical_h;
        self.display_scale = display_scale;
        self.pixels_rgba = Some(rgba);
        self.texture = None;

        log::info!("Screenshot: {}x{} (physical) | Monitor: {}x{} (logical) | Scale: {}x",
                 w, h, logical_w, logical_h, display_scale);

        Ok(())
    }
}
