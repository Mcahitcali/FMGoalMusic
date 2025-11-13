use gpui::{px, rgb, App};
use gpui_component::theme::{self, Theme, ThemeColor, ThemeMode};

pub fn install(cx: &mut App) {
    theme::init(cx);

    // Start from gpui's default dark palette so every token has a sane value, then
    // override the hues we care about for the refreshed mid-tone “slate dusk” look.
    let mut colors = *ThemeColor::dark();
    // Core palette
    colors.background = rgb(0x0f172a).into();
    colors.foreground = rgb(0xe2e8f0).into();
    colors.primary = rgb(0x2563eb).into();
    colors.primary_hover = rgb(0x1d4ed8).into();
    colors.primary_active = rgb(0x1e40af).into();
    colors.primary_foreground = rgb(0xffffff).into();
    // Accents and surfaces
    colors.accent = rgb(0x38bdf8).into();
    colors.accent_foreground = rgb(0x082f49).into();
    colors.border = rgb(0x1e293b).into();
    // Cards / panels
    colors.group_box = rgb(0x1e293b).into();
    colors.group_box_foreground = colors.foreground;
    colors.muted = rgb(0x111c2e).into();
    colors.muted_foreground = rgb(0x9ca3af).into();
    colors.list = rgb(0x1b2435).into();
    colors.list_even = rgb(0x1f2937).into();
    colors.list_hover = rgb(0x273449).into();
    colors.list_active = rgb(0x2f3a51).into();
    colors.list_head = rgb(0x1d2535).into();
    colors.list_active_border = colors.primary;
    colors.slider_bar = rgb(0x233044).into();
    colors.slider_thumb = colors.primary;
    // Tabs
    colors.tab = rgb(0x111827).into();
    colors.tab_active = rgb(0x1f2937).into();
    colors.tab_active_foreground = colors.foreground;
    colors.tab_foreground = rgb(0x9ca3af).into();
    colors.tab_bar = rgb(0x0b1220).into();
    // Selection and sidebar
    colors.selection = colors.primary;
    colors.sidebar = rgb(0x0b1220).into();
    colors.sidebar_foreground = colors.foreground;
    colors.sidebar_border = colors.border;
    colors.switch = rgb(0x1f2937).into();
    // Status tokens
    colors.warning = rgb(0xf6c343).into();
    colors.warning_foreground = rgb(0x281d08).into();
    colors.danger = rgb(0xf05d70).into();
    colors.danger_foreground = rgb(0x300006).into();
    colors.success = rgb(0x39ff14).into();
    colors.success_foreground = rgb(0x0a0a0a).into();
    colors.info = rgb(0x4c9ef4).into();
    colors.info_foreground = rgb(0x041321).into();

    let mut theme = Theme::from(&colors);
    theme.mode = ThemeMode::Dark;
    theme.font_size = px(15.0);

    if cx.has_global::<Theme>() {
        *Theme::global_mut(cx) = theme;
    } else {
        cx.set_global(theme);
    }
}
