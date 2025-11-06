/// Settings view - Configuration options
///
/// Displays all configurable settings for the application.

use eframe::egui;
use std::time::Instant;

use crate::gui::state::save_capture_image;
use crate::gui::theme;

use super::super::FMGoalMusicsApp;

/// Render the settings tab
pub fn render_settings(app: &mut FMGoalMusicsApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    // ========================================
    // GROUP 1: Display & Capture
    // ========================================
    theme::card_frame().show(ui, |ui| {
        theme::styled_heading(ui, "🖥️ Display & Capture");
        theme::add_space_small(ui);

    // Monitor selection (multi-monitor support)
    ui.horizontal(|ui| {
        ui.label("Display:");
        let mut state = app.state.lock();

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
            app.save_config();
        }
    });
    ui.label("💡 Select which monitor to capture from");

    ui.add_space(5.0);

    // Capture region controls
    {
        let mut state = app.state.lock();
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
            app.start_region_selection();
        }
        if ui.button("🔄 Reset Region").clicked() {
            if let Some((screen_w, screen_h)) = app.screen_resolution {
                let mut state = app.state.lock();
                let capture_height = (screen_h / 4).max(1);
                let capture_y = screen_h.saturating_sub(capture_height);
                state.capture_region = [0, capture_y, screen_w, capture_height];
            }
        }
    });
    ui.label("💡 Use visual selector for accurate coordinates on HiDPI/Retina displays");

    ui.add_space(5.0);

    // Capture preview
    app.refresh_capture_preview(ctx);
    if let Some(texture) = &app.capture_preview.texture {
        ui.group(|ui| {
            ui.label("📷 Capture Preview");
            let aspect = texture.size()[0] as f32 / texture.size()[1] as f32;
            let max_width = ui.available_width().min(400.0);
            let desired_size = egui::Vec2::new(max_width, max_width / aspect);
            ui.image(egui::load::SizedTexture::new(texture.id(), desired_size));

            ui.horizontal(|ui| {
                ui.label(format!(
                    "Resolution: {}x{}",
                    app.capture_preview.width, app.capture_preview.height
                ));

                if let Some(ts) = app.capture_preview.timestamp {
                    let age = Instant::now().saturating_duration_since(ts);
                    ui.label(format!("Age: {:.1}s", age.as_secs_f32()));
                }

                if ui.button("Save frame...").clicked() {
                    if let Some(img) = &app.capture_preview.last_image {
                        if let Err(e) = save_capture_image(img) {
                            let mut st = app.state.lock();
                            st.status_message = format!("Failed to save capture: {}", e);
                        } else {
                            let mut st = app.state.lock();
                            st.status_message = "Saved capture preview to disk".to_string();
                        }
                    }
                }
            });
        });
    }

    });

    theme::add_space_medium(ui);

    // ========================================
    // GROUP 2: Detection Settings
    // ========================================
    theme::card_frame().show(ui, |ui| {
        theme::styled_heading(ui, "🔍 Detection Settings");
        theme::add_space_small(ui);

    {
        let mut state = app.state.lock();

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
    });

    theme::add_space_medium(ui);

    // ========================================
    // GROUP 3: Audio Settings
    // ========================================
    theme::card_frame().show(ui, |ui| {
        theme::styled_heading(ui, "🔊 Audio Settings");
        theme::add_space_small(ui);

    // Volume Controls
    ui.label("Volume:");
    ui.horizontal(|ui| {
        ui.label("  🎵 Music:");
        let mut state = app.state.lock();
        let mut music_vol_percent = (state.music_volume * 100.0) as i32;
        if ui.add(egui::Slider::new(&mut music_vol_percent, 0..=100).suffix("%")).changed() {
            state.music_volume = (music_vol_percent as f32) / 100.0;
            drop(state);
            app.save_config();
        }
    });

    ui.horizontal(|ui| {
        ui.label("  🔉 Ambiance:");
        let mut state = app.state.lock();
        let mut ambiance_vol_percent = (state.ambiance_volume * 100.0) as i32;
        if ui.add(egui::Slider::new(&mut ambiance_vol_percent, 0..=100).suffix("%")).changed() {
            state.ambiance_volume = (ambiance_vol_percent as f32) / 100.0;
            drop(state);
            app.save_config();
        }
    });

    ui.add_space(5.0);

    // Sound Length Controls
    ui.label("Playback Duration:");
    ui.horizontal(|ui| {
        ui.label("  🎵 Music:");
        let mut state = app.state.lock();
        let mut music_length_seconds = (state.music_length_ms as f32) / 1000.0;
        if ui.add(egui::Slider::new(&mut music_length_seconds, 0.0..=60.0).suffix(" seconds").step_by(1.0)).changed() {
            state.music_length_ms = (music_length_seconds * 1000.0) as u64;
            drop(state);
            app.save_config();
        }
    });

    ui.horizontal(|ui| {
        ui.label("  🔉 Ambiance:");
        let mut state = app.state.lock();
        let mut ambiance_length_seconds = (state.ambiance_length_ms as f32) / 1000.0;
        if ui.add(egui::Slider::new(&mut ambiance_length_seconds, 0.0..=60.0).suffix(" seconds").step_by(1.0)).changed() {
            state.ambiance_length_ms = (ambiance_length_seconds * 1000.0) as u64;
            drop(state);
            app.save_config();
            }
        });
    });

    theme::add_space_medium(ui);

    // ========================================
    // GROUP 4: Updates
    // ========================================
    theme::card_frame().show(ui, |ui| {
        theme::styled_heading(ui, "🔄 Updates");
        theme::add_space_small(ui);

        ui.horizontal(|ui| {
            let mut state = app.state.lock();
            if ui.checkbox(&mut state.auto_check_updates, "Check for updates on startup").changed() {
                drop(state);
                app.save_config();
            }
        });

        theme::add_space_small(ui);

        if theme::styled_button(ui, "🔍 Check for Updates Now").clicked() {
            app.check_for_updates_manually();
        }
    });
}
