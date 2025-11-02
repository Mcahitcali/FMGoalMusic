/// Text extraction and parsing utilities for goal detection
///
/// This module handles parsing OCR results to extract goal-related information,
/// specifically team names from "GOAL FOR {team}" or "GOL {team}" patterns.

/// Check if text contains goal-related keywords
///
/// Accepts either:
/// - "GOAL FOR {team}" (English)
/// - "GOL {team}" (abbreviated/international)
///
/// # Arguments
/// * `text` - The OCR-extracted text to check
///
/// # Returns
/// `true` if goal text is detected, `false` otherwise
pub fn contains_goal_text(text: &str) -> bool {
    let normalized = text.trim().to_uppercase();
    normalized.contains("GOAL FOR") || normalized.contains("GOL ")
}

/// Extract team name from goal text
///
/// Parses text like "GOAL FOR Manchester United" or "GOL Barcelona"
/// and returns the team name.
///
/// # Arguments
/// * `text` - The OCR-extracted text containing goal information
///
/// # Returns
/// `Some(team_name)` if a team name was found, `None` otherwise
///
/// # Examples
/// ```
/// # use fm_goal_musics::ocr::text_extraction::extract_team_name;
/// assert_eq!(extract_team_name("GOAL FOR Arsenal"), Some("ARSENAL".to_string()));
/// assert_eq!(extract_team_name("GOL Barcelona"), Some("BARCELONA".to_string()));
/// assert_eq!(extract_team_name("Random text"), None);
/// ```
pub fn extract_team_name(text: &str) -> Option<String> {
    let normalized = text.trim().to_uppercase();

    // Try "GOAL FOR {team}" pattern first
    if let Some(pos) = normalized.find("GOAL FOR") {
        let after = normalized[pos + 8..].trim();
        if !after.is_empty() {
            return Some(after.to_string());
        }
    }

    // Try "GOL {team}" pattern
    if let Some(pos) = normalized.find("GOL ") {
        let after = normalized[pos + 4..].trim();
        if !after.is_empty() {
            return Some(after.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_goal_text() {
        // Positive cases
        assert!(contains_goal_text("GOAL FOR Arsenal"));
        assert!(contains_goal_text("goal for arsenal")); // Case insensitive
        assert!(contains_goal_text("GOL Barcelona"));
        assert!(contains_goal_text("Some noise GOAL FOR TeamName more noise"));

        // Negative cases
        assert!(!contains_goal_text(""));
        assert!(!contains_goal_text("Random text"));
        assert!(!contains_goal_text("GOALKEEPER")); // Shouldn't match
    }

    #[test]
    fn test_extract_team_name_goal_for() {
        assert_eq!(
            extract_team_name("GOAL FOR Arsenal"),
            Some("ARSENAL".to_string())
        );
        assert_eq!(
            extract_team_name("  GOAL FOR   Manchester United  "),
            Some("MANCHESTER UNITED".to_string())
        );
        assert_eq!(
            extract_team_name("goal for barcelona"), // Case insensitive
            Some("BARCELONA".to_string())
        );
    }

    #[test]
    fn test_extract_team_name_gol() {
        assert_eq!(
            extract_team_name("GOL Barcelona"),
            Some("BARCELONA".to_string())
        );
        assert_eq!(
            extract_team_name("GOL Real Madrid"),
            Some("REAL MADRID".to_string())
        );
    }

    #[test]
    fn test_extract_team_name_no_match() {
        assert_eq!(extract_team_name("Random text"), None);
        assert_eq!(extract_team_name(""), None);
        assert_eq!(extract_team_name("GOAL FOR"), None); // No team name
        assert_eq!(extract_team_name("GOL"), None); // No team name
    }

    #[test]
    fn test_extract_team_name_with_noise() {
        // Should extract team even with surrounding noise
        assert_eq!(
            extract_team_name("Some text GOAL FOR Arsenal more text"),
            Some("ARSENAL MORE TEXT".to_string())
        );
    }
}
