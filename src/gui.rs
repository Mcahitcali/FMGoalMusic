use display_info::DisplayInfo;
use eframe::egui;
use image::ImageBuffer;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
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

/// Music entry with file path and optional keyboard shortcut
#[derive(Clone, Debug)]
pub struct MusicEntry {
    pub name: String,
    #[allow(dead_code)]
    pub path: PathBuf,
    pub shortcut: Option<String>,
}

struct PreviewAudio {
    manager: AudioManager,
    path: PathBuf,
}

#[derive(Default)]
struct CapturePreview {
    texture: Option<egui::TextureHandle>,
    last_image: Option<egui::ColorImage>,
    width: u32,
    height: u32,
    timestamp: Option<std::time::Instant>,
}

fn save_capture_image(image: &egui::ColorImage) -> Result<(), Box<dyn std::error::Error>> {
    let path = rfd::FileDialog::new()
        .add_filter("PNG Image", &["png"])
        .set_file_name("capture_preview.png")
        .save_file();

    let Some(path) = path else {
        return Ok(());
    };

    let mut buf = Vec::with_capacity(image.pixels.len() * 4);
    for pixel in &image.pixels {
        buf.extend_from_slice(&[pixel.r(), pixel.g(), pixel.b(), pixel.a()]);
    }

    image::save_buffer_with_format(
        path,
        &buf,
        image.width() as u32,
        image.height() as u32,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )?;

    Ok(())
}

/// Detection process state
#[derive(Clone, Copy, PartialEq)]
pub enum ProcessState {
    Stopped,
    Running,
    Paused,
}

/// Shared state between GUI and background detection thread
pub struct AppState {
    pub music_list: Vec<MusicEntry>,
    pub selected_music_index: Option<usize>,
    pub process_state: ProcessState,
    pub capture_region: [u32; 4],
    pub ocr_threshold: u8,
    pub debounce_ms: u64,
    pub enable_morph_open: bool,
    pub status_message: String,
    pub detection_count: usize,
    pub selected_team: Option<crate::config::SelectedTeam>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            music_list: Vec::new(),
            selected_music_index: None,
            process_state: ProcessState::Stopped,
            capture_region: [0, 0, 200, 100],
            ocr_threshold: 0,
            debounce_ms: 8000,
            enable_morph_open: false,
            status_message: "Ready".to_string(),
            detection_count: 0,
            selected_team: None,
        }
    }
}

/// Main GUI application
enum DetectionCommand {
    StopAudio,
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

        let app = Self {
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
        };

