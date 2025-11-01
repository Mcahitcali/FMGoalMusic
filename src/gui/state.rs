/// State module for GUI application
///
/// Contains all state-related structs and enums used by the GUI

use eframe::egui;
use std::path::PathBuf;

use crate::audio::AudioManager;

/// Music entry with file path and optional keyboard shortcut
#[derive(Clone, Debug)]
pub struct MusicEntry {
    pub name: String,
    #[allow(dead_code)]
    pub path: PathBuf,
    pub shortcut: Option<String>,
}

/// Preview audio manager with associated file path
pub(super) struct PreviewAudio {
    pub manager: AudioManager,
    pub path: PathBuf,
}

/// Capture preview state
#[derive(Default)]
pub(super) struct CapturePreview {
    pub texture: Option<egui::TextureHandle>,
    pub last_image: Option<egui::ColorImage>,
    pub width: u32,
    pub height: u32,
    pub timestamp: Option<std::time::Instant>,
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
    pub music_volume: f32,
    pub ambiance_volume: f32,
    pub goal_ambiance_path: Option<String>,
    pub ambiance_enabled: bool,
    pub music_length_ms: u64,
    pub ambiance_length_ms: u64,
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
            music_volume: 1.0,
            ambiance_volume: 0.6,
            goal_ambiance_path: None,
            ambiance_enabled: true,
            music_length_ms: 20000, // 20 seconds
            ambiance_length_ms: 20000, // 20 seconds
        }
    }
}

/// Application tab selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AppTab {
    Library,
    TeamSelection,
    Settings,
    Help,
}

impl AppTab {
    pub const ALL: [AppTab; 4] = [
        AppTab::Library,
        AppTab::TeamSelection,
        AppTab::Settings,
        AppTab::Help,
    ];

    pub fn label(self) -> &'static str {
        match self {
            AppTab::Library => "🎵 Library",
            AppTab::TeamSelection => "⚽ Team Selection",
            AppTab::Settings => "⚙️ Settings",
            AppTab::Help => "ℹ️ Help",
        }
    }
}

/// Save a capture image to disk via file dialog
pub(super) fn save_capture_image(image: &egui::ColorImage) -> Result<(), Box<dyn std::error::Error>> {
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
