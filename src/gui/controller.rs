use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};
use dirs::config_dir;
use image::DynamicImage;
use parking_lot::Mutex;

use crate::audio::AudioManager;
use crate::audio_converter;
use crate::capture::{CaptureManager, CaptureRegion};
use crate::config::{Config, MusicEntry as ConfigMusicEntry, SelectedTeam};
use crate::detection::i18n::Language;
use crate::ocr::OcrManager;
use crate::slug::slugify;
use crate::state::{AppState, MusicEntry, ProcessState};
use crate::team_matcher::TeamMatcher;
use crate::teams::{Team, TeamDatabase};
use crate::update_checker::{self, UpdateCheckResult};
use crate::utils::Debouncer;
use tracing::{error, info, warn};
use xcap::Monitor;

#[derive(Clone, Debug)]
pub struct MonitorSummary {
    pub index: usize,
    pub label: String,
}

const DEFAULT_BENCH_FRAMES: usize = 500;
const AUDIO_FADE_MS: u64 = 200;

enum DetectionCommand {
    Stop,
    StopAudio,
}

#[derive(Clone)]
pub struct GuiController {
    inner: Arc<ControllerInner>,
}

struct ControllerInner {
    state: Arc<Mutex<AppState>>,
    team_database: Mutex<Option<TeamDatabase>>,
    detection_thread: Mutex<Option<thread::JoinHandle<()>>>,
    detection_cmd_tx: Mutex<Option<Sender<DetectionCommand>>>,
}

impl GuiController {
    pub fn new() -> Result<Self> {
        let state = Arc::new(Mutex::new(AppState::default()));
        let team_database = TeamDatabase::load().ok();

        if let Ok(config) = Config::load() {
            apply_config(&state, &config);
        }

        Ok(Self {
            inner: Arc::new(ControllerInner {
                state,
                team_database: Mutex::new(team_database),
                detection_thread: Mutex::new(None),
                detection_cmd_tx: Mutex::new(None),
            }),
        })
    }

    pub fn state(&self) -> Arc<Mutex<AppState>> {
        Arc::clone(&self.inner.state)
    }

    pub fn team_database(&self) -> Option<TeamDatabase> {
        self.inner.team_database.lock().clone()
    }

    pub fn status_message(&self) -> String {
        self.inner.state.lock().status_message.clone()
    }

    pub fn monitor_summaries(&self) -> Vec<MonitorSummary> {
        match Monitor::all() {
            Ok(monitors) if !monitors.is_empty() => monitors
                .into_iter()
                .enumerate()
                .map(|(idx, monitor)| {
                    let width = monitor.width().unwrap_or(0);
                    let height = monitor.height().unwrap_or(0);
                    let name = monitor.name().unwrap_or_else(|_| "Unknown".to_string());
                    MonitorSummary {
                        index: idx,
                        label: format!("Display {} · {}x{} ({})", idx + 1, width, height, name),
                    }
                })
                .collect(),
            _ => vec![MonitorSummary {
                index: 0,
                label: "Display 1".to_string(),
            }],
        }
    }

    fn shutdown_detection_runtime(&self) {
        if let Some(tx) = self.inner.detection_cmd_tx.lock().take() {
            let _ = tx.send(DetectionCommand::Stop);
        }
        if let Some(handle) = self.inner.detection_thread.lock().take() {
            let _ = handle.join();
        }
    }

    fn mark_start_failure(&self, message: impl Into<String>) {
        let mut state = self.inner.state.lock();
        state.process_state = ProcessState::Stopped;
        state.status_message = message.into();
    }

