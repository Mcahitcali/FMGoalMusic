/// Internationalization support for detection phrases
///
/// Provides language-specific phrases for detecting game events.

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Turkish,
    Spanish,
    French,
    German,
    Italian,
    Portuguese,
}

impl Language {
    /// Get language name
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Turkish => "Türkçe",
            Language::Spanish => "Español",
            Language::French => "Français",
            Language::German => "Deutsch",
            Language::Italian => "Italiano",
            Language::Portuguese => "Português",
        }
    }

    /// Get language code (ISO 639-1)
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Turkish => "tr",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::German => "de",
            Language::Italian => "it",
            Language::Portuguese => "pt",
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Internationalization phrases for detection
#[derive(Debug, Clone)]
pub struct I18nPhrases {
    /// Language
    pub language: Language,
    /// Goal phrases (e.g., "GOAL!", "GOL!", "BUT!")
    pub goal_phrases: Vec<String>,
    /// Kickoff phrases (e.g., "Kick Off", "Saque Inicial")
    pub kickoff_phrases: Vec<String>,
    /// Match end phrases (e.g., "Full Time", "Final")
    pub match_end_phrases: Vec<String>,
}

impl I18nPhrases {
    /// Create phrases for a language from embedded JSON assets
    ///
    /// This is the preferred method as it loads from externalized JSON files.
    /// Falls back to hardcoded phrases if loading fails.
    pub fn new(language: Language) -> Self {
        // Try to load from embedded JSON assets
        if let Ok(phrases) = super::i18n_loader::load_phrases(language) {
            return phrases;
        }

        // Fallback to hardcoded phrases
        tracing::warn!("Failed to load i18n from JSON, using hardcoded fallback for {:?}", language);
        Self::new_hardcoded(language)
    }

    /// Create phrases for a language (hardcoded fallback)
    ///
    /// This is kept as a fallback in case JSON loading fails.
    pub fn new_hardcoded(language: Language) -> Self {
        let (goal_phrases, kickoff_phrases, match_end_phrases) = match language {
            Language::English => (
                vec!["GOAL!".to_string(), "Goal!".to_string()],
                vec!["Kick Off".to_string(), "Kick-Off".to_string()],
                vec!["Full Time".to_string(), "FT".to_string()],
            ),
            Language::Turkish => (
                vec!["GOL!".to_string(), "Gol!".to_string()],
                vec!["Başlangıç".to_string(), "Maç Başlangıcı".to_string()],
                vec!["Maç Sonu".to_string(), "MS".to_string()],
            ),
            Language::Spanish => (
                vec!["¡GOL!".to_string(), "¡Gol!".to_string()],
                vec!["Saque Inicial".to_string(), "Inicio".to_string()],
                vec!["Final".to_string(), "Tiempo Final".to_string()],
            ),
            Language::French => (
                vec!["BUT!".to_string(), "But!".to_string()],
                vec!["Coup d'envoi".to_string(), "Début".to_string()],
                vec!["Fin du Match".to_string(), "Temps plein".to_string()],
            ),
            Language::German => (
                vec!["TOR!".to_string(), "Tor!".to_string()],
                vec!["Anstoß".to_string(), "Spielbeginn".to_string()],
                vec!["Spielende".to_string(), "Abpfiff".to_string()],
            ),
            Language::Italian => (
                vec!["GOL!".to_string(), "Gol!".to_string(), "RETE!".to_string()],
                vec!["Calcio d'inizio".to_string(), "Inizio".to_string()],
                vec!["Fine Partita".to_string(), "Finito".to_string()],
            ),
            Language::Portuguese => (
                vec!["GOL!".to_string(), "Gol!".to_string(), "GOLO!".to_string()],
                vec!["Pontapé Inicial".to_string(), "Início".to_string()],
                vec!["Fim de Jogo".to_string(), "FJ".to_string()],
            ),
        };

        Self {
            language,
            goal_phrases,
            kickoff_phrases,
            match_end_phrases,
        }
    }

    /// Check if text contains any goal phrase
    pub fn contains_goal_phrase(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        self.goal_phrases
            .iter()
            .any(|phrase| text_lower.contains(&phrase.to_lowercase()))
    }

    /// Check if text contains any kickoff phrase
    pub fn contains_kickoff_phrase(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        self.kickoff_phrases
            .iter()
            .any(|phrase| text_lower.contains(&phrase.to_lowercase()))
    }

    /// Check if text contains any match end phrase
    pub fn contains_match_end_phrase(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        self.match_end_phrases
            .iter()
            .any(|phrase| text_lower.contains(&phrase.to_lowercase()))
    }
}

impl Default for I18nPhrases {
    fn default() -> Self {
        Self::new(Language::English)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_name() {
        assert_eq!(Language::English.name(), "English");
        assert_eq!(Language::Turkish.name(), "Türkçe");
    }

    #[test]
    fn test_language_code() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::Turkish.code(), "tr");
    }

    #[test]
    fn test_english_phrases() {
        let phrases = I18nPhrases::new(Language::English);
        assert!(phrases.contains_goal_phrase("GOAL!"));
        assert!(phrases.contains_kickoff_phrase("Kick Off"));
        assert!(phrases.contains_match_end_phrase("Full Time"));
    }

    #[test]
    fn test_turkish_phrases() {
        let phrases = I18nPhrases::new(Language::Turkish);
        assert!(phrases.contains_goal_phrase("GOL!"));
        assert!(phrases.contains_kickoff_phrase("Başlangıç"));
        assert!(phrases.contains_match_end_phrase("Maç Sonu"));
    }

    #[test]
    fn test_case_insensitive_matching() {
        let phrases = I18nPhrases::new(Language::English);
        assert!(phrases.contains_goal_phrase("goal!"));
        assert!(phrases.contains_goal_phrase("GOAL!"));
        assert!(phrases.contains_goal_phrase("Goal!"));
    }

    #[test]
    fn test_no_match() {
        let phrases = I18nPhrases::new(Language::English);
        assert!(!phrases.contains_goal_phrase("Random text"));
        assert!(!phrases.contains_kickoff_phrase("Random text"));
        assert!(!phrases.contains_match_end_phrase("Random text"));
    }
}
