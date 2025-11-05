// Hide console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod audio_converter;
mod audio_system;
mod capture;
mod config;
mod detection;
mod error;
mod gui;
mod messaging;
mod ocr;
mod region_selector;
mod slug;
mod state;
mod utils;
mod teams;
mod team_matcher;
mod update_checker;
mod wizard;

use display_info::DisplayInfo;
use sysinfo::System;

const LOG_TARGET_STARTUP: &str = "fm_goal_musics::startup";

/// Initialize tracing with file rotation
///
/// Logs are written to:
/// - macOS: ~/Library/Application Support/FMGoalMusic/logs/
/// - Windows: %APPDATA%/FMGoalMusic/logs/
/// - Linux: ~/.config/FMGoalMusic/logs/
///
/// Log rotation:
/// - Daily rotation (new file each day)
/// - Files named: fm-goal-musics.YYYY-MM-DD.log
///
/// Log output:
/// - Debug builds: Console + File
/// - Release builds: File only (console hidden on Windows)
fn initialize_tracing() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    use tracing_appender::rolling;

    // Get log directory in user config folder
    let log_dir = dirs::config_dir()
        .map(|dir| dir.join("FMGoalMusic").join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("logs"));

    // Create log directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Warning: Failed to create log directory: {}", e);
    }

    // Create file appender with daily rotation
    let file_appender = rolling::daily(&log_dir, "fm-goal-musics.log");

    // Configure filter (info level by default)
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap_or_else(|_| EnvFilter::new("warn"));

    // Create file layer
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(true);

    // In debug builds, also log to console
    #[cfg(debug_assertions)]
    {
        let console_layer = fmt::layer()
            .with_writer(std::io::stdout)
            .with_ansi(true)
            .with_target(false);

        tracing_subscriber::registry()
            .with(filter)
            .with(file_layer)
            .with(console_layer)
            .init();
    }

    // In release builds, only log to file
    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::registry()
            .with(filter)
            .with(file_layer)
            .init();
    }

    tracing::info!("Log directory: {}", log_dir.display());
}

fn log_runtime_environment() {
    let mut system = System::new_all();
    system.refresh_all();

    let version = env!("CARGO_PKG_VERSION");
    let os_name = System::long_os_version()
        .or_else(|| System::name())
        .unwrap_or_else(|| "Unknown OS".to_string());
    let kernel = System::kernel_version()
        .unwrap_or_else(|| "Unknown Kernel".to_string());
    let architecture = std::env::consts::ARCH;

    tracing::info!(target: LOG_TARGET_STARTUP,"Starting FM Goal Musics v{} on ({})", version, architecture);
    tracing::info!(target: LOG_TARGET_STARTUP, "Operating System: {} (kernel {})", os_name, kernel);

    // CPU and memory usage intentionally not logged (per user request)

    if let Ok(displays) = DisplayInfo::all() {
        tracing::info!(
            target: LOG_TARGET_STARTUP,
            "Displays: {} detected",
            displays.len()
        );
        for (index, disp) in displays.iter().enumerate() {
            tracing::debug!(
                target: LOG_TARGET_STARTUP,
                "  Display {}: {}x{}{}",
                index,
                disp.width,
                disp.height,
                if disp.is_primary { " (primary)" } else { "" }
            );
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    // Initialize tracing with file rotation
    initialize_tracing();
    log_runtime_environment();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(
                // Default icon
                eframe::icon_data::from_png_bytes(&[]).unwrap_or_default(),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "FM Goal Musics",
        options,
        Box::new(|cc| Ok(Box::new(gui::FMGoalMusicsApp::new(cc)))),
    )
}
