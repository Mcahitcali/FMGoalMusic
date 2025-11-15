/// I18n phrase loader from embedded JSON files
///
/// Loads detection phrases from embedded JSON assets.
use super::i18n::{I18nPhrases, Language};
use serde::Deserialize;

/// I18n JSON structure
#[derive(Debug, Deserialize)]
struct I18nJson {
    language: String,
    code: String,
    detection: DetectionPhrases,
}

/// Detection phrases in JSON
#[derive(Debug, Deserialize)]
struct DetectionPhrases {
    goal_phrases: Vec<String>,
    kickoff_phrases: Vec<String>,
    match_end_phrases: Vec<String>,
}

/// Embedded language files
const EN_JSON: &str = include_str!("../../assets/i18n/en.json");
const TR_JSON: &str = include_str!("../../assets/i18n/tr.json");
const ES_JSON: &str = include_str!("../../assets/i18n/es.json");
const FR_JSON: &str = include_str!("../../assets/i18n/fr.json");
const DE_JSON: &str = include_str!("../../assets/i18n/de.json");
const IT_JSON: &str = include_str!("../../assets/i18n/it.json");
const PT_JSON: &str = include_str!("../../assets/i18n/pt.json");

/// Load I18n phrases from embedded JSON
pub fn load_phrases(language: Language) -> Result<I18nPhrases, Box<dyn std::error::Error>> {
    let json_str = match language {
        Language::English => EN_JSON,
        Language::Turkish => TR_JSON,
        Language::Spanish => ES_JSON,
        Language::French => FR_JSON,
        Language::German => DE_JSON,
        Language::Italian => IT_JSON,
        Language::Portuguese => PT_JSON,
    };

    let i18n_json: I18nJson = serde_json::from_str(json_str)?;

    Ok(I18nPhrases {
        language,
        goal_phrases: i18n_json.detection.goal_phrases,
        kickoff_phrases: i18n_json.detection.kickoff_phrases,
        match_end_phrases: i18n_json.detection.match_end_phrases,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_english() {
        let phrases = load_phrases(Language::English).unwrap();
        assert_eq!(phrases.language, Language::English);
        assert!(!phrases.goal_phrases.is_empty());
        assert!(!phrases.kickoff_phrases.is_empty());
        assert!(!phrases.match_end_phrases.is_empty());
    }

    #[test]
    fn test_load_turkish() {
        let phrases = load_phrases(Language::Turkish).unwrap();
        assert_eq!(phrases.language, Language::Turkish);
        assert!(phrases.goal_phrases.contains(&"GOL".to_string()));
    }

    #[test]
    fn test_load_all_languages() {
        let languages = vec![
            Language::English,
            Language::Turkish,
            Language::Spanish,
            Language::French,
            Language::German,
            Language::Italian,
            Language::Portuguese,
        ];

        for lang in languages {
            let result = load_phrases(lang);
            assert!(result.is_ok(), "Failed to load {:?}", lang);

            let phrases = result.unwrap();
            assert!(!phrases.goal_phrases.is_empty());
            assert!(!phrases.kickoff_phrases.is_empty());
            assert!(!phrases.match_end_phrases.is_empty());
        }
    }

    #[test]
    fn test_embedded_json_valid() {
        // Verify all embedded JSON is valid
        let jsons = vec![
            ("en", EN_JSON),
            ("tr", TR_JSON),
            ("es", ES_JSON),
            ("fr", FR_JSON),
            ("de", DE_JSON),
            ("it", IT_JSON),
            ("pt", PT_JSON),
        ];

        for (code, json) in jsons {
            let result: Result<I18nJson, _> = serde_json::from_str(json);
            assert!(
                result.is_ok(),
                "Invalid JSON for {}: {:?}",
                code,
                result.err()
            );
        }
    }
}
