/// GUI Model for MVU pattern
///
/// Contains all GUI-specific state separated from application logic.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;

use crossbeam_channel::{Sender, Receiver};
use image::ImageBuffer;
use parking_lot::Mutex;

use crate::config::SelectedTeam;
use crate::state::AppState;
use crate::update_checker::UpdateCheckResult;

use super::state::{AppTab, CapturePreview, PreviewAudio};

/// Region selection state
pub struct RegionSelectState {
    pub start_pos: Option<(f32, f32)>,
}

/// Detection command types
pub enum DetectionCommand {
    StopAudio,
}

/// Update modal display state
pub enum UpdateModalState {
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

/// GUI Model - all state for the MVU pattern
pub struct Model {
    // Core application state (shared with detection thread)
    pub state: Arc<Mutex<AppState>>,

    // Detection thread management
    pub detection_thread: Option<thread::JoinHandle<()>>,
    pub detection_cmd_tx: Option<Sender<DetectionCommand>>,

    // Region selection
    pub selecting_region: bool,
    pub region_selector: Option<RegionSelectState>,
    pub hide_window_for_capture: bool,
    pub capture_delay_frames: u8,

    // Audio preview
    pub preview_audio: Option<PreviewAudio>,
    pub preview_playing: bool,

    // Screen and capture
    pub screen_resolution: Option<(u32, u32)>,
    pub capture_preview: CapturePreview,
    pub latest_capture: Arc<Mutex<Option<(ImageBuffer<image::Rgba<u8>, Vec<u8>>, std::time::Instant)>>>,
    pub capture_dirty: Arc<AtomicBool>,

    // Audio caching
    pub cached_audio_data: Option<(PathBuf, Arc<Vec<u8>>)>,

    // Team database
    pub team_database: Option<crate::teams::TeamDatabase>,
    pub selected_league: Option<String>,
    pub selected_team_key: Option<String>,

    // UI state
    pub active_tab: AppTab,

    // Update checker
    pub update_check_rx: Option<Receiver<UpdateCheckResult>>,
    pub update_modal_state: Option<UpdateModalState>,
}

impl Model {
    /// Create a new GUI model
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        Self {
            state,
            detection_thread: None,
            detection_cmd_tx: None,
            selecting_region: false,
            region_selector: None,
            hide_window_for_capture: false,
            capture_delay_frames: 0,
            preview_audio: None,
            preview_playing: false,
            screen_resolution: None,
            capture_preview: CapturePreview::default(),
            latest_capture: Arc::new(Mutex::new(None)),
            capture_dirty: Arc::new(AtomicBool::new(false)),
            cached_audio_data: None,
            team_database: None,
            selected_league: None,
            selected_team_key: None,
            active_tab: AppTab::Library,
            update_check_rx: None,
            update_modal_state: None,
        }
    }

    /// Get the active tab
    pub fn active_tab(&self) -> AppTab {
        self.active_tab
    }

    /// Set the active tab
    pub fn set_active_tab(&mut self, tab: AppTab) {
        self.active_tab = tab;
    }

    /// Check if detection is running
    pub fn is_detection_running(&self) -> bool {
        let state = self.state.lock();
        state.process_state.is_running()
    }

    /// Check if we're in region selection mode
    pub fn is_selecting_region(&self) -> bool {
        self.selecting_region
    }
}
