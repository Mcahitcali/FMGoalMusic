/// Help view - Documentation and troubleshooting
///
/// Displays user documentation for all features.
use eframe::egui;

use crate::legacy_gui::theme;

/// Render the help tab
pub fn render_help(ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        theme::card_frame().show(ui, |ui| {
            theme::styled_heading(ui, "📖 How to Use FM Goal Musics");
            theme::add_space_medium(ui);

            // Quick Start - Most important section, shown first and open by default
            egui::CollapsingHeader::new("🏁 Quick Start")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label("1. Add at least one music file in Library tab");
                    ui.label("2. (Optional) Select your team in Team Selection tab");
                    ui.label("3. Configure capture region in Settings tab");
                    ui.label("4. Click '▶️ Start Detection' button (top-left)");
                    ui.label("5. Play Football Manager and watch for goals!");
                    ui.label("");
                    ui.label("✅ Status bar shows detection state (Green = Running)");
                    ui.label("✅ Detection count increments with each goal detected");
                });

            ui.collapsing("🎵 Library Tab", |ui| {
                ui.label("• Click '➕ Add Music File' to add celebration music (MP3, WAV, OGG)");
                ui.label("• Select a music file from the list");
                ui.label("• Click '▶️ Preview' to test the selected music");
                ui.label("• Click '🗑️ Remove Selected' to remove unwanted music");
                ui.label("");
                ui.label("Ambiance Sounds:");
                ui.label("• Enable 'Ambiance' checkbox for crowd cheer effects");
                ui.label("• Click '➕ Add Goal Cheer Sound' to add a WAV crowd sound");
                ui.label("• This plays alongside your music for extra atmosphere");
            });

            ui.collapsing("⚽ Team Selection Tab", |ui| {
                ui.label("• Select a League from the dropdown");
                ui.label("• Select your Team from the filtered list");
                ui.label("• Goal music will only play for your selected team's goals");
                ui.label("• Leave unselected to play for all goals");
                ui.label("");
                ui.label("Add New Team:");
                ui.label("• Use '➕ Add New Team' section to add custom teams");
                ui.label("• Can use existing league or create a new one");
                ui.label("• Add variations to improve OCR detection accuracy");
            });

            ui.collapsing("⚙️ Settings Tab", |ui| {
                ui.label("Display & Capture:");
                ui.label("• Select which monitor to capture from (multi-monitor support)");
                ui.label("• Capture Region: X, Y, Width, Height of screen area to monitor");
                ui.label("• Click '🎯 Select Region Visually' to drag-select on screen");
                ui.label("• Capture Preview: Shows real-time capture of the screen region");
                ui.label("• Use 'Save frame...' to export the current captured frame");
                ui.label("");
                ui.label("Detection Settings:");
                ui.label("• OCR Threshold: 0 = auto (recommended), 1-255 = manual");
                ui.label("• Debounce: Cooldown between detections (default 8000ms)");
                ui.label("• Morphological Opening: Reduces noise (adds 5-10ms latency)");
                ui.label("");
                ui.label("Audio Settings:");
                ui.label("• Music Volume: 0-100% playback volume for celebration music");
                ui.label("• Ambiance Volume: 0-100% playback volume for crowd cheer");
                ui.label("• Music Length: How long to play music (0 = full track)");
                ui.label("• Ambiance Length: How long to play crowd sound (0 = full)");
                ui.label("");
                ui.label("Updates:");
                ui.label("• Enable automatic update checking on startup");
                ui.label("• Manually check for updates anytime");
            });

            ui.collapsing("🔧 Configuring teams.json", |ui| {
                ui.label("The teams database is automatically created on first run at:");
                ui.label("  macOS: ~/Library/Application Support/FMGoalMusic/teams.json");
                ui.label("  Windows: %APPDATA%/FMGoalMusic/teams.json");
                ui.label("  Linux: ~/.config/FMGoalMusic/teams.json");
                ui.label("");
                ui.label("You can safely edit this file to add your favorite teams!");
                ui.label("");
                ui.label("Structure:");
                ui.label("  {");
                ui.label("    \"Premier League\": {");
                ui.label("      \"manchester_united\": {");
                ui.label("        \"display_name\": \"Manchester Utd\",");
                ui.label("        \"variations\": [\"Man Utd\", \"Man United\", \"MUFC\"]");
                ui.label("      }");
                ui.label("    }");
                ui.label("  }");
                ui.label("");
                ui.label("• Add your leagues and teams with variations");
                ui.label("• Variations help match different OCR results");
                ui.label("• Restart the app after editing teams.json");
            });

            ui.collapsing("❓ Troubleshooting", |ui| {
                ui.label("Music not playing:");
                ui.label("• Check music file is selected in Library tab");
                ui.label("• Verify 'Start Detection' is active (button shows 'Stop')");
                ui.label("• Confirm capture region covers goal text area");
                ui.label("");
                ui.label("No goals detected:");
                ui.label("• Check 'Capture Preview' in Settings tab to verify region");
                ui.label("• Ensure capture region covers the goal notification text");
                ui.label("• Try OCR Threshold = 0 (auto) first");
                ui.label("• Increase debounce if detecting multiple times per goal");
                ui.label("");
                ui.label("Team selection not working:");
                ui.label("• Verify team exists in teams.json with correct variations");
                ui.label("• Check Team Selection tab shows '✓ Selected: [team]'");
                ui.label("• Ensure OCR is reading team name correctly via Capture Preview");
            });
        });
    });
}
