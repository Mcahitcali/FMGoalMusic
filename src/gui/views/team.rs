/// Team Selection view - Team picker
///
/// Allows users to select a specific team for goal detection.

use eframe::egui;

use crate::gui::theme;

use super::super::FMGoalMusicsApp;

/// Render the team selection tab
pub fn render_team_selection(app: &mut FMGoalMusicsApp, ui: &mut egui::Ui, _ctx: &egui::Context) {
    theme::card_frame().show(ui, |ui| {
        theme::styled_heading(ui, "⚽ Team Selection");
        theme::add_space_small(ui);

    if let Some(ref db) = app.team_database {
        ui.label("Select your team to play sound only for their goals:");

        // League dropdown
        let leagues = db.get_leagues();
        let mut league_changed = false;

        ui.horizontal(|ui| {
            ui.label("League:");
            egui::ComboBox::from_label("")
                .selected_text(app.selected_league.as_deref().unwrap_or("-- Select League --"))
                .show_ui(ui, |ui| {
                    if ui.selectable_label(app.selected_league.is_none(), "-- Select League --").clicked() {
                        app.selected_league = None;
                        app.selected_team_key = None;
                        league_changed = true;
                    }
                    for league in &leagues {
                        if ui.selectable_label(app.selected_league.as_ref() == Some(league), league).clicked() {
                            app.selected_league = Some(league.clone());
                            app.selected_team_key = None;
                            league_changed = true;
                        }
                    }
                });
        });

        // Team dropdown (only if league is selected)
        if let Some(ref league) = app.selected_league {
            if let Some(teams) = db.get_teams(league) {
                ui.horizontal(|ui| {
                    ui.label("Team:");
                    egui::ComboBox::from_label(" ")
                        .selected_text(
                            app.selected_team_key.as_ref()
                                .and_then(|key| teams.iter().find(|(k, _)| k == key))
                                .map(|(_, team)| team.display_name.as_str())
                                .unwrap_or("-- Select Team --")
                        )
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(app.selected_team_key.is_none(), "-- Select Team --").clicked() {
                                app.selected_team_key = None;
                                league_changed = true;
                            }
                            for (key, team) in &teams {
                                if ui.selectable_label(app.selected_team_key.as_ref() == Some(key), &team.display_name).clicked() {
                                    app.selected_team_key = Some(key.clone());
                                    league_changed = true;
                                }
                            }
                        });
                });
            }
        }

        // Update state and save if team selection changed
        if league_changed {
            let mut state = app.state.lock();

            if let (Some(ref league), Some(ref team_key)) = (&app.selected_league, &app.selected_team_key) {
                if let Some(team) = db.find_team(league, team_key) {
                    state.selected_team = Some(crate::config::SelectedTeam {
                        league: league.clone(),
                        team_key: team_key.clone(),
                        display_name: team.display_name.clone(),
                    });
                }
            } else {
                state.selected_team = None;
            }

            drop(state);
            app.save_config();
        }

        // Display current selection
        {
            let state = app.state.lock();
            if let Some(ref team) = state.selected_team {
                ui.label(format!("✓ Selected: {} ({})", team.display_name, team.league));
            } else {
                ui.label("ℹ No team selected - will play for all goals");
            }
        }

            if theme::styled_danger_button(ui, "🗑️ Clear Selection").clicked() {
                app.selected_league = None;
                app.selected_team_key = None;
                let mut state = app.state.lock();
                state.selected_team = None;
                drop(state);
                app.save_config();
            }

            theme::add_space_medium(ui);
            theme::styled_separator(ui);

            // Add New Team section
            app.render_add_team_ui(ui);
        } else {
            theme::error_label(ui, "⚠ Team database not available");
        }
    });
}
