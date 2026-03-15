//! Hyphenation support via the `hyphenation` crate.
//!
//! Provides word-level hyphenation using Knuth-Liang patterns. Currently
//! supports English (US) with embedded dictionary data so no external files
//! are required at runtime.
//!
//! # Examples
//!
//! ```
//! use s1_text::hyphenation::hyphenate_word;
//!
//! let breaks = hyphenate_word("programming", "en-US");
//! assert!(!breaks.is_empty());
//! ```

use hyphenation::{Hyphenator, Language, Load, Standard};

/// Find valid hyphenation break points in a word.
///
/// Returns a sorted list of byte offsets within the word where a hyphen may be
/// inserted. Each offset marks a position *between* two characters — the word
/// can be split at `word[..offset]` + "-" and `word[offset..]`.
///
/// Returns an empty list when:
/// - The word has fewer than 5 characters (too short to hyphenate meaningfully).
/// - The language tag is not supported.
/// - The embedded dictionary cannot be loaded (should not happen for English).
///
/// # Arguments
///
/// * `word` — A single word (no spaces or punctuation).
/// * `lang` — A BCP-47-style language tag such as `"en"`, `"en-US"`, or
///   `"en-GB"`. Only English variants are currently supported.
///
/// # Errors
///
/// This function does not return errors; unsupported inputs simply produce an
/// empty vector.
pub fn hyphenate_word(word: &str, lang: &str) -> Vec<usize> {
    // Words shorter than 5 characters rarely benefit from hyphenation and the
    // dictionary's own minima would exclude them anyway.
    if word.chars().count() < 5 {
        return Vec::new();
    }

    let language = match lang {
        "en" | "en-US" | "en-GB" => Language::EnglishUS,
        _ => return Vec::new(),
    };

    let dict = match Standard::from_embedded(language) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let hyphenated = dict.hyphenate(word);
    hyphenated.breaks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hyphenate_programming() {
        let breaks = hyphenate_word("programming", "en-US");
        // "pro-gram-ming" → expect break points inside the word.
        assert!(
            !breaks.is_empty(),
            "expected hyphenation breaks for 'programming'"
        );
        // Each break must be a valid UTF-8 boundary inside the word.
        for &b in &breaks {
            assert!(
                b > 0 && b < "programming".len(),
                "break {b} out of range for 'programming'"
            );
            assert!(
                "programming".is_char_boundary(b),
                "break {b} is not a char boundary"
            );
        }
    }

    #[test]
    fn short_words_no_breaks() {
        assert!(hyphenate_word("the", "en").is_empty());
        assert!(hyphenate_word("do", "en-US").is_empty());
        assert!(hyphenate_word("it", "en-GB").is_empty());
        assert!(hyphenate_word("a", "en").is_empty());
    }

    #[test]
    fn unsupported_language_returns_empty() {
        assert!(hyphenate_word("programmierung", "de").is_empty());
        assert!(hyphenate_word("programmation", "fr").is_empty());
        assert!(hyphenate_word("programming", "xx-ZZ").is_empty());
    }

    #[test]
    fn empty_and_whitespace_words() {
        assert!(hyphenate_word("", "en").is_empty());
        assert!(hyphenate_word("   ", "en").is_empty());
    }

    #[test]
    fn en_gb_maps_to_english() {
        // en-GB should still produce breaks (mapped to EnglishUS dictionary).
        let breaks = hyphenate_word("international", "en-GB");
        assert!(
            !breaks.is_empty(),
            "expected breaks for 'international' with en-GB"
        );
    }

    #[test]
    fn breaks_are_sorted() {
        let breaks = hyphenate_word("international", "en-US");
        for window in breaks.windows(2) {
            assert!(
                window[0] < window[1],
                "breaks should be strictly increasing: {:?}",
                breaks
            );
        }
    }

    #[test]
    fn segments_reconstruct_word() {
        let word = "hyphenation";
        let breaks = hyphenate_word(word, "en");
        // Splitting at the break points and rejoining must produce the original word.
        let mut segments = Vec::new();
        let mut prev = 0;
        for &b in &breaks {
            segments.push(&word[prev..b]);
            prev = b;
        }
        segments.push(&word[prev..]);
        let reconstructed: String = segments.concat();
        assert_eq!(reconstructed, word);
    }
}
