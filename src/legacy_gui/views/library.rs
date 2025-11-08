/// Library view - Music files and ambiance sounds
///
/// Displays music library management and preview functionality.
use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;

use crate::audio::AudioManager;
use crate::legacy_gui::state::PreviewAudio;
use crate::legacy_gui::theme;

use super::super::FMGoalMusicsApp;

/// Render the library tab
pub fn render_library(app: &mut FMGoalMusicsApp, ui: &mut egui::Ui) {
    // ========================================
    // Music Files Section
    // ========================================
    theme::card_frame().show(ui, |ui| {
        theme::styled_heading(ui, "🎵 Music Files");
        theme::add_space_small(ui);

        ui.horizontal(|ui| {
            if theme::styled_button(ui, "➕ Add Music File").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Audio", &["mp3", "wav", "ogg"])
                    .pick_file()
                {
                    app.add_music_file(path);
                }
            }

            // Preview button (moved before Remove for better UX flow: Add → Preview → Remove)
            let preview_active = app.preview_playing;
            let preview_label = if preview_active {
                "🔇 Stop Preview"
            } else {
                "▶️ Preview"
            };

            if theme::styled_button(ui, preview_label).clicked() {
                if preview_active {
                    app.stop_preview();
                } else {
                    let selected_path = {
                        let state = app.state.lock();
                        state
                            .selected_music_index
                            .and_then(|idx| state.music_list.get(idx))
                            .map(|entry| entry.path.clone())
                    };

                    match selected_path {
                        Some(path) => {
                            let needs_reload = app
                                .preview_audio
                                .as_ref()
                                .map_or(true, |p| p.path.as_path() != path.as_path());

                            let audio_data = match app.get_or_load_audio_data(&path) {
                                Ok(data) => data,
                                Err(err) => {
                                    let mut st = app.state.lock();
                                    st.status_message = err;
                                    return;
                                }
                            };

                            if needs_reload {
                                app.stop_preview();
                                match AudioManager::from_preloaded(Arc::clone(&audio_data)) {
                                    Ok(manager) => {
                                        app.preview_audio = Some(PreviewAudio {
                                            manager,
                                            path: path.clone(),
                                        });
                                    }
                                    Err(e) => {
                                        let mut st = app.state.lock();
                                        st.status_message = format!("Preview init failed: {}", e);
                                        return;
                                    }
                                }
                            }

                            if let Some(preview) = app.preview_audio.as_ref() {
                                preview.manager.stop();
                                match preview.manager.play_sound() {
                                    Ok(()) => {
                                        app.preview_playing = true;
                                        let mut st = app.state.lock();
                                        st.status_message = "Preview playing...".to_string();
                                    }
                                    Err(e) => {
                                        let mut st = app.state.lock();
                                        st.status_message = format!("Preview failed: {}", e);
                                    }
                                }
                            }
                        }
                        None => {
                            let mut st = app.state.lock();
                            st.status_message = "Select a music file to preview.".to_string();
                        }
                    }
                }
            }

            if theme::styled_danger_button(ui, "🗑️ Remove Selected").clicked() {
                let mut state = app.state.lock();
                if let Some(idx) = state.selected_music_index {
                    state.music_list.remove(idx);
                    state.selected_music_index = None;
                }
                drop(state);
                app.stop_preview();
                app.save_config();
            }
        });

        theme::styled_separator(ui);

        // Music list display
        let selection_changed = egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                let mut state = app.state.lock();
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
            })
            .inner;

        if selection_changed {
            app.save_config();
        }
    });

    theme::add_space_medium(ui);

    // ========================================
    // Ambiance Sounds Section
    // ========================================
    theme::card_frame().show(ui, |ui| {
        theme::styled_heading(ui, "🎺 Ambiance Sounds");
        theme::add_space_small(ui);

        ui.horizontal(|ui| {
            let mut state = app.state.lock();
            if ui
                .checkbox(&mut state.ambiance_enabled, "Enable Ambiance")
                .changed()
            {
                drop(state);
                app.save_config();
            }
        });

        theme::add_space_small(ui);

        ui.horizontal(|ui| {
            if theme::styled_button(ui, "➕ Add Goal Cheer Sound").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Audio", &["wav"])
                    .pick_file()
                {
                    let mut state = app.state.lock();
                    state.goal_ambiance_path = Some(path.to_string_lossy().to_string());
                    drop(state);
                    app.save_config();
                }
            }

            // Preview ambiance button
            let ambiance_preview_active = app.ambiance_preview_playing;
            let ambiance_preview_label = if ambiance_preview_active {
                "🔇 Stop Preview"
            } else {
                "▶️ Preview"
            };

            if theme::styled_button(ui, ambiance_preview_label).clicked() {
                if ambiance_preview_active {
                    app.stop_ambiance_preview();
                } else {
                    let ambiance_path = {
                        let state = app.state.lock();
                        state.goal_ambiance_path.clone()
                    };

                    match ambiance_path {
                        Some(path) => {
                            let path_buf = PathBuf::from(&path);
                            let audio_data = match app.get_or_load_audio_data(&path_buf) {
                                Ok(data) => data,
                                Err(err) => {
                                    let mut st = app.state.lock();
                                    st.status_message = err;
                                    return;
                                }
                            };

                            let needs_reload = app
                                .ambiance_preview_audio
                                .as_ref()
                                .map_or(true, |p| p.path.as_path() != path_buf.as_path());

                            if needs_reload {
                                app.stop_ambiance_preview();
                                match AudioManager::from_preloaded(Arc::clone(&audio_data)) {
                                    Ok(manager) => {
                                        app.ambiance_preview_audio = Some(PreviewAudio {
                                            manager,
                                            path: path_buf.clone(),
                                        });
                                    }
                                    Err(e) => {
                                        let mut st = app.state.lock();
                                        st.status_message =
                                            format!("Ambiance preview init failed: {}", e);
                                        return;
                                    }
                                }
                            }

                            if let Some(preview) = app.ambiance_preview_audio.as_ref() {
                                preview.manager.stop();
                                match preview.manager.play_sound() {
                                    Ok(()) => {
                                        app.ambiance_preview_playing = true;
                                        let mut st = app.state.lock();
                                        st.status_message =
                                            "Ambiance preview playing...".to_string();
                                    }
                                    Err(e) => {
                                        let mut st = app.state.lock();
                                        st.status_message =
                                            format!("Ambiance preview failed: {}", e);
                                    }
                                }
                            }
                        }
                        None => {
                            let mut st = app.state.lock();
                            st.status_message =
                                "Add a goal cheer sound first to preview.".to_string();
                        }
                    }
                }
            }

            if theme::styled_danger_button(ui, "🗑️ Remove Cheer Sound").clicked() {
                let mut state = app.state.lock();
                state.goal_ambiance_path = None;
                drop(state);
                app.stop_ambiance_preview();
                app.save_config();
            }
        });

        theme::add_space_small(ui);

        {
            let state = app.state.lock();
            if let Some(ref path) = state.goal_ambiance_path {
                let display_name = PathBuf::from(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.clone());
                theme::success_label(ui, &format!("✓ Crowd cheer: {}", display_name));
            } else {
                theme::info_label(ui, "ℹ No crowd cheer sound selected");
            }
        }
    });
}
