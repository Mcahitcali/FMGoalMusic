use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;

/// Convert a string to ASCII-safe slug with Turkish character support
/// - Maps Turkish letters: ç→c, ğ→g, ı→i, İ→I, ö→o, ş→s, ü→u
/// - Strips diacritics via Unicode NFD decomposition
/// - Replaces spaces and non-alphanumeric with underscores
/// - Collapses multiple underscores
/// - Trims leading/trailing underscores
pub fn slugify(input: &str) -> String {
    // Pre-allocate with input length as estimate
    let mut result = String::with_capacity(input.len());
    let mut last_was_underscore = false;

    // Single pass through NFD-normalized characters
    for ch in input.nfd() {
        // Skip combining marks (diacritics)
        if is_combining_mark(ch) {
            continue;
        }

        // Map Turkish characters that are NOT handled by Unicode normalization
        // (ı, İ, ğ, Ğ, ş, Ş are unique codepoints, not composed)
        let mapped = match ch {
            // Turkish unique codepoints
            'ğ' => 'g',
            'ı' => 'i', // dotless i
            'ş' => 's',
            'Ğ' => 'G',
            'İ' => 'I', // dotted I
            'Ş' => 'S',
            // Other composed characters that might appear
            'ĉ' => 'c',
            'Ĉ' => 'C',
            _ => ch,
        };

        // Handle spaces and valid characters
        if mapped.is_whitespace() {
            if !last_was_underscore && !result.is_empty() {
                result.push('_');
                last_was_underscore = true;
            }
        } else if mapped.is_ascii_alphanumeric() || matches!(mapped, '-' | '.') {
            result.push(mapped);
            last_was_underscore = false;
        } else if !mapped.is_control() {
            // Replace other characters with underscore
            if !last_was_underscore && !result.is_empty() {
                result.push('_');
                last_was_underscore = true;
            }
        }
        // Skip control characters entirely
    }

    // Trim trailing underscore if present
    if result.ends_with('_') {
        result.pop();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turkish_characters() {
        // Test unique codepoints (need manual mapping)
        assert_eq!(slugify("ıstanbul"), "istanbul");
        assert_eq!(slugify("İstanbul"), "Istanbul");
        assert_eq!(slugify("şağ"), "sag");
        assert_eq!(slugify("ŞAĞ"), "SAG");
        assert_eq!(slugify("ğ"), "g");
        assert_eq!(slugify("Ğ"), "G");
        
        // Test composed characters (handled by Unicode-NFD)
        assert_eq!(slugify("çağlar"), "caglar");
        assert_eq!(slugify("Çağlar"), "Caglar");
        assert_eq!(slugify("Göztepe"), "Goztepe");
        assert_eq!(slugify("GÖZTEPE"), "GOZTEPE");
        assert_eq!(slugify("Üsküdar"), "Uskudar");
        assert_eq!(slugify("ÜSKÜDAR"), "USKUDAR");
    }

    #[test]
    fn test_spaces_to_underscores() {
        assert_eq!(slugify("Hello World"), "Hello_World");
        assert_eq!(slugify("  multiple   spaces  "), "multiple_spaces");
    }

    #[test]
    fn test_diacritics() {
        assert_eq!(slugify("café"), "cafe");
        assert_eq!(slugify("naïve"), "naive");
    }

    #[test]
    fn test_special_characters() {
        assert_eq!(slugify("song (remix) – 2024"), "song_remix_2024");
        assert_eq!(slugify("track#1@test"), "track_1_test");
    }

    #[test]
    fn test_collapse_underscores() {
        assert_eq!(slugify("a___b"), "a_b");
        assert_eq!(slugify("___start"), "start");
        assert_eq!(slugify("end___"), "end");
    }
}
