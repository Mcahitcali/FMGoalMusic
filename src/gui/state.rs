/// GUI-specific state module
///
/// Contains GUI-only state types (preview, audio, tabs, etc.)
/// Main application state is now in src/state/

use eframe::egui;
use std::path::PathBuf;

use crate::audio::AudioManager;

// Re-export main state types for compatibility
pub use crate::state::{AppState, MusicEntry, ProcessState};

// Re-export SelectedTeam from config for now (will migrate later)
pub use crate::config::SelectedTeam;

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
pub(super) fn save_capture_image(
    image: &egui::ColorImage,
) -> Result<(), Box<dyn std::error::Error>> {
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