    pub fn capture_preview(&self) -> Result<PathBuf> {
        let (region, monitor_index, generation) = {
            let state = self.inner.state.lock();
            (
                state.capture_region,
                state.selected_monitor_index,
                state.preview_generation,
            )
        };

        let mut capture_manager =
            CaptureManager::new(CaptureRegion::from_array(region), monitor_index)
                .map_err(|err| anyhow!("Preview capture init failed: {err}"))?;
        let image = capture_manager
            .capture_region()
            .map_err(|err| anyhow!("Failed to capture preview: {err}"))?;

        // Use generation counter in filename to bust cache
        let new_generation = generation.wrapping_add(1);
        let preview_path = preview_image_path_with_generation(new_generation)?;

        DynamicImage::ImageRgba8(image)
            .save(&preview_path)
            .map_err(|err| anyhow!("Failed to save preview: {err}"))?;

        // Clean up old preview files
        if let Ok(old_path) = preview_image_path_with_generation(generation) {
            let _ = fs::remove_file(old_path); // Ignore errors if file doesn't exist
        }
        // Also clean up the one before that (in case of rapid clicks)
        if generation > 0 {
            if let Ok(old_path) = preview_image_path_with_generation(generation - 1) {
                let _ = fs::remove_file(old_path);
            }
        }

        {
            let mut state = self.inner.state.lock();
            state.preview_image_path = Some(preview_path.clone());
            state.preview_generation = new_generation;
            state.status_message = "Capture preview updated".into();
        }

        Ok(preview_path)
    }

    pub fn capture_fullscreen_for_selection(&self) -> Result<RegionCapture> {
        let selected_monitor = {
            let state = self.inner.state.lock();
            state.selected_monitor_index
        };

        let monitors =
            Monitor::all().map_err(|err| anyhow!("Failed to enumerate monitors: {err}"))?;
        if monitors.is_empty() {
            return Err(anyhow!("No monitors detected"));
        }

        let monitor = monitors
            .into_iter()
            .nth(selected_monitor)
            .or_else(|| Monitor::all().ok()?.into_iter().next())
            .ok_or_else(|| anyhow!("Unable to acquire monitor handle"))?;

        let logical_w = monitor.width().unwrap_or(0);
        let logical_h = monitor.height().unwrap_or(0);
        let image = monitor
            .capture_image()
            .map_err(|err| anyhow!("Failed to capture screen: {err}"))?;
        let physical_w = image.width();
        let physical_h = image.height();
        let device_scale = if logical_w > 0 {
            physical_w as f32 / logical_w as f32
        } else {
            1.0
        };

        let path = region_selection_image_path()?;
        DynamicImage::ImageRgba8(image)
            .save(&path)
            .map_err(|err| anyhow!("Failed to save capture snapshot: {err}"))?;

        Ok(RegionCapture {
            image_path: path,
            physical_size: (physical_w, physical_h),
            logical_size: (logical_w, logical_h),
            device_scale,
        })
    }

    pub fn add_music_file(&self, path: PathBuf) -> Result<()> {
        let final_path = audio_converter::convert_to_wav(&path)
            .map_err(|err| anyhow!("failed to convert audio {}: {err}", path.display()))?;

        let name = final_path
            .file_stem()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        {
            let mut state = self.inner.state.lock();
            state.music_list.push(MusicEntry {
                name,
                path: final_path,
                shortcut: None,
            });
            state.selected_music_index = Some(state.music_list.len().saturating_sub(1));
            state.status_message = "Music file added".to_string();
        }

        self.save_config()?;
        Ok(())
    }

