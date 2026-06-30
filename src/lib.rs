//! Count words in mixed multilingual text.
//!
//! One heuristic counts words across scripts. Each run of Latin, Cyrillic, or
//! Malayalam letters between separators is one word. Each run of ASCII digits is
//! one word. Each CJK ideograph, Japanese kana, and Korean Hangul character is
//! its own word. A token made only of characters outside the known ranges
//! survives whole. That is how Arabic, Hebrew, and most Indic text get counted
//! by whitespace splitting. A token that mixes a known script with an unknown
//! one keeps only the known runs and drops the unknown remainder. Scripts
//! written without spaces between words, such as Thai, Lao, and Khmer, scan to
//! one token and count as one word.
//!
//! ```
//! use count_words::count_words;
//!
//! assert_eq!(count_words("Hello World", &Default::default()), 2);
//! assert_eq!(count_words("你好，世界", &Default::default()), 4);
//! assert_eq!(count_words("Hello \"世界\"", &Default::default()), 3);
//! ```
//!
//! Three entry points share one core. [`detect_words`] returns the word list,
//! and its [`WordsResult::count`](WordsResult::count) method reports how many
//! there are. [`count_words`] returns just the count. [`split_words`] returns
//! just the list.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod ranges;

/// Options that change how punctuation is handled.
///
/// All fields default to off. Build with `Config::default()` for the standard
/// behavior, then set individual fields.
///
/// ```
/// use count_words::{count_words, Config};
///
/// let cfg = Config { punctuation_as_breaker: true, ..Default::default() };
/// assert_eq!(count_words("Google's home", &cfg), 3);
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
    /// the built-in list, in order. Empty entries are ignored.
    pub punctuation: Vec<String>,
}

/// The result of [`detect_words`]: the detected words in input order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WordsResult {
    /// The detected words, in input order.
    pub words: Vec<String>,
}

impl WordsResult {
    /// The number of detected words.
    ///
    /// This is `self.words.len()`. The result holds no separate counter, so the
    /// count cannot drift from the list.
    #[must_use]
    pub fn count(&self) -> usize {
        self.words.len()
    }
}

/// Built-in punctuation, stripped unless [`Config::disable_default_punctuation`]
/// is set. Order matches the sequence of replacements.
const DEFAULT_PUNCTUATION: &[&str] = &[
    ",", "，", ".", "。", ":", "：", ";", "；", "[", "]", "【", "】", "{", "｛", "}", "｝", "(",
    "（", ")", "）", "<", "《", ">", "》", "$", "￥", "!", "！", "?", "？", "~", "～", "'", "’",
    "\"", "“", "”", "*", "/", "\\", "&", "%", "@", "#", "^", "、",
];

/// Detect the words in `text`.
///
/// This is the core function. [`count_words`] and [`split_words`] project one
/// part of this result.
///
/// Empty input and whitespace-only input return an empty result.
///
/// ```
/// use count_words::detect_words;
///
/// let r = detect_words("Hello, 你好。", &Default::default());
/// assert_eq!(r.words, vec!["Hello", "你", "好"]);
/// assert_eq!(r.count(), 3);
/// ```
#[must_use]
pub fn detect_words(text: &str, config: &Config) -> WordsResult {
    if is_blank(text) {
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
        // Skip empty entries. str::replace on "" splices the replacer between
        // every character, which is never what a caller means.
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
        .filter(|t| !is_blank(t))
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

    WordsResult { words: detected }
}

/// Count the words in `text`.
///
/// Thin wrapper over [`detect_words`] that returns the count.
///
/// ```
/// use count_words::count_words;
///
/// assert_eq!(count_words("Hello World", &Default::default()), 2);
/// ```
#[must_use]
pub fn count_words(text: &str, config: &Config) -> usize {
    detect_words(text, config).count()
}

/// Split `text` into its detected words.
///
/// Thin wrapper over [`detect_words`] that returns the word list.
///
/// ```
/// use count_words::split_words;
///
/// assert_eq!(split_words("100世界", &Default::default()), vec!["100", "世", "界"]);
/// ```
#[must_use]
pub fn split_words(text: &str, config: &Config) -> Vec<String> {
    detect_words(text, config).words
}

/// True when `c` lives in a block that is always stripped: General Punctuation
/// (U+2000-U+206F) or Halfwidth and Fullwidth Forms (U+FF00-U+FFEF).
fn is_symbol_block(c: char) -> bool {
    matches!(c, '\u{2000}'..='\u{206F}' | '\u{FF00}'..='\u{FFEF}')
}

/// True when `c` is whitespace for the empty guard, the token filter, and the
/// first-run collapse.
///
/// This is a fixed set, not the Unicode White_Space property. It includes
/// U+FEFF (byte order mark) and excludes U+0085 (next line). A BOM-only string
/// trims to empty and counts as zero words. A NEL-only string survives as one
/// token. `char::is_whitespace` and `str::trim` get both of these wrong, so the
/// code uses this predicate instead.
fn is_break_space(c: char) -> bool {
    matches!(c,
        '\u{0009}'..='\u{000D}'  // tab, LF, VT, FF, CR
        | '\u{0020}'              // space
        | '\u{00A0}'              // no-break space
        | '\u{1680}'              // ogham space mark
        | '\u{2000}'..='\u{200A}' // en quad through hair space
        | '\u{2028}'              // line separator
        | '\u{2029}'              // paragraph separator
        | '\u{202F}'              // narrow no-break space
        | '\u{205F}'              // medium mathematical space
        | '\u{3000}'              // ideographic space
        | '\u{FEFF}'              // zero width no-break space (BOM)
    )
}

/// True when `s` is empty or made only of `is_break_space` characters.
fn is_blank(s: &str) -> bool {
    s.chars().all(is_break_space)
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
        if is_break_space(c) {
            prefix_end = Some(idx);
            // Skip the whole first run.
            while let Some(&(next_idx, next)) = chars.peek() {
                if is_break_space(next) {
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
    let mut carry: Vec<String> = Vec::new();
    let mut chars = token.char_indices().peekable();
    while let Some(&(start, c)) = chars.peek() {
        if c.is_ascii_digit() {
            let end = run_end(&mut chars, |c| c.is_ascii_digit());
            carry.push(token[start..end].to_string());
        } else if ranges::is_latin_class(c) {
            let end = run_end(&mut chars, ranges::is_latin_class);
            carry.push(token[start..end].to_string());
        } else if ranges::is_cjk_class(c) {
            carry.push(c.to_string());
            chars.next();
        } else {
            // No alternative matched at this position. Skip the character.
            chars.next();
        }
    }
    carry
}

/// Consume the maximal run of characters that satisfy `pred` and return the byte
/// offset one past the run.
///
/// The iterator points at the first character of the run on entry. On return it
/// points at the first character that fails `pred`, or is exhausted.
fn run_end<I>(chars: &mut std::iter::Peekable<I>, pred: fn(char) -> bool) -> usize
where
    I: Iterator<Item = (usize, char)>,
{
    let mut end = 0;
    while let Some(&(idx, c)) = chars.peek() {
        if pred(c) {
            end = idx + c.len_utf8();
            chars.next();
        } else {
            break;
        }
    }
    end
}
