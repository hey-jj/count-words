//! Count words in mixed multilingual text.
//!
//! One heuristic counts words across scripts. Each run of Latin, Cyrillic, or
//! Malayalam letters between separators is one word. Each run of ASCII digits is
//! one word. Each CJK ideograph, Japanese kana, and Korean Hangul character is
//! its own word. Characters outside the known ranges fall through to a
//! whole-token fallback, which is how Arabic, Hebrew, Thai, and most Indic text
//! get counted by whitespace splitting.
//!
//! ```
//! use count_words::words_count;
//!
//! assert_eq!(words_count("Hello World", &Default::default()), 2);
//! assert_eq!(words_count("你好，世界", &Default::default()), 4);
//! assert_eq!(words_count("Hello \"世界\"", &Default::default()), 3);
//! ```
//!
//! Three entry points share one core. [`words_detect`] returns both the word
//! list and the count. [`words_count`] returns just the count. [`words_split`]
//! returns just the list.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod ranges;

/// Options that change how punctuation is handled.
///
/// All fields default to off. Build with `Config::default()` for the standard
/// behavior, then set individual fields.
///
/// ```
/// use count_words::{words_count, Config};
///
/// let cfg = Config { punctuation_as_breaker: true, ..Default::default() };
/// assert_eq!(words_count("Google's home", &cfg), 3);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    /// Replace punctuation with a space instead of deleting it.
    ///
    /// Off by default, so punctuation is removed and the surrounding text joins
    /// into one token. When on, each punctuation mark becomes a space and can
    /// split a token in two.
    pub punctuation_as_breaker: bool,
    /// Skip the built-in punctuation list.
    ///
    /// Off by default, so the built-in marks are stripped. When on, only the
    /// marks in [`Config::punctuation`] are handled.
    pub disable_default_punctuation: bool,
    /// Extra punctuation marks to strip or break on.
    ///
    /// Each entry is treated as a literal substring. Entries are applied after
    /// the built-in list, in order.
    pub punctuation: Vec<String>,
}

/// The result of [`words_detect`]: the detected words and how many there are.
///
/// `count` always equals `words.len()`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WordsResult {
    /// The detected words, in input order.
    pub words: Vec<String>,
    /// The number of detected words. Always equals `words.len()`.
    pub count: usize,
}

/// Built-in punctuation, stripped unless [`Config::disable_default_punctuation`]
/// is set. Order matches the sequence of replacements.
const DEFAULT_PUNCTUATION: &[&str] = &[
    ",", "，", ".", "。", ":", "：", ";", "；", "[", "]", "【", "]", "】", "{", "｛", "}", "｝",
    "(", "（", ")", "）", "<", "《", ">", "》", "$", "￥", "!", "！", "?", "？", "~", "～", "'",
    "’", "\"", "“", "”", "*", "/", "\\", "&", "%", "@", "#", "^", "、", "、", "、", "、",
];

/// Detect words in `text` and return them with their count.
///
/// This is the core function. [`words_count`] and [`words_split`] project one
/// field out of this result.
///
/// Empty input and whitespace-only input return an empty result.
///
/// ```
/// use count_words::words_detect;
///
/// let r = words_detect("Hello, 你好。", &Default::default());
/// assert_eq!(r.words, vec!["Hello", "你", "好"]);
/// assert_eq!(r.count, 3);
/// ```
pub fn words_detect(text: &str, config: &Config) -> WordsResult {
    if text.is_empty() || text.trim().is_empty() {
        return WordsResult::default();
    }

    let replacer = if config.punctuation_as_breaker {
        " "
    } else {
        ""
    };

    // Strip or break on punctuation. Defaults first, then custom entries, each
    // as a literal substring replaced everywhere.
    let mut words = text.to_string();
    if !config.disable_default_punctuation {
        for p in DEFAULT_PUNCTUATION {
            words = words.replace(p, replacer);
        }
    }
    for p in &config.punctuation {
        if !p.is_empty() {
            words = words.replace(p.as_str(), replacer);
        }
    }

    // Drop General Punctuation and Halfwidth/Fullwidth Forms. This removes smart
    // quotes, dashes, ellipses, fullwidth commas, and fullwidth digits.
    words.retain(|c| !is_symbol_block(c));

    // Collapse only the first whitespace run, then split on the single ASCII
    // space. Tokens that trim to empty are dropped.
    let collapsed = collapse_first_whitespace_run(&words);
    let tokens = collapsed
        .split(' ')
        .filter(|t| !t.trim().is_empty())
        .collect::<Vec<_>>();

    let mut detected: Vec<String> = Vec::new();
    for token in tokens {
        let carry = scan_token(token);
        if carry.is_empty() {
            detected.push(token.to_string());
        } else {
            detected.extend(carry);
        }
    }

    let count = detected.len();
    WordsResult {
        words: detected,
        count,
    }
}