    pub fn remove_music(&self, index: usize) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            if index < state.music_list.len() {
                state.music_list.remove(index);
                if state.selected_music_index == Some(index) {
                    state.selected_music_index = None;
                } else if let Some(sel) = state.selected_music_index {
                    if index < sel {
                        state.selected_music_index = Some(sel - 1);
                    }
                }
                state.status_message = "Music entry removed".to_string();
            }
        }

        self.save_config()?;
        Ok(())
    }

    pub fn select_music(&self, index: Option<usize>) {
        let mut state = self.inner.state.lock();
        state.selected_music_index = index;
        state.status_message = match index {
            Some(idx) => format!("Selected music #{idx}"),
            None => "No music selected".to_string(),
        };
    }

    pub fn set_league(&self, league: Option<String>) {
        {
            let mut state = self.inner.state.lock();
            if league.is_none() {
                state.selected_team = None;
            }
        }

        if let Some(team_db) = self.team_database() {
            let mut data = self.inner.state.lock();
            if let Some(league_name) = league {
                if !team_db.get_leagues().contains(&league_name) {
                    data.status_message = format!("League '{league_name}' not found");
                } else {
                    if let Some(selected) = data
                        .selected_team
                        .as_mut()
                        .filter(|team| team.league != league_name)
                    {
                        selected.league = league_name.clone();
                        selected.team_key.clear();
                    }
                    data.status_message = format!("League set to {league_name}");
                }
            } else {
                data.status_message = "Cleared league selection".to_string();
            }
        }
    }

    pub fn select_team(&self, league: &str, team_key: &str) -> Result<()> {
        let team_db = self
            .team_database()
            .ok_or_else(|| anyhow::anyhow!("team database not available"))?;

        let team = team_db
            .find_team(league, team_key)
            .ok_or_else(|| anyhow::anyhow!("team not found in database"))?;

        {
            let mut state = self.inner.state.lock();
            state.selected_team = Some(SelectedTeam {
                league: league.to_string(),
                team_key: team_key.to_string(),
                display_name: team.display_name.clone(),
            });
            state.status_message = format!("Selected team {}", team.display_name);
        }

        self.save_config()?;
        Ok(())
    }

    pub fn add_custom_team(
        &self,
        league_name: String,
        team_name: String,
        variations: Vec<String>,
        allow_create_league: bool,
        logo_path: Option<PathBuf>,
    ) -> Result<SelectedTeam> {
        let mut guard = self.inner.team_database.lock();
        let db = guard
            .as_mut()
            .ok_or_else(|| anyhow!("team database not available"))?;

        if !db.has_league(&league_name) {
            if allow_create_league {
                db.add_league(league_name.clone()).map_err(|e| anyhow!(e))?;
            } else {
                return Err(anyhow!("League '{league_name}' not found"));
            }
        }

        let team_key = slugify(&team_name);
        let team = Team {
            display_name: team_name.clone(),
            variations,
        };

        db.add_team(league_name.clone(), team_key.clone(), team)
            .map_err(|e| anyhow!(e))?;
        db.save()
            .map_err(|err| anyhow!("failed to save team database: {err}"))?;

        if let Some(path) = logo_path {
            self.save_team_logo(&league_name, &team_key, &path)?;
        }

        let selected_team = SelectedTeam {
            league: league_name.clone(),
            team_key: team_key.clone(),
            display_name: team_name.clone(),
        };

        {
            let mut state = self.inner.state.lock();
            state.selected_team = Some(selected_team.clone());
            state.status_message = format!("Added {team_name} to {league_name}");
        }

        self.save_config()?;
        Ok(selected_team)
    }

    fn save_team_logo(
        &self,
        league_name: &str,
        team_key: &str,
        source_path: &PathBuf,
    ) -> Result<()> {
        if !source_path.exists() {
            return Err(anyhow!(format!(
                "Logo file does not exist: {}",
                source_path.display()
            )));
        }

        let extension_valid = source_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("png"))
            .unwrap_or(false);

        if !extension_valid {
            return Err(anyhow!("Team logo must be a PNG file"));
        }

        let base_dir = config_dir()
            .ok_or_else(|| anyhow!("Could not determine user config directory"))?
            .join("FMGoalMusic")
            .join("teams")
            .join(league_name);

        fs::create_dir_all(&base_dir)
            .with_context(|| format!("Failed to create logo directory: {}", base_dir.display()))?;

        let target_path = base_dir.join(format!("{}.png", team_key));

        fs::copy(source_path, &target_path).with_context(|| {
            format!(
                "Failed to copy logo from {} to {}",
                source_path.display(),
                target_path.display()
            )
        })?;

        info!(
            "[logo] Saved custom team logo for {} ({}) to {}",
            team_key,
            league_name,
            target_path.display()
        );

        Ok(())
    }

    pub fn clear_team_selection(&self) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            state.selected_team = None;
            state.status_message = "Team selection cleared".to_string();
        }
        self.save_config()?;
        Ok(())
    }

    pub fn update_capture_region(&self, region: [u32; 4]) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            state.capture_region = region;
            state.status_message = format!(
                "Capture region set to [{}, {}, {}, {}]",
                region[0], region[1], region[2], region[3]
            );
        }
        self.save_config()?;
        Ok(())
    }

    pub fn set_monitor_index(&self, index: usize) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            state.selected_monitor_index = index;
            state.status_message = format!("Monitor set to {}", index + 1);
        }
        self.save_config()?;
        Ok(())
    }

    pub fn set_status(&self, message: impl Into<String>) {
        let mut state = self.inner.state.lock();
        state.status_message = message.into();
    }

    pub fn config_file_path(&self) -> Option<PathBuf> {
        Config::config_path().ok()
    }

    pub fn logs_directory(&self) -> Option<PathBuf> {
        config_dir().map(|dir| dir.join("FMGoalMusic").join("logs"))
    }

    pub fn start_monitoring(&self) -> Result<()> {
        self.shutdown_detection_runtime();

        let setup = {
            let state = self.inner.state.lock();
            if state.process_state.is_running() {
                drop(state);
                self.set_status("Detection already running");
                return Ok(());
            }

            state
                .validate_region()
                .map_err(|err| anyhow!(err.to_string()))?;
            state
                .validate_music_selection()
                .map_err(|err| anyhow!(err.to_string()))?;

            let selected_idx = state
                .selected_music_index
                .ok_or_else(|| anyhow!("Select a music file before starting detection"))?;
            let music_entry = state
                .music_list
                .get(selected_idx)
                .ok_or_else(|| anyhow!("Selected music entry no longer exists"))?
                .clone();

            DetectionSetup {
                music_entry,
                capture_region: state.capture_region,
                monitor_index: state.selected_monitor_index,
                ocr_threshold: state.ocr_threshold,
                enable_morph_open: state.enable_morph_open,
                debounce_ms: state.debounce_ms,
                selected_team: state.selected_team.clone(),
                music_volume: state.music_volume,
                ambiance_volume: state.ambiance_volume,
                ambiance_path: state.goal_ambiance_path.clone(),
                ambiance_enabled: state.ambiance_enabled,
                music_length_ms: state.music_length_ms,
                ambiance_length_ms: state.ambiance_length_ms,
                custom_goal_phrases: state.custom_goal_phrases.clone(),
            }
        };

        let music_bytes = match fs::read(&setup.music_entry.path)
            .with_context(|| format!("Failed to read audio {}", setup.music_entry.path.display()))
        {
            Ok(bytes) => Arc::new(bytes),
            Err(err) => {
                self.mark_start_failure(format!("{err:#}"));
                return Err(err);
            }
        };

        let ambiance_bytes = if setup.ambiance_enabled {
            if let Some(ref path) = setup.ambiance_path {
                match fs::read(path).with_context(|| format!("Failed to read ambiance {}", path)) {
                    Ok(bytes) => Some(Arc::new(bytes)),
                    Err(err) => {
                        self.mark_start_failure(format!("{err:#}"));
                        return Err(err);
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        let team_profile = setup.selected_team.as_ref().and_then(|team| {
            self.team_database()
                .and_then(|db| db.find_team(&team.league, &team.team_key))
        });

        let track_name = setup.music_entry.name.clone();

        {
            let mut state = self.inner.state.lock();
            state.process_state = ProcessState::Running {
                since: Instant::now(),
            };
            state.detection_count = 0;
            state.status_message = format!("Monitoring goals — will play '{}'", track_name);
        }

        let (cmd_tx, cmd_rx) = unbounded();
        {
            let mut tx_slot = self.inner.detection_cmd_tx.lock();
            *tx_slot = Some(cmd_tx);
        }

        let state_arc = Arc::clone(&self.inner.state);
        let handle = thread::spawn(move || {
            if let Err(err) = run_detection_loop(
                state_arc,
                cmd_rx,
                setup,
                music_bytes,
                ambiance_bytes,
                team_profile,
            ) {
                error!("Detection loop exited with error: {err:#}");
            }
        });

        let mut thread_slot = self.inner.detection_thread.lock();
        *thread_slot = Some(handle);

        Ok(())
    }

    pub fn stop_monitoring(&self) -> Result<()> {
        self.shutdown_detection_runtime();
        let mut state = self.inner.state.lock();
        state.process_state = ProcessState::Stopped;
        state.status_message = "Monitoring stopped".to_string();
        Ok(())
    }

    /// Stop currently playing goal celebration music without stopping monitoring
    pub fn stop_goal_music(&self) -> Result<()> {
        if let Some(tx) = self.inner.detection_cmd_tx.lock().as_ref() {
            tx.send(DetectionCommand::StopAudio)
                .map_err(|e| anyhow::anyhow!("Failed to send stop audio command: {}", e))?;
            let mut state = self.inner.state.lock();
            state.status_message = "Goal music stopped".to_string();
        }
        Ok(())
    }

    pub fn set_ambiance_enabled(&self, enabled: bool) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            state.ambiance_enabled = enabled;
            state.status_message = if enabled {
                "Ambiance enabled".to_string()
            } else {
                "Ambiance disabled".to_string()
            };
        }
        self.save_config()
    }

    pub fn set_goal_ambiance_path(&self, path: Option<PathBuf>) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            state.goal_ambiance_path = path.as_ref().map(|p| p.to_string_lossy().to_string());
            state.status_message = match &state.goal_ambiance_path {
                Some(p) => format!("Ambiance sound set to {}", p),
                None => "Ambiance sound cleared".to_string(),
            };
        }
        self.save_config()
    }

    pub fn set_music_volume(&self, volume: f32) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            state.music_volume = volume.clamp(0.0, 1.0);
            state.status_message = format!(
                "Music volume set to {}%",
                (state.music_volume * 100.0).round()
            );
        }
        self.save_config()
    }

    pub fn set_ambiance_volume(&self, volume: f32) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            state.ambiance_volume = volume.clamp(0.0, 1.0);
            state.status_message = format!(
                "Ambiance volume set to {}%",
                (state.ambiance_volume * 100.0).round()
            );
        }
        self.save_config()
    }

    pub fn set_music_length_ms(&self, length_ms: u64) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            state.music_length_ms = length_ms.clamp(1_000, 120_000);
            state.status_message = format!(
                "Music playback length set to {}s",
                state.music_length_ms / 1000
            );
        }
        self.save_config()
    }

    pub fn set_ambiance_length_ms(&self, length_ms: u64) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            state.ambiance_length_ms = length_ms.clamp(1_000, 120_000);
            state.status_message = format!(
                "Ambiance playback length set to {}s",
                state.ambiance_length_ms / 1000
            );
        }
        self.save_config()
    }

    pub fn set_ocr_threshold(&self, value: i16) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            let clamped = value.clamp(0, 255) as u8;
            state.ocr_threshold = clamped;
            if clamped == 0 {
                state.status_message = "OCR threshold set to Auto (Otsu)".to_string();
            } else {
                state.status_message = format!("OCR threshold set to {}", clamped);
            }
        }
        self.save_config()
    }

    pub fn adjust_ocr_threshold(&self, delta: i16) -> Result<()> {
        let current = { self.inner.state.lock().ocr_threshold };
        self.set_ocr_threshold(current as i16 + delta)
    }

    pub fn set_debounce_ms(&self, debounce_ms: u64) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            state.debounce_ms = debounce_ms.clamp(100, 60_000);
            state.status_message = format!("Debounce set to {} ms", state.debounce_ms);
        }
        self.save_config()
    }

    pub fn adjust_debounce_ms(&self, delta: i64) -> Result<()> {
        let current = { self.inner.state.lock().debounce_ms };
        let new_val = (current as i64 + delta).clamp(100, 60_000) as u64;
        self.set_debounce_ms(new_val)
    }

    pub fn set_morph_open(&self, enabled: bool) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            state.enable_morph_open = enabled;
            state.status_message = if enabled {
                "Morphological opening enabled".to_string()
            } else {
                "Morphological opening disabled".to_string()
            };
        }
        self.save_config()
    }

    pub fn set_auto_check_updates(&self, enabled: bool) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            state.auto_check_updates = enabled;
            state.status_message = if enabled {
                "Will check for updates on startup".to_string()
            } else {
                "Auto update checks disabled".to_string()
            };
        }
        self.save_config()
    }

    pub fn adjust_capture_region(&self, index: usize, delta: i32) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            if let Some(value) = state.capture_region.get_mut(index) {
                let min_value = if index >= 2 { 1 } else { 0 };
                let new_value = (*value as i64 + delta as i64).max(min_value as i64) as u32;
                *value = new_value;
                state.status_message = format!(
                    "Capture region updated to [{}, {}, {}, {}]",
                    state.capture_region[0],
                    state.capture_region[1],
                    state.capture_region[2],
                    state.capture_region[3]
                );
            }
        }
        self.save_config()
    }

    pub fn reset_capture_region(&self, region: [u32; 4]) -> Result<()> {
        self.update_capture_region(region)
    }

    pub fn set_selected_language(&self, language: Language) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            state.selected_language = language;
            state.status_message = format!("Language set to {}", language.name());
        }
        self.save_config()
    }

    pub fn get_available_languages() -> Vec<(Language, String)> {
        vec![
            (Language::English, Language::English.name().to_string()),
            (Language::Turkish, Language::Turkish.name().to_string()),
            (Language::Spanish, Language::Spanish.name().to_string()),
            (Language::French, Language::French.name().to_string()),
            (Language::German, Language::German.name().to_string()),
            (Language::Italian, Language::Italian.name().to_string()),
            (
                Language::Portuguese,
                Language::Portuguese.name().to_string(),
            ),
        ]
    }

    pub fn add_custom_goal_phrase(&self, phrase: String) -> Result<()> {
        // Validate phrase
        let trimmed = phrase.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("Phrase cannot be empty"));
        }
        if trimmed.len() > 100 {
            return Err(anyhow!("Phrase too long (max 100 characters)"));
        }

        {
            let mut state = self.inner.state.lock();
            // Check if phrase already exists
            if state
                .custom_goal_phrases
                .iter()
                .any(|p| p.eq_ignore_ascii_case(trimmed))
            {
                return Err(anyhow!("Phrase already exists"));
            }
            state.custom_goal_phrases.push(trimmed.to_string());
            state.status_message = format!("Custom goal phrase added: '{}'", trimmed);
        }

        self.save_config()
    }

    pub fn remove_custom_goal_phrase(&self, phrase: &str) -> Result<()> {
        {
            let mut state = self.inner.state.lock();
            state
                .custom_goal_phrases
                .retain(|p| !p.eq_ignore_ascii_case(phrase));
            state.status_message = format!("Custom goal phrase removed: '{}'", phrase);
        }
        self.save_config()
    }

    pub fn get_custom_goal_phrases(&self) -> Vec<String> {
        self.inner.state.lock().custom_goal_phrases.clone()
    }

    pub fn check_for_updates(&self) {
        self.set_status("Checking for updates...");
        let state = Arc::clone(&self.inner.state);
        thread::spawn(move || {
            let result = update_checker::check_for_updates();
            let mut guard = state.lock();
            guard.status_message = match result {
                UpdateCheckResult::UpdateAvailable {
                    latest_version,
                    current_version,
                    ..
                } => format!("Update available: {current_version} → {latest_version}"),
                UpdateCheckResult::UpToDate { current_version } => {
                    format!("Up to date (v{current_version})")
                }
                UpdateCheckResult::Error { message } => {
                    format!("Update check failed: {message}")
                }
            };
        });
    }

    fn save_config(&self) -> Result<()> {
        let state = self.inner.state.lock();
        let config = Config {
            capture_region: state.capture_region,
            ocr_threshold: state.ocr_threshold,
            debounce_ms: state.debounce_ms,
            enable_morph_open: state.enable_morph_open,
            bench_frames: DEFAULT_BENCH_FRAMES,
            music_list: state
                .music_list
                .iter()
                .map(|entry| ConfigMusicEntry {
                    name: entry.name.clone(),
                    path: entry.path.to_string_lossy().to_string(),
                    shortcut: entry.shortcut.clone(),
                })
                .collect(),
            selected_music_index: state.selected_music_index,
            goal_ambiance_path: state.goal_ambiance_path.clone(),
            ambiance_enabled: state.ambiance_enabled,
            music_volume: state.music_volume,
            ambiance_volume: state.ambiance_volume,
            music_length_ms: state.music_length_ms,
            ambiance_length_ms: state.ambiance_length_ms,
            selected_team: state.selected_team.clone(),
            auto_check_updates: state.auto_check_updates,
            skipped_version: state.skipped_version.clone(),
            selected_monitor_index: state.selected_monitor_index,
            selected_language: state.selected_language,
            custom_goal_phrases: state.custom_goal_phrases.clone(),
        };
        drop(state);

        config
            .save()
            .map_err(|err| anyhow!("failed to save config: {err}"))
    }
}

