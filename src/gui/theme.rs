/// Football-themed design system
///
/// Provides colors, spacing, and component styling for the application.

use eframe::egui;

// ============================================================================
// COLOR PALETTE - Football/Stadium Theme
// ============================================================================

/// Primary Colors
pub const PITCH_GREEN: egui::Color32 = egui::Color32::from_rgb(27, 94, 32); // Dark forest green
pub const PITCH_LIGHT: egui::Color32 = egui::Color32::from_rgb(46, 125, 50); // Lighter green for hover
pub const GOAL_GOLD: egui::Color32 = egui::Color32::from_rgb(255, 215, 0); // Gold for highlights
pub const STADIUM_DARK: egui::Color32 = egui::Color32::from_rgb(26, 26, 29); // Deep charcoal
pub const WHITE_LINE: egui::Color32 = egui::Color32::from_rgb(255, 255, 255); // Pure white

/// Secondary Colors
pub const GRASS_ACCENT: egui::Color32 = egui::Color32::from_rgb(76, 175, 80); // Bright green
pub const SHADOW: egui::Color32 = egui::Color32::from_rgb(13, 13, 15); // Deep shadow
pub const SCOREBOARD_BG: egui::Color32 = egui::Color32::from_rgb(44, 44, 48); // Medium gray

/// Semantic Colors
pub const SUCCESS: egui::Color32 = GRASS_ACCENT;
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(255, 167, 38); // Orange
pub const ERROR: egui::Color32 = egui::Color32::from_rgb(239, 83, 80); // Red
pub const INFO: egui::Color32 = egui::Color32::from_rgb(66, 165, 245); // Blue

// ============================================================================
// SPACING SYSTEM
// ============================================================================

pub const SPACING_XS: f32 = 4.0;
pub const SPACING_SM: f32 = 8.0;
pub const SPACING_MD: f32 = 12.0;
pub const SPACING_LG: f32 = 16.0;
pub const SPACING_XL: f32 = 24.0;

// ============================================================================
// COMPONENT SIZES
// ============================================================================

pub const BUTTON_ROUNDING: f32 = 6.0;
pub const CARD_ROUNDING: f32 = 8.0;
pub const DOT_RADIUS: f32 = 6.0;

// ============================================================================
// THEME APPLICATION
// ============================================================================

/// Apply the football theme to the egui context
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Window styling
    style.visuals.window_fill = STADIUM_DARK;
    style.visuals.window_stroke = egui::Stroke::new(1.0, PITCH_GREEN);

    // Panel styling
    style.visuals.panel_fill = egui::Color32::from_rgb(30, 30, 33);

    // Widget styling
    style.visuals.widgets.noninteractive.bg_fill = SCOREBOARD_BG;
    style.visuals.widgets.noninteractive.weak_bg_fill = SCOREBOARD_BG;
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, WHITE_LINE);

    style.visuals.widgets.inactive.bg_fill = PITCH_GREEN;
    style.visuals.widgets.inactive.weak_bg_fill = PITCH_GREEN;
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, WHITE_LINE);

    style.visuals.widgets.hovered.bg_fill = PITCH_LIGHT;
    style.visuals.widgets.hovered.weak_bg_fill = PITCH_LIGHT;
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, WHITE_LINE);
    style.visuals.widgets.hovered.expansion = 1.0;

    style.visuals.widgets.active.bg_fill = GRASS_ACCENT;
    style.visuals.widgets.active.weak_bg_fill = GRASS_ACCENT;
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, WHITE_LINE);

    // Selection colors
    style.visuals.selection.bg_fill = PITCH_GREEN;
    style.visuals.selection.stroke = egui::Stroke::new(1.0, GOAL_GOLD);

    // Hyperlink color
    style.visuals.hyperlink_color = INFO;

    // Text colors
    style.visuals.override_text_color = Some(WHITE_LINE);

    // Spacing
    style.spacing.button_padding = egui::vec2(SPACING_MD, SPACING_SM);
    style.spacing.item_spacing = egui::vec2(SPACING_SM, SPACING_SM);

    ctx.set_style(style);
}

// ============================================================================
// COMPONENT STYLING FUNCTIONS
// ============================================================================

