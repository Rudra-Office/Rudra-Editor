//! Unicode script detection and text splitting.
//!
//! Detects Unicode scripts in text and splits runs at script boundaries
//! so each segment can be shaped with the correct script tag.

use unicode_script::{Script, UnicodeScript};

/// A contiguous run of text belonging to the same Unicode script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptRun {
    /// Byte offset in the source text where this run starts.
    pub start: usize,
    /// Byte offset in the source text where this run ends (exclusive).
    pub end: usize,
    /// The resolved script for this run.
    pub script: Script,
}

/// Split text into runs of the same Unicode script.
///
/// Common and Inherited characters (spaces, punctuation, diacritics) are
/// absorbed into the surrounding script run rather than creating separate
/// segments. This ensures that "Hello, world!" is one Latin run, and
/// "مرحبا بالعالم" is one Arabic run, even though spaces are Common.
pub fn split_by_script(text: &str) -> Vec<ScriptRun> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut runs: Vec<ScriptRun> = Vec::new();
    let mut current_script = Script::Common;
    let mut run_start = 0;

    for (byte_idx, ch) in text.char_indices() {
        let ch_script = ch.script();

        // Common and Inherited scripts inherit from the surrounding run
        if ch_script == Script::Common || ch_script == Script::Inherited {
            continue;
        }

        if current_script == Script::Common {
            // First non-Common character sets the run's script
            current_script = ch_script;
        } else if ch_script != current_script {
            // Script change — emit the previous run
            runs.push(ScriptRun {
                start: run_start,
                end: byte_idx,
                script: current_script,
            });
            run_start = byte_idx;
            current_script = ch_script;
        }
    }

    // Emit the final run
    runs.push(ScriptRun {
        start: run_start,
        end: text.len(),
        script: current_script,
    });

    runs
}

/// Map a Unicode script to a rustybuzz `Script` tag.
///
/// Returns `None` for Common/Inherited scripts (let rustybuzz auto-detect).
pub fn script_to_rustybuzz(script: Script) -> Option<rustybuzz::Script> {
    // rustybuzz uses predefined constants in rustybuzz::script module
    match script {
        Script::Arabic => Some(rustybuzz::script::ARABIC),
        Script::Armenian => Some(rustybuzz::script::ARMENIAN),
        Script::Bengali => Some(rustybuzz::script::BENGALI),
        Script::Bopomofo => Some(rustybuzz::script::BOPOMOFO),
        Script::Cherokee => Some(rustybuzz::script::CHEROKEE),
        Script::Cyrillic => Some(rustybuzz::script::CYRILLIC),
        Script::Devanagari => Some(rustybuzz::script::DEVANAGARI),
        Script::Ethiopic => Some(rustybuzz::script::ETHIOPIC),
        Script::Georgian => Some(rustybuzz::script::GEORGIAN),
        Script::Greek => Some(rustybuzz::script::GREEK),
        Script::Gujarati => Some(rustybuzz::script::GUJARATI),
        Script::Gurmukhi => Some(rustybuzz::script::GURMUKHI),
        Script::Han => Some(rustybuzz::script::HAN),
        Script::Hangul => Some(rustybuzz::script::HANGUL),
        Script::Hebrew => Some(rustybuzz::script::HEBREW),
        Script::Hiragana => Some(rustybuzz::script::HIRAGANA),
        Script::Kannada => Some(rustybuzz::script::KANNADA),
        Script::Katakana => Some(rustybuzz::script::KATAKANA),
        Script::Khmer => Some(rustybuzz::script::KHMER),
        Script::Lao => Some(rustybuzz::script::LAO),
        Script::Latin => Some(rustybuzz::script::LATIN),
        Script::Malayalam => Some(rustybuzz::script::MALAYALAM),
        Script::Oriya => Some(rustybuzz::script::ORIYA),
        Script::Tamil => Some(rustybuzz::script::TAMIL),
        Script::Telugu => Some(rustybuzz::script::TELUGU),
        Script::Thai => Some(rustybuzz::script::THAI),
        Script::Tibetan => Some(rustybuzz::script::TIBETAN),
        _ => None,
    }
}

/// Default OpenType features to enable for text shaping.
///
/// These features are enabled by default in most professional text renderers:
/// - `kern` — Kerning (pair-wise glyph spacing)
/// - `liga` — Standard ligatures (fi, fl, etc.)
/// - `clig` — Contextual ligatures
/// - `calt` — Contextual alternates
pub fn default_shaping_features() -> Vec<crate::types::FontFeature> {
    vec![
        crate::types::FontFeature::enabled(*b"kern"),
        crate::types::FontFeature::enabled(*b"liga"),
        crate::types::FontFeature::enabled(*b"clig"),
        crate::types::FontFeature::enabled(*b"calt"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_latin_only() {
        let runs = split_by_script("Hello World");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].script, Script::Latin);
        assert_eq!(runs[0].start, 0);
        assert_eq!(runs[0].end, 11);
    }

    #[test]
    fn split_arabic_only() {
        let text = "مرحبا بالعالم";
        let runs = split_by_script(text);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].script, Script::Arabic);
    }

    #[test]
    fn split_mixed_latin_arabic() {
        let text = "Hello مرحبا World";
        let runs = split_by_script(text);
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0].script, Script::Latin);
        assert_eq!(runs[1].script, Script::Arabic);
        assert_eq!(runs[2].script, Script::Latin);
    }

    #[test]
    fn split_cjk() {
        let text = "你好世界";
        let runs = split_by_script(text);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].script, Script::Han);
    }

    #[test]
    fn split_mixed_latin_cjk() {
        let text = "Hello你好World";
        let runs = split_by_script(text);
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0].script, Script::Latin);
        assert_eq!(runs[1].script, Script::Han);
        assert_eq!(runs[2].script, Script::Latin);
    }

    #[test]
    fn split_devanagari() {
        let text = "नमस्ते दुनिया";
        let runs = split_by_script(text);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].script, Script::Devanagari);
    }

    #[test]
    fn split_empty() {
        let runs = split_by_script("");
        assert!(runs.is_empty());
    }

    #[test]
    fn split_common_only() {
        // All Common characters (numbers, punctuation)
        let runs = split_by_script("123, 456!");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].script, Script::Common);
    }

    #[test]
    fn script_to_rustybuzz_mapping() {
        assert!(script_to_rustybuzz(Script::Arabic).is_some());
        assert!(script_to_rustybuzz(Script::Latin).is_some());
        assert!(script_to_rustybuzz(Script::Han).is_some());
        assert!(script_to_rustybuzz(Script::Devanagari).is_some());
        assert!(script_to_rustybuzz(Script::Thai).is_some());
        // Common script should return None (auto-detect)
        assert!(script_to_rustybuzz(Script::Common).is_none());
    }

    #[test]
    fn default_features_not_empty() {
        let features = default_shaping_features();
        assert!(features.len() >= 4);
        // Verify kern and liga are present
        assert!(features.iter().any(|f| &f.tag == b"kern"));
        assert!(features.iter().any(|f| &f.tag == b"liga"));
    }
}