fn apply_config(state: &Arc<Mutex<AppState>>, config: &Config) {
    let mut st = state.lock();
    st.capture_region = config.capture_region;
    st.ocr_threshold = config.ocr_threshold;
    st.debounce_ms = config.debounce_ms;
    st.enable_morph_open = config.enable_morph_open;
    st.music_list = config
        .music_list
        .iter()
        .map(|entry| MusicEntry {
            name: entry.name.clone(),
            path: PathBuf::from(&entry.path),
            shortcut: entry.shortcut.clone(),
        })
        .collect();
    st.selected_music_index = config.selected_music_index;
    st.goal_ambiance_path = config.goal_ambiance_path.clone();
    st.ambiance_enabled = config.ambiance_enabled;
    st.music_volume = config.music_volume;
    st.ambiance_volume = config.ambiance_volume;
    st.music_length_ms = config.music_length_ms;
    st.ambiance_length_ms = config.ambiance_length_ms;
    st.selected_team = config.selected_team.clone();
    st.auto_check_updates = config.auto_check_updates;
    st.skipped_version = config.skipped_version.clone();
    st.selected_monitor_index = config.selected_monitor_index;
    st.selected_language = config.selected_language;
    st.custom_goal_phrases = config.custom_goal_phrases.clone();
    st.status_message = "Ready".to_string();
    st.process_state = ProcessState::Stopped;
    st.preview_image_path = None;
}

