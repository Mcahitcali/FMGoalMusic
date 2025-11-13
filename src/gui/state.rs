#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Dashboard,
    Library,
    TeamSelection,
    Detection,
    Settings,
    Shortcuts,
    Help,
}

impl AppTab {
    pub const ALL: [AppTab; 7] = [
        AppTab::Dashboard,
        AppTab::Library,
        AppTab::TeamSelection,
        AppTab::Detection,
        AppTab::Settings,
        AppTab::Shortcuts,
        AppTab::Help,
    ];

    pub fn label(self) -> &'static str {
        match self {
            AppTab::Dashboard => "🏟️ Dashboard",
            AppTab::Library => "🎵 Library",
            AppTab::TeamSelection => "⚽ Team Selection",
            AppTab::Detection => "🛰 Detection",
            AppTab::Settings => "⚙️ Settings",
            AppTab::Shortcuts => "⌨️ Shortcuts",
            AppTab::Help => "ℹ️ Help",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            AppTab::Dashboard => "Dashboard",
            AppTab::Library => "Library",
            AppTab::TeamSelection => "Team Selection",
            AppTab::Detection => "Detection",
            AppTab::Settings => "Settings",
            AppTab::Shortcuts => "Keyboard Shortcuts",
            AppTab::Help => "Help",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            AppTab::Dashboard => "🏟️",
            AppTab::Library => "🎵",
            AppTab::TeamSelection => "⚽",
            AppTab::Detection => "🛰",
            AppTab::Settings => "⚙️",
            AppTab::Shortcuts => "⌨️",
            AppTab::Help => "ℹ️",
        }
    }
}
