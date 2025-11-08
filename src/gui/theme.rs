use gpui::{px, rgb, App};
use gpui_component::theme::{self, Theme, ThemeColor, ThemeMode};

pub fn install(cx: &mut App) {
    theme::init(cx);

    // Start from gpui's default dark palette so every token has a sane value, then
    // override the hues we care about for the refreshed mid-tone “slate dusk” look.
    let mut colors = *ThemeColor::dark();
    colors.background = rgb(0x262c34).into(); // slate navy
    colors.foreground = rgb(0xf5f7fb).into();
    colors.primary = rgb(0x4c7cf4).into();
    colors.primary_hover = rgb(0x5f8dff).into();
    colors.primary_active = rgb(0x365ed1).into();
    colors.primary_foreground = rgb(0xffffff).into();
    colors.accent = rgb(0xf48c4c).into();
    colors.accent_foreground = rgb(0x1c1309).into();
    colors.border = rgb(0x3d4452).into();
    colors.group_box = rgb(0x2f3540).into();
    colors.group_box_foreground = colors.foreground;
    colors.muted = rgb(0x2c323b).into();
    colors.muted_foreground = rgb(0xb7bfcd).into();
    colors.list = rgb(0x2f3540).into();
    colors.list_even = rgb(0x333a46).into();
    colors.list_hover = rgb(0x394252).into();
    colors.list_active = rgb(0x3f4960).into();
    colors.list_head = rgb(0x343c48).into();
    colors.list_active_border = rgb(0x5d7fe2).into();
    colors.slider_bar = rgb(0x3c4454).into();
    colors.slider_thumb = colors.primary;
    colors.tab = rgb(0x2a303a).into();
    colors.tab_active = rgb(0x3a4250).into();
    colors.tab_active_foreground = colors.foreground;
    colors.tab_foreground = rgb(0xbfc7d4).into();
    colors.tab_bar = rgb(0x262b34).into();
    colors.selection = rgb(0x5a82ff).into();
    colors.sidebar = rgb(0x222730).into();
    colors.sidebar_foreground = colors.foreground;
    colors.sidebar_border = colors.border;
    colors.switch = rgb(0x3b4352).into();
    colors.warning = rgb(0xf6c343).into();
    colors.warning_foreground = rgb(0x281d08).into();
    colors.danger = rgb(0xf05d70).into();
    colors.danger_foreground = rgb(0x300006).into();
    colors.success = rgb(0x55b685).into();
    colors.success_foreground = rgb(0x062119).into();
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
