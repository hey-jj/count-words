//! Unicode block ranges for character classification.
//!
//! These ranges come straight from the heuristic. They are ad-hoc block slices,
//! not formal Unicode categories. The CJK class is over-broad on purpose: it
//! includes CJK symbols, enclosed forms, and radicals. A character that lands in
//! these ranges counts as one word on its own.
//!
//! All ranges are in the Basic Multilingual Plane. Characters at or above
//! U+10000 (astral plane) match nothing here, so an astral ideograph or emoji is
//! never counted as a CJK word.

/// True when `c` is a Latin, Cyrillic, or Malayalam letter that joins a run.
///
/// A maximal run of these is one word.
pub(crate) fn is_latin_class(c: char) -> bool {
    matches!(c,
        'a'..='z'
        | 'A'..='Z'
        | '\u{00C0}'..='\u{00FF}'  // Latin-1 Supplement letters
        | '\u{0100}'..='\u{017F}'  // Latin Extended-A
        | '\u{0180}'..='\u{024F}'  // Latin Extended-B
        | '\u{0250}'..='\u{02AF}'  // IPA Extensions
        | '\u{1E00}'..='\u{1EFF}'  // Latin Extended Additional
        | '\u{0400}'..='\u{04FF}'  // Cyrillic
        | '\u{0500}'..='\u{052F}'  // Cyrillic Supplement
        | '\u{0D00}'..='\u{0D7F}'  // Malayalam
    )
}

/// True when `c` is a CJK ideograph, Japanese kana, or Korean Hangul character.
///
/// Each such character is one word on its own.
pub(crate) fn is_cjk_class(c: char) -> bool {
    is_cjk(c) || is_jp(c) || is_kr(c)
}

/// Chinese Hanzi, Japanese Kanji, Korean Hanja, plus CJK symbols and radicals.
fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{2E80}'..='\u{2EFF}'  // CJK Radicals Supplement
        | '\u{2F00}'..='\u{2FDF}'  // Kangxi Radicals
        | '\u{3001}'..='\u{303F}'  // CJK Symbols and Punctuation (U+3000 is whitespace)
        | '\u{31C0}'..='\u{31EF}'  // CJK Strokes
        | '\u{3200}'..='\u{32FF}'  // Enclosed CJK Letters and Months
        | '\u{3300}'..='\u{33FF}'  // CJK Compatibility
        | '\u{3400}'..='\u{9FFF}'  // CJK Ext-A through Unified Ideographs
        | '\u{F900}'..='\u{FAFF}'  // CJK Compatibility Ideographs
    )
}

/// Japanese Hiragana, Katakana, and kana extensions.
fn is_jp(c: char) -> bool {
    matches!(c,
        '\u{3040}'..='\u{309F}'  // Hiragana
        | '\u{30A0}'..='\u{30FF}'  // Katakana
        | '\u{31F0}'..='\u{31FF}'  // Katakana Phonetic Extensions
        | '\u{3190}'..='\u{319F}'  // Kanbun
    )
}

/// Korean Hangul jamo and syllables.
fn is_kr(c: char) -> bool {
    matches!(c,
        '\u{1100}'..='\u{11FF}'  // Hangul Jamo
        | '\u{3130}'..='\u{318F}'  // Hangul Compatibility Jamo
        | '\u{A960}'..='\u{A97F}'  // Hangul Jamo Extended-A
        | '\u{AC00}'..='\u{D7FF}'  // Hangul Syllables and Jamo Extended-B
    )
}