struct DetectionSetup {
    music_entry: MusicEntry,
    capture_region: [u32; 4],
    monitor_index: usize,
    ocr_threshold: u8,
    enable_morph_open: bool,
    debounce_ms: u64,
    selected_team: Option<SelectedTeam>,
    music_volume: f32,
    ambiance_volume: f32,
    ambiance_path: Option<String>,
    ambiance_enabled: bool,
    music_length_ms: u64,
    ambiance_length_ms: u64,
    custom_goal_phrases: Vec<String>,
}

pub struct RegionCapture {
    pub image_path: PathBuf,
    pub physical_size: (u32, u32),
    pub logical_size: (u32, u32),
    pub device_scale: f32,
}

fn preview_image_path() -> Result<PathBuf> {
    let base = config_dir().ok_or_else(|| anyhow!("Unable to locate config directory"))?;
    let dir = base.join("FMGoalMusic").join("previews");
    fs::create_dir_all(&dir).context("Failed to create preview directory")?;
    Ok(dir.join("capture_preview.png"))
}

fn preview_image_path_with_generation(generation: u32) -> Result<PathBuf> {
    let base = config_dir().ok_or_else(|| anyhow!("Unable to locate config directory"))?;
    let dir = base.join("FMGoalMusic").join("previews");
    fs::create_dir_all(&dir).context("Failed to create preview directory")?;
    Ok(dir.join(format!("capture_preview_{}.png", generation)))
}

