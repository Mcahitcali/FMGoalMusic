use gpui::{px, rgb, App};
use gpui_component::theme::{self, Theme, ThemeColor, ThemeMode};

pub fn install(cx: &mut App) {
    theme::init(cx);

    // Start from gpui's default dark palette so every token has a sane value, then
    // override the hues we care about for the refreshed mid-tone “slate dusk” look.
    let mut colors = *ThemeColor::dark();
    // Core palette
    colors.background = rgb(0x211111).into();
    colors.foreground = rgb(0xf5f7fb).into();
    colors.primary = rgb(0xEA2831).into();
    colors.primary_hover = rgb(0xFF3B44).into();
    colors.primary_active = rgb(0xC51F27).into();
    colors.primary_foreground = rgb(0xffffff).into();
    // Accents and surfaces
    colors.accent = rgb(0xf48c4c).into();
    colors.accent_foreground = rgb(0x1c1309).into();
    colors.border = rgb(0x232323).into();
    // Cards / panels
    colors.group_box = rgb(0x1e1e1e).into();
    colors.group_box_foreground = colors.foreground;
    colors.muted = rgb(0x1a1a1a).into();
    colors.muted_foreground = rgb(0xb7bfcd).into();
    colors.list = rgb(0x1e1e1e).into();
    colors.list_even = rgb(0x242424).into();
    colors.list_hover = rgb(0x2a2a2a).into();
    colors.list_active = rgb(0x333333).into();
    colors.list_head = rgb(0x1f1f1f).into();
    colors.list_active_border = colors.primary;
    colors.slider_bar = rgb(0x2a2a2a).into();
    colors.slider_thumb = colors.primary;
    // Tabs
    colors.tab = rgb(0x1a1a1a).into();
    colors.tab_active = rgb(0x252525).into();
    colors.tab_active_foreground = colors.foreground;
    colors.tab_foreground = rgb(0xbfc7d4).into();
    colors.tab_bar = rgb(0x121212).into();
    // Selection and sidebar
    colors.selection = colors.primary;
    colors.sidebar = rgb(0x121212).into();
    colors.sidebar_foreground = colors.foreground;
    colors.sidebar_border = colors.border;
    colors.switch = rgb(0x2a2a2a).into();
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
