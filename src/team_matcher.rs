use crate::teams::Team;

/// Matcher for checking if a detected team name matches the selected team
pub struct TeamMatcher {
    normalized_variations: Vec<String>,
}

impl TeamMatcher {
    /// Create a new matcher for a specific team
    pub fn new(team: &Team) -> Self {
        let normalized_variations = team
            .variations
            .iter()
            .map(|v| Self::normalize(v))
            .collect();

        Self {
            normalized_variations,
        }
    }

    /// Check if the detected team name matches any variation of the selected team
    pub fn matches(&self, detected_name: &str) -> bool {
        let normalized_detected = Self::normalize(detected_name);

        // Fast path: exact equality with any normalized variation
        if self
            .normalized_variations
            .iter()
            .any(|variation| variation == &normalized_detected)
        {
            return true;
        }

        // Token-subset match:
        // Accept when all tokens from a variation are present in detected tokens.
        // This covers cases like detected: "fc internazionale milano" vs variation: "fc internazionale".
        let detected_tokens = Self::tokens(&normalized_detected);
        self.normalized_variations.iter().any(|variation| {
            let var_tokens = Self::tokens(variation);
            !var_tokens.is_empty() && var_tokens.is_subset(&detected_tokens)
        })
    }

    /// Normalize a team name for matching
    /// - Convert to lowercase
    /// - Remove special characters (keep only ASCII alphanumerics and spaces)
    /// - Normalize whitespace (trim and collapse multiple spaces)
    fn normalize(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Split a normalized string into lowercase ASCII tokens
    fn tokens(text: &str) -> std::collections::HashSet<&str> {
        text.split_whitespace().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_team() -> Team {
        Team {
            display_name: "Manchester Utd".to_string(),
            variations: vec![
                "Man United".to_string(),
                "Manchester Utd".to_string(),
                "Manchester United".to_string(),
                "Manchester United FC".to_string(),
            ],
        }
    }

    #[test]
    fn test_normalize() {
        assert_eq!(
            TeamMatcher::normalize("Manchester United"),
            "manchester united"
        );
        assert_eq!(TeamMatcher::normalize("Man. United"), "man united");
        assert_eq!(TeamMatcher::normalize("Man  United"), "man united");
        assert_eq!(TeamMatcher::normalize("  Man United  "), "man united");
    }

    #[test]
    fn test_normalize_special_chars() {
        assert_eq!(TeamMatcher::normalize("FC Barcelona"), "fc barcelona");
        assert_eq!(
            TeamMatcher::normalize("Atlético Madrid"),
            "atltico madrid"
        );
        assert_eq!(TeamMatcher::normalize("Man. City!"), "man city");
    }

    #[test]
    fn test_matches_exact() {
        let team = create_test_team();
        let matcher = TeamMatcher::new(&team);

        assert!(matcher.matches("Man United"));
        assert!(matcher.matches("Manchester United"));
        assert!(matcher.matches("Manchester Utd"));
    }

    #[test]
    fn test_matches_case_insensitive() {
        let team = create_test_team();
        let matcher = TeamMatcher::new(&team);

        assert!(matcher.matches("MAN UNITED"));
        assert!(matcher.matches("man united"));
        assert!(matcher.matches("Man United"));
    }

    #[test]
    fn test_matches_with_special_chars() {
        let team = create_test_team();
        let matcher = TeamMatcher::new(&team);

        assert!(matcher.matches("Man. United"));
        assert!(matcher.matches("Manchester United FC"));
    }

    #[test]
    fn test_matches_with_extra_spaces() {
        let team = create_test_team();
        let matcher = TeamMatcher::new(&team);

        assert!(matcher.matches("Man  United"));
        assert!(matcher.matches("  Manchester United  "));
    }

    #[test]
    fn test_does_not_match() {
        let team = create_test_team();
        let matcher = TeamMatcher::new(&team);

        assert!(!matcher.matches("Liverpool"));
        assert!(!matcher.matches("Man City"));
        assert!(!matcher.matches("Chelsea"));
    }

    #[test]
    fn test_partial_match_fails() {
        let team = create_test_team();
        let matcher = TeamMatcher::new(&team);

        // Should not match partial names
        assert!(!matcher.matches("United"));
        assert!(!matcher.matches("Manchester"));
    }

    #[test]
    fn test_matcher_performance() {
        let team = create_test_team();
        let matcher = TeamMatcher::new(&team);

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            matcher.matches("Manchester United");
        }
        let elapsed = start.elapsed();

        // Should complete 1000 matches in less than 10ms (very fast)
        assert!(elapsed.as_millis() < 10);
    }
}
