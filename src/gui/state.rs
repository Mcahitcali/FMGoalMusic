#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Dashboard,
    Library,
    TeamSelection,
    Detection,
    Settings,
    Help,
}

impl AppTab {
    pub const ALL: [AppTab; 6] = [
        AppTab::Dashboard,
        AppTab::Library,
        AppTab::TeamSelection,
        AppTab::Detection,
        AppTab::Settings,
        AppTab::Help,
    ];

    pub fn label(self) -> &'static str {
        match self {
            AppTab::Dashboard => "🏟️ Dashboard",
            AppTab::Library => "🎵 Library",
            AppTab::TeamSelection => "⚽ Team Selection",
            AppTab::Detection => "🛰 Detection",
            AppTab::Settings => "⚙️ Settings",
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
            AppTab::Help => "ℹ️",
        }
    }
}
