// Hide console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod audio_converter;
mod audio_system;
mod capture;
mod config;
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

use display_info::DisplayInfo;
use sysinfo::System;

const LOG_TARGET_STARTUP: &str = "fm_goal_musics::startup";

/// Initialize file logging with rotation
///
/// Logs are written to:
/// - macOS: ~/Library/Application Support/FMGoalMusic/logs/
/// - Windows: %APPDATA%/FMGoalMusic/logs/
/// - Linux: ~/.config/FMGoalMusic/logs/
///
/// Log rotation:
/// - Max file size: 10 MB
/// - Keep last 5 log files
/// - Files named: fm-goal-musics_TIMESTAMP.log
///
/// Log output:
/// - Debug builds: Console + File
/// - Release builds: File only (console hidden on Windows)
fn initialize_logging() {
    use flexi_logger::{Duplicate, FileSpec, Logger};

    // Get log directory in user config folder
    let log_dir = dirs::config_dir()
        .map(|dir| dir.join("FMGoalMusic").join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("logs"));

    // Create log directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Warning: Failed to create log directory: {}", e);
    }

    // Configure logger
    let logger = Logger::try_with_str("info")
        .unwrap_or_else(|e| {
            eprintln!("Warning: Failed to parse log level: {}", e);
            Logger::try_with_str("warn").expect("Failed to initialize logger")
        })
        .log_to_file(
            FileSpec::default()
                .directory(&log_dir)
                .basename("fm-goal-musics")
                .suffix("log")
        )
        .rotate(
            flexi_logger::Criterion::Size(10 * 1024 * 1024), // 10 MB
            flexi_logger::Naming::Timestamps,
            flexi_logger::Cleanup::KeepLogFiles(5), // Keep last 5 log files
        )
        .format_for_files(flexi_logger::detailed_format) // Timestamp, level, module, message
        .format_for_stdout(flexi_logger::colored_opt_format); // Colored output for console

    // In debug builds, duplicate to console for easier development
    // In release builds, file only (console is hidden on Windows anyway)
    #[cfg(debug_assertions)]
    let logger = logger.duplicate_to_stdout(Duplicate::Info);

    // Start logger
    if let Err(e) = logger.start() {
        eprintln!("Warning: Failed to initialize file logging: {}", e);
        eprintln!("Logs will not be written to file.");
    } else {
        // Log the log file location (will appear in the log file itself)
        log::info!("Log directory: {}", log_dir.display());
    }
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

    log::info!(target: LOG_TARGET_STARTUP,"Starting FM Goal Musics v{} on ({})", version, architecture);
    log::info!(target: LOG_TARGET_STARTUP, "Operating System: {} (kernel {})", os_name, kernel);

    // CPU and memory usage intentionally not logged (per user request)

    if let Ok(displays) = DisplayInfo::all() {
        log::info!(
            target: LOG_TARGET_STARTUP,
            "Displays: {} detected",
            displays.len()
        );
        for (index, display) in displays.iter().enumerate() {
            log::debug!(
                target: LOG_TARGET_STARTUP,
                "  Display {}: {}x{}{}",
                index,
                display.width,
                display.height,
                if display.is_primary { " (primary)" } else { "" }
            );
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    // Initialize file logging with rotation
    initialize_logging();
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