        // Load config and restore music list
        match Config::load() {
            Ok(config) => {
                let mut state = app.state.lock().unwrap();
                state.capture_region = config.capture_region;
                state.ocr_threshold = config.ocr_threshold;
                state.debounce_ms = config.debounce_ms;
                state.enable_morph_open = config.enable_morph_open;
                state.selected_music_index = config.selected_music_index;
                state.selected_team = config.selected_team;
                
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
                
                println!("✓ Loaded {} music files from config", state.music_list.len());
            }
            Err(e) => {
                println!("⚠ Failed to load config: {}", e);
                // Use default screen-based capture region
                if let Some((screen_w, screen_h)) = screen_resolution {
                    let mut state = app.state.lock().unwrap();
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
        self.selecting_region = true;
        self.region_selector = Some(RegionSelectState::default());
        self.hide_window_for_capture = false;
        self.capture_delay_frames = 0;
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
        println!("✓ Preloaded audio file: {} ({} bytes)", path.display(), bytes.len());
        let arc = Arc::new(bytes);
        self.cached_audio_data = Some((path.to_path_buf(), Arc::clone(&arc)));
        Ok(arc)
    }

    fn refresh_capture_preview(&mut self, ctx: &egui::Context) {
        if !self.capture_dirty.swap(false, Ordering::SeqCst) {
            return;
        }

        let maybe_capture = {
            let mut slot = self.latest_capture.lock().unwrap();
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

            if let Some(texture) = self.capture_preview.texture.as_mut() {
                texture.set(color_image.clone(), egui::TextureOptions::LINEAR);
            } else {
                let tex = ctx.load_texture(
                    "capture_preview",
                    color_image.clone(),
                    egui::TextureOptions::LINEAR,
                );
                self.capture_preview.texture = Some(tex);
            }

            self.capture_preview.last_image = Some(color_image);
        }
    }

    fn save_config(&self) {
        let state = self.state.lock().unwrap();
        
        let config = Config {
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
        };
        
        if let Err(e) = config.save() {
            println!("⚠ Failed to save config: {}", e);
        } else {
            println!("✓ Config saved");
        }
    }

    fn add_music_file(&mut self, path: PathBuf) {
        // Convert to WAV if not already WAV
        let final_path = match audio_converter::convert_to_wav(&path) {
            Ok(wav_path) => wav_path,
            Err(e) => {
                let mut state = self.state.lock().unwrap();
                state.status_message = format!("Failed to convert audio: {}", e);
                return;
            }
        };

        let name = final_path
            .file_stem()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let mut state = self.state.lock().unwrap();
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

        let (music_path, music_name, capture_region, ocr_threshold, debounce_ms, enable_morph_open, selected_team) = {
            let mut state = self.state.lock().unwrap();

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

            println!(
                "[fm-goal-musics] Starting detection\n  music='{}'\n  region=[{}, {}, {}, {}]\n  ocr_threshold={}\n  debounce_ms={}\n  morph_open={}",
                entry.name,
                capture_region[0], capture_region[1], capture_region[2], capture_region[3],
                ocr_threshold,
                debounce_ms,
                enable_morph_open
            );
            
            if let Some(ref team) = selected_team {
                println!("  team_selection={} ({})", team.display_name, team.league);
            }

            state.process_state = ProcessState::Running;
            state.status_message = format!("Starting detection with '{}'", entry.name);
            state.detection_count = 0;

            (entry.path.clone(), entry.name.clone(), capture_region, ocr_threshold, debounce_ms, enable_morph_open, selected_team)
        };

        let audio_data = match self.get_or_load_audio_data(&music_path) {
            Ok(data) => data,
            Err(err) => {
                let mut state = self.state.lock().unwrap();
                state.status_message = err;
                state.process_state = ProcessState::Stopped;
                return;
            }
        };

        let state_clone = Arc::clone(&self.state);
        let latest_capture = Arc::clone(&self.latest_capture);
        let capture_dirty = Arc::clone(&self.capture_dirty);
        let (cmd_tx, cmd_rx) = mpsc::channel();
        self.detection_cmd_tx = Some(cmd_tx);

        let handle = thread::spawn(move || {
            let notify_error = |message: String| {
                let mut st = state_clone.lock().unwrap();
                st.status_message = message;
                st.process_state = ProcessState::Stopped;
            };

            let audio_manager = match AudioManager::from_preloaded(Arc::clone(&audio_data)) {
                Ok(manager) => manager,
                Err(e) => {
                    notify_error(format!("Audio init failed: {}", e));
                    return;
                }
            };

            let mut capture_manager = match CaptureManager::new(CaptureRegion::from_array(capture_region)) {
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
                                println!("[fm-goal-musics] Team matcher initialized for: {}", sel_team.display_name);
                                Some(crate::team_matcher::TeamMatcher::new(&team))
                            }
                            None => {
                                println!("[fm-goal-musics] Warning: Selected team not found in database");
                                None
                            }
                        }
                    }
                    Err(e) => {
                        println!("[fm-goal-musics] Warning: Failed to load team database: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            let mut debouncer = Debouncer::new(debounce_ms);

            {
                let mut st = state_clone.lock().unwrap();
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
                            let mut st = state_clone.lock().unwrap();
                            st.status_message = "Goal audio stopped".to_string();
                        }
                    }
                }

                let process_state = {
                    let state = state_clone.lock().unwrap();
                    state.process_state
                };

                match process_state {
                    ProcessState::Stopped => break,
                    ProcessState::Paused => {
                        thread::sleep(Duration::from_millis(120));
                        continue;
                    }
                    ProcessState::Running => {}
                }

                let image = match capture_manager.capture_region() {
                    Ok(img) => img,
                    Err(e) => {
                        notify_error(format!("Capture error: {}", e));
                        break;
                    }
                };

                {
                    let mut slot = latest_capture.lock().unwrap();
                    *slot = Some((image.clone(), std::time::Instant::now()));
                    capture_dirty.store(true, Ordering::SeqCst);
                }

                // Perform OCR - use team detection if team matcher is available
                let should_play_sound = if let Some(ref matcher) = team_matcher {
                    // Team selection enabled - extract and match team name
                    match ocr_manager.detect_goal_with_team(&image) {
                        Ok(Some(detected_team)) => {
                            if matcher.matches(&detected_team) {
                                println!("[fm-goal-musics] 🎯 GOAL FOR SELECTED TEAM: {}", detected_team);
                                true
                            } else {
                                println!("[fm-goal-musics] Goal detected for: {} (not selected team)", detected_team);
                                false
                            }
                        }
                        Ok(None) => false,
                        Err(e) => {
                            println!("[fm-goal-musics] OCR error: {}", e);
                            false
                        }
                    }
                } else {
                    // No team selection - use original detection
                    match ocr_manager.detect_goal(&image) {
                        Ok(detected) => detected,
                        Err(e) => {
                            println!("[fm-goal-musics] OCR error: {}", e);
                            false
                        }
                    }
                };

                if should_play_sound && debouncer.should_trigger() {
                    match audio_manager.play_sound() {
                        Ok(()) => {
                            let mut st = state_clone.lock().unwrap();
                            st.detection_count += 1;
                            st.status_message = format!(
                                "Goal detected! Played '{}' (total: {})",
                                music_name,
                                st.detection_count
                            );
                        }
                        Err(e) => {
                            println!("[fm-goal-musics] Failed to play sound: {}", e);
                            let mut st = state_clone.lock().unwrap();
                            st.status_message = format!("Failed to play sound: {}", e);
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
        let mut state = self.state.lock().unwrap();
        state.process_state = ProcessState::Stopped;
        state.status_message = "Stopped".to_string();
        drop(state);
    }

    fn pause_detection(&mut self) {
        let mut state = self.state.lock().unwrap();
        if state.process_state == ProcessState::Running {
            state.process_state = ProcessState::Paused;
            state.status_message = "Paused".to_string();
        } else if state.process_state == ProcessState::Paused {
            state.process_state = ProcessState::Running;
            state.status_message = "Resumed".to_string();
        }
    }

    fn stop_detection_audio(&mut self) {
        if let Some(tx) = &self.detection_cmd_tx {
            let _ = tx.send(DetectionCommand::StopAudio);
        }
    }
}

impl eframe::App for FMGoalMusicsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("⚽ FM Goal Musics");
            ui.separator();

            // Status bar
            {
                let state = self.state.lock().unwrap();
                let window_rect = ctx.input(|i| i.viewport_rect());
                let window_width = window_rect.width().round() as i32;
                let window_height = window_rect.height().round() as i32;
                ui.horizontal(|ui| {
                    ui.label("Status:");
                    let status_color = match state.process_state {
                        ProcessState::Running => egui::Color32::GREEN,
                        ProcessState::Paused => egui::Color32::YELLOW,
                        ProcessState::Stopped => egui::Color32::RED,
                    };
                    ui.colored_label(status_color, &state.status_message);
                    ui.label(format!("| Detections: {}", state.detection_count));
                    if let Some((sw, sh)) = self.screen_resolution {
                        ui.label(format!("| Display: {}x{}", sw, sh));
                    }
                    ui.label(format!("| Window: {}x{}", window_width, window_height));
                });
            }

            ui.separator();

            // Music list section
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
                    let mut state = self.state.lock().unwrap();
                    if let Some(idx) = state.selected_music_index {
                        state.music_list.remove(idx);
                        state.selected_music_index = None;
                    }
                    drop(state);
                    self.stop_preview();
                    self.save_config();
                }
            });

            ui.separator();

            // Music list display
            let selection_changed = egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    let mut state = self.state.lock().unwrap();
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

            // Control buttons
            ui.heading("🎮 Controls");
            
            ui.horizontal(|ui| {
                let state = self.state.lock().unwrap();
                let is_stopped = state.process_state == ProcessState::Stopped;
                let is_running = state.process_state == ProcessState::Running;
                let is_paused = state.process_state == ProcessState::Paused;
                drop(state);

                if ui.add_enabled(is_stopped, egui::Button::new("▶️ Start Detection")).clicked() {
                    self.start_detection();
                }

                if ui.add_enabled(is_running || is_paused, egui::Button::new("⏸️ Pause/Resume")).clicked() {
                    self.pause_detection();
                }

                if ui.add_enabled(!is_stopped, egui::Button::new("⏹️ Stop")).clicked() {
                    self.stop_detection();
                }

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
                            let state = self.state.lock().unwrap();
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
                                        let mut st = self.state.lock().unwrap();
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
                                            let mut st = self.state.lock().unwrap();
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
                                            let mut st = self.state.lock().unwrap();
                                            st.status_message = "Preview playing...".to_string();
                                        }
                                        Err(e) => {
                                            let mut st = self.state.lock().unwrap();
                                            st.status_message = format!("Preview failed: {}", e);
                                        }
                                    }
                                }
                            }
                            None => {
                                let mut st = self.state.lock().unwrap();
                                st.status_message = "Select a music file to preview.".to_string();
                            }
                        }
                    }
                }
            });

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
                                    let mut st = self.state.lock().unwrap();
                                    st.status_message = format!("Failed to save capture: {}", e);
                                } else {
                                    let mut st = self.state.lock().unwrap();
                                    st.status_message = "Saved capture preview to disk".to_string();
                                }
                            }
                        }
                    });
                });
            }

            ui.separator();

            // Configuration section
            ui.heading("⚙️ Configuration");
            
            // Capture region controls
            {
                let mut state = self.state.lock().unwrap();
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
                        let mut state = self.state.lock().unwrap();
                        let capture_height = (screen_h / 4).max(1);
                        let capture_y = screen_h.saturating_sub(capture_height);
                        state.capture_region = [0, capture_y, screen_w, capture_height];
                    }
                }
                ui.label("(Click to select region on screen)");
            });

            // Other configuration options
            {
                let mut state = self.state.lock().unwrap();
                
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

            // Help text
            ui.collapsing("ℹ️ Help", |ui| {
                ui.label("1. Add music files using the '➕ Add Music File' button");
                ui.label("2. Select a music file from the list");
                ui.label("3. Configure capture region and settings");
                ui.label("4. Click '▶️ Start Detection' to begin monitoring");
                ui.label("5. The selected music will play when 'GOAL FOR' is detected");
                ui.label("");
                ui.label("💡 Tip: Use test mode to find the correct capture region:");
                ui.label("   cargo run --release -- --test");
            });
        });

        // Region selector overlay window (implemented inline)
        if self.selecting_region {
            // Initialize on first show
            if let Some(sel) = &mut self.region_selector {
                if !self.hide_window_for_capture {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                    self.hide_window_for_capture = true;
                    self.capture_delay_frames = 2;
                    ctx.request_repaint();
                    return;
                }
                if !sel.initialized {
                    if !self.hide_window_for_capture {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                        self.hide_window_for_capture = true;
                        self.capture_delay_frames = 2;
                        ctx.request_repaint();
                        return;
                    }

                    if self.capture_delay_frames > 0 {
                        self.capture_delay_frames = self.capture_delay_frames.saturating_sub(1);
                        ctx.request_repaint();
                        return;
                    }

                    let capture_result = sel.capture_fullscreen();
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    self.hide_window_for_capture = false;
                    self.capture_delay_frames = 0;

                    match capture_result {
                        Err(e) => {
                            let mut st = self.state.lock().unwrap();
                            st.status_message = format!("Region selection failed: {}", e);
                            self.selecting_region = false;
                            self.region_selector = None;
                            return;
                        }
                        Ok(()) => {
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
                                    let x1 = s.x.min(c.x);
                                    let y1 = s.y.min(c.y);
                                    let x2 = s.x.max(c.x);
                                    let y2 = s.y.max(c.y);

                                    // Convert to screen pixels
                                    let x = (x1 / sel.scale) as u32;
                                    let y = (y1 / sel.scale) as u32;
                                    let w = ((x2 - x1) / sel.scale) as u32;
                                    let h = ((y2 - y1) / sel.scale) as u32;

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
                    let mut st = self.state.lock().unwrap();
                    st.capture_region = [x, y, w, h];
                    st.status_message = format!("Region selected: [{}, {}, {}, {}]", x, y, w, h);
                    // Restore window state
                    if sel.fullscreen_on {
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
    // Raw pixels captured from screen
    img_w: usize,
    img_h: usize,
    pixels_rgba: Option<Vec<u8>>,
    // UI interaction state
    scale: f32,
    start: Option<egui::Pos2>,
    current: Option<egui::Pos2>,
    fullscreen_on: bool,
}

impl RegionSelectState {
    fn capture_fullscreen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Check platform support and permissions
        if !scap::is_supported() {
            return Err("Screen capture not supported on this platform".into());
        }
        if !scap::has_permission() {
            let _ = scap::request_permission();
            if !scap::has_permission() {
                return Err("Screen Recording permission is required. Enable it and restart the app.".into());
            }
        }

        use scap::capturer::{Capturer, Options, Resolution};
        use scap::frame::{Frame, FrameType, VideoFrame};

        let options = Options {
            fps: 1,
            target: None,
            show_cursor: false,
            show_highlight: false,
            excluded_targets: None,
            output_type: FrameType::BGRAFrame,
            output_resolution: Resolution::Captured,
            crop_area: None,
            captures_audio: false,
            ..Default::default()
        };

        let mut capturer = Capturer::build(options)?;
        capturer.start_capture();
        let frame = loop {
            match capturer.get_next_frame()? {
                Frame::Video(f) => break f,
                Frame::Audio(_) => continue,
            }
        };
        capturer.stop_capture();

        let (w, h, rgba) = match frame {
            VideoFrame::BGRA(f) => {
                let mut pixels = f.data.clone();
                for p in pixels.chunks_exact_mut(4) { p.swap(0, 2); }
                (f.width as u32, f.height as u32, pixels)
            }
            VideoFrame::BGR0(f) => {
                let mut pixels = Vec::with_capacity((f.width * f.height * 4) as usize);
                for c in f.data.chunks_exact(4) { pixels.extend_from_slice(&[c[2], c[1], c[0], 255]); }
                (f.width as u32, f.height as u32, pixels)
            }
            _ => return Err("Unsupported video frame format".into()),
        };

        // Defer texture creation to the active egui Context inside the selector window.
        self.img_w = w as usize;
        self.img_h = h as usize;
        self.pixels_rgba = Some(rgba);
        self.texture = None;
        Ok(())
    }
}