/// Count the words in `text`.
///
/// Thin wrapper over [`words_detect`] that returns the count.
///
/// ```
/// use count_words::words_count;
///
/// assert_eq!(words_count("Hello World", &Default::default()), 2);
/// ```
pub fn words_count(text: &str, config: &Config) -> usize {
    words_detect(text, config).count
}

/// Split `text` into its detected words.
///
/// Thin wrapper over [`words_detect`] that returns the word list.
///
/// ```
/// use count_words::words_split;
///
/// assert_eq!(words_split("100世界", &Default::default()), vec!["100", "世", "界"]);
/// ```
pub fn words_split(text: &str, config: &Config) -> Vec<String> {
    words_detect(text, config).words
}

/// True when `c` lives in a block that is always stripped: General Punctuation
/// (U+2000-U+206F) or Halfwidth and Fullwidth Forms (U+FF00-U+FFEF).
fn is_symbol_block(c: char) -> bool {
    matches!(c, '\u{2000}'..='\u{206F}' | '\u{FF00}'..='\u{FFEF}')
}

/// Replace the first maximal run of whitespace with a single ASCII space.
///
/// This mirrors a non-global regex replace. Only the first run changes. Later
/// runs stay as written, which matters for tokens that the scanner cannot match.
fn collapse_first_whitespace_run(s: &str) -> String {
    let mut chars = s.char_indices().peekable();
    let mut prefix_end = None;
    let mut rest_start = None;
    while let Some(&(idx, c)) = chars.peek() {
        if c.is_whitespace() {
            prefix_end = Some(idx);
            // Skip the whole first run.
            while let Some(&(next_idx, next)) = chars.peek() {
                if next.is_whitespace() {
                    chars.next();
                } else {
                    rest_start = Some(next_idx);
                    break;
                }
            }
            break;
        }
        chars.next();
    }

    match prefix_end {
        None => s.to_string(),
        Some(end) => {
            let mut out = String::with_capacity(s.len());
            out.push_str(&s[..end]);
            out.push(' ');
            if let Some(start) = rest_start {
                out.push_str(&s[start..]);
            }
            out
        }
    }
}

/// Scan one whitespace token into words.
///
/// Walks the token left to right. At each position it tries, in order: a run of
/// ASCII digits, a run of Latin/Cyrillic/Malayalam letters, then a single
/// CJK/kana/Hangul character. Characters that match nothing are skipped. The
/// returned list is empty when the token has no matchable character, which tells
/// the caller to keep the token whole.
fn scan_token(token: &str) -> Vec<String> {
    let chars: Vec<char> = token.chars().collect();
    let mut carry: Vec<String> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            carry.push(chars[start..i].iter().collect());
        } else if ranges::is_latin_class(c) {
            let start = i;
            while i < chars.len() && ranges::is_latin_class(chars[i]) {
                i += 1;
            }
            carry.push(chars[start..i].iter().collect());
        } else if ranges::is_cjk_class(c) {
            carry.push(c.to_string());
            i += 1;
        } else {
            // No alternative matched at this position. Skip the character.
            i += 1;
        }
    }
    carry
}