fn region_selection_image_path() -> Result<PathBuf> {
    let base = config_dir().ok_or_else(|| anyhow!("Unable to locate config directory"))?;
    let dir = base.join("FMGoalMusic").join("previews");
    fs::create_dir_all(&dir).context("Failed to create preview directory")?;
    Ok(dir.join("region_selection.png"))
}

fn run_detection_loop(
    state: Arc<Mutex<AppState>>,
    cmd_rx: Receiver<DetectionCommand>,
    setup: DetectionSetup,
    music_bytes: Arc<Vec<u8>>,
    ambiance_bytes: Option<Arc<Vec<u8>>>,
    team_profile: Option<Team>,
) -> Result<()> {
    let DetectionSetup {
        music_entry,
        capture_region,
        monitor_index,
        ocr_threshold,
        enable_morph_open,
        debounce_ms,
        selected_team,
        music_volume,
        ambiance_volume,
        ambiance_path: _,
        ambiance_enabled,
        music_length_ms,
        ambiance_length_ms,
        custom_goal_phrases,
    } = setup;

    let music_name = music_entry.name.clone();

    let audio_manager = AudioManager::from_preloaded(music_bytes)
        .map_err(|err| anyhow!("Failed to initialize audio output: {err}"))?;
    audio_manager.set_volume(music_volume);

    let ambiance_manager = if ambiance_enabled {
        if let Some(bytes) = ambiance_bytes {
            let manager = AudioManager::from_preloaded(bytes)
                .map_err(|err| anyhow!("Failed to initialize ambiance audio: {err}"))?;
            manager.set_volume(ambiance_volume);
            Some(manager)
        } else {
            None
        }
    } else {
        None
    };

    let mut capture_manager =
        CaptureManager::new(CaptureRegion::from_array(capture_region), monitor_index)
            .map_err(|err| anyhow!("Failed to initialize capture manager: {err}"))?;

    let mut ocr_manager = OcrManager::new_with_options(ocr_threshold, enable_morph_open)
        .map_err(|err| anyhow!("Failed to initialize OCR manager: {err}"))?;

    let team_matcher = team_profile.as_ref().map(|team| TeamMatcher::new(team));
    if let Some(team) = &team_profile {
        info!("Team-specific monitoring enabled for {}", team.display_name);
    }

    let mut debouncer = Debouncer::new(debounce_ms.max(100));

    loop {
        match cmd_rx.try_recv() {
            Ok(DetectionCommand::Stop) => {
                audio_manager.stop();
                if let Some(ref ambiance) = ambiance_manager {
                    ambiance.stop();
                }
                let mut st = state.lock();
                st.process_state = ProcessState::Stopped;
                st.status_message = "Monitoring stopped".to_string();
                return Ok(());
            }
            Ok(DetectionCommand::StopAudio) => {
                audio_manager.stop();
                if let Some(ref ambiance) = ambiance_manager {
                    ambiance.stop();
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                let mut st = state.lock();
                st.process_state = ProcessState::Stopped;
                st.status_message = "Monitoring stopped".to_string();
                return Ok(());
            }
        }

        let image = match capture_manager.capture_region() {
            Ok(img) => img,
            Err(err) => {
                let mut st = state.lock();
                st.process_state = ProcessState::Stopped;
                st.status_message = format!("Capture error: {err}");
                return Err(anyhow!("Capture error: {err}"));
            }
        };

        let should_play = if let Some(matcher) = &team_matcher {
            match ocr_manager.detect_goal_with_team(&image) {
                Ok(Some(team_name)) => {
                    if matcher.matches(&team_name) {
                        info!("Goal detected for selected team: {}", team_name);
                        true
                    } else {
                        false
                    }
                }
                Ok(None) => false,
                Err(err) => {
                    warn!("OCR error: {err}");
                    false
                }
            }
        } else {
            // Use custom phrases if available, otherwise use standard detection
            if !custom_goal_phrases.is_empty() {
                match ocr_manager.detect_goal_with_custom_phrases(&image, &custom_goal_phrases) {
                    Ok(result) => result,
                    Err(err) => {
                        warn!("OCR error: {err}");
                        false
                    }
                }
            } else {
                match ocr_manager.detect_goal(&image) {
                    Ok(result) => result,
                    Err(err) => {
                        warn!("OCR error: {err}");
                        false
                    }
                }
            }
        };

        if should_play && debouncer.should_trigger() {
            if let Some(ref ambiance) = ambiance_manager {
                let result = if ambiance_length_ms > 0 {
                    ambiance.play_sound_with_fade_and_limit(AUDIO_FADE_MS, ambiance_length_ms)
                } else {
                    ambiance.play_sound_with_fade(AUDIO_FADE_MS)
                };
                if let Err(err) = result {
                    warn!("Failed to play ambiance: {err}");
                }
            }

            let music_result = if music_length_ms > 0 {
                audio_manager.play_sound_with_fade_and_limit(AUDIO_FADE_MS, music_length_ms)
            } else {
                audio_manager.play_sound_with_fade(AUDIO_FADE_MS)
            };

            if let Err(err) = music_result {
                let mut st = state.lock();
                st.status_message = format!("Failed to play music: {err}");
            } else {
                let mut st = state.lock();
                st.detection_count += 1;
                let ambiance_note = if ambiance_manager.is_some() {
                    " + crowd cheer"
                } else {
                    ""
                };
                st.status_message = format!(
                    "Goal detected! Played '{}'{} (total: {})",
                    music_name, ambiance_note, st.detection_count
                );

                if let Some(team) = &selected_team {
                    info!(
                        "Goal #{} for {} ({}) detected",
                        st.detection_count, team.display_name, team.league
                    );
                }
            }
        }

        thread::sleep(Duration::from_millis(16));
    }
}