/// Create a styled button with football theme
pub fn styled_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(text)
        .fill(PITCH_GREEN)
        .stroke(egui::Stroke::new(1.0, WHITE_LINE))
        .rounding(BUTTON_ROUNDING);
    ui.add(button)
}

/// Create a primary action button with gold accent
pub fn styled_primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(egui::RichText::new(text).color(STADIUM_DARK).strong())
        .fill(GOAL_GOLD)
        .stroke(egui::Stroke::new(1.0, GOAL_GOLD))
        .rounding(BUTTON_ROUNDING);
    ui.add(button)
}

/// Create a danger/remove button with red color
pub fn styled_danger_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(text)
        .fill(ERROR)
        .stroke(egui::Stroke::new(1.0, WHITE_LINE))
        .rounding(BUTTON_ROUNDING);
    ui.add(button)
}

/// Create a styled heading with pitch green color
pub fn styled_heading(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .color(PITCH_GREEN)
            .heading()
            .strong()
    );
}

/// Create a styled subheading
pub fn styled_subheading(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .color(PITCH_LIGHT)
            .size(16.0)
            .strong()
    );
}

/// Create a card frame with shadow and rounded corners
pub fn card_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(SCOREBOARD_BG)
        .stroke(egui::Stroke::new(1.0, PITCH_GREEN))
        .rounding(CARD_ROUNDING)
        .inner_margin(SPACING_MD)
        .shadow(egui::epaint::Shadow {
            offset: [0, 2],
            blur: 8,
            spread: 0,
            color: SHADOW,
        })
}

/// Create a section frame (lighter than card)
pub fn section_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(35, 35, 38))
        .stroke(egui::Stroke::new(0.5, PITCH_GREEN))
        .rounding(BUTTON_ROUNDING)
        .inner_margin(SPACING_SM)
}

/// Render a status dot indicator
pub fn status_dot(ui: &mut egui::Ui, active: bool) {
    let color = if active { SUCCESS } else { egui::Color32::GRAY };
    let painter = ui.painter();
    let dot_pos = ui.cursor().left_top() + egui::vec2(DOT_RADIUS + 2.0, DOT_RADIUS + 2.0);
    painter.circle_filled(dot_pos, DOT_RADIUS, color);
    ui.add_space(DOT_RADIUS * 2.0 + 4.0);
}

/// Create a styled separator with pitch green color
pub fn styled_separator(ui: &mut egui::Ui) {
    ui.add_space(SPACING_SM);
    ui.add(egui::Separator::default().horizontal().spacing(SPACING_SM));
    ui.add_space(SPACING_SM);
}

/// Create a success message label
pub fn success_label(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(SUCCESS));
}

/// Create a warning message label
pub fn warning_label(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(WARNING));
}

/// Create an error message label
pub fn error_label(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(ERROR));
}

/// Create an info message label
pub fn info_label(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(INFO));
}

// ============================================================================
// LAYOUT HELPERS
// ============================================================================

/// Add vertical space based on spacing constants
pub fn add_space_small(ui: &mut egui::Ui) {
    ui.add_space(SPACING_SM);
}

pub fn add_space_medium(ui: &mut egui::Ui) {
    ui.add_space(SPACING_MD);
}

pub fn add_space_large(ui: &mut egui::Ui) {
    ui.add_space(SPACING_LG);
}

/// Create a two-column layout helper
pub fn two_column_row<R>(
    ui: &mut egui::Ui,
    left: impl FnOnce(&mut egui::Ui) -> R,
    right: impl FnOnce(&mut egui::Ui) -> R,
) -> (R, R) {
    let available_width = ui.available_width();
    let column_width = (available_width - SPACING_MD) / 2.0;

    ui.horizontal(|ui| {
        let left_result = ui.vertical(|ui| {
            ui.set_width(column_width);
            left(ui)
        }).inner;

        ui.add_space(SPACING_MD);

        let right_result = ui.vertical(|ui| {
            ui.set_width(column_width);
            right(ui)
        }).inner;

        (left_result, right_result)
    }).inner
}

// ============================================================================
// ANIMATION HELPERS
// ============================================================================

/// Get hover scale factor (for button animations)
pub fn hover_scale() -> f32 {
    1.02
}

/// Get active scale factor (for button press animations)
pub fn active_scale() -> f32 {
    0.98
}
