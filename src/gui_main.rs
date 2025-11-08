// Hide console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use display_info::DisplayInfo;
use sysinfo::System;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

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
mod slug;
mod state;
mod team_matcher;
mod teams;
mod update_checker;
mod utils;
mod wizard;

const LOG_TARGET_STARTUP: &str = "fm_goal_musics::startup";

fn main() -> Result<()> {
    initialize_tracing();
    log_runtime_environment();
    gui::run()
}

fn initialize_tracing() {
    let log_dir = dirs::config_dir()
        .map(|dir| dir.join("FMGoalMusic").join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("logs"));

    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Warning: Failed to create log directory: {}", e);
    }

    let file_appender = rolling::daily(&log_dir, "fm-goal-musics.log");

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap_or_else(|_| EnvFilter::new("warn"));

    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(true);

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
    let kernel = System::kernel_version().unwrap_or_else(|| "Unknown Kernel".to_string());
    let architecture = std::env::consts::ARCH;

    tracing::info!(target: LOG_TARGET_STARTUP,"Starting FM Goal Musics v{} on ({})", version, architecture);
    tracing::info!(target: LOG_TARGET_STARTUP, "Operating System: {} (kernel {})", os_name, kernel);

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
