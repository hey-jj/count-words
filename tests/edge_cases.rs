//! Edge cases that pin behavior the language corpus does not reach: empty and
//! whitespace input, tabs and newlines, fullwidth and Arabic-Indic digits,
//! emoji, astral characters, symbol-block stripping, and custom punctuation.

use count_words::{words_count, words_detect, words_split, Config};

fn default() -> Config {
    Config::default()
}

#[test]
fn empty_string_is_zero() {
    let r = words_detect("", &default());
    assert_eq!(r.count, 0);
    assert!(r.words.is_empty());
}

#[test]
fn whitespace_only_is_zero() {
    let r = words_detect("   ", &default());
    assert_eq!(r.count, 0);
    assert!(r.words.is_empty());

    assert_eq!(words_count("\t\n  ", &default()), 0);
}

#[test]
fn tabs_and_newlines_separate_words() {
    assert_eq!(
        words_split("a\nb\tc", &default()),
        vec!["a", "b", "c"],
        "tabs and newlines split covered-script tokens"
    );
}

#[test]
fn multiple_whitespace_runs_collapse() {
    assert_eq!(words_split("a\t\tb   c", &default()), vec!["a", "b", "c"]);
}

#[test]
fn readme_example() {
    // Hello plus two Han characters, with the quotes stripped.
    assert_eq!(
        words_split("Hello \"世界\"", &default()),
        vec!["Hello", "世", "界"]
    );
    assert_eq!(words_count("Hello \"世界\"", &default()), 3);
}

#[test]
fn fullwidth_digits_are_stripped() {
    // Fullwidth digits live in U+FF10-U+FF19, inside the always-stripped block.
    // They are not ASCII digits, so they vanish and leave no word.
    assert_eq!(words_count("１２３", &default()), 0);
}

#[test]
fn arabic_indic_digits_stay_one_token() {
    // Arabic-Indic digits are not ASCII digits and not in any stripped block.
    // The token matches nothing, so it survives whole.
    assert_eq!(words_split("٢٠٢٠", &default()), vec!["٢٠٢٠"]);
    assert_eq!(words_count("٢٠٢٠", &default()), 1);
}

#[test]
fn emoji_survives_as_its_own_token() {
    assert_eq!(
        words_split("hi 😀 bye", &default()),
        vec!["hi", "😀", "bye"]
    );
}

#[test]
fn lone_emoji_is_one_word() {
    assert_eq!(words_count("😀", &default()), 1);
    // Two emoji with no separator stay one token.
    assert_eq!(words_split("😀😀", &default()), vec!["😀😀"]);
}

#[test]
fn astral_characters_drop_when_mixed() {
    // U+1F600 emoji between two ASCII letters: the emoji is not in any range and
    // is discarded because the token has other matches.
    assert_eq!(words_split("a😀b", &default()), vec!["a", "b"]);
    // U+20000 is a Han ideograph but lives above the BMP, so it is not counted
    // as CJK. It drops out the same way.
    assert_eq!(words_split("a\u{20000}b", &default()), vec!["a", "b"]);
}

#[test]
fn number_only_token() {
    assert_eq!(words_count("100", &default()), 1);
}

#[test]
fn glued_digit_letter_runs_split() {
    assert_eq!(
        words_split("abc123def", &default()),
        vec!["abc", "123", "def"]
    );
}

#[test]
fn glued_digit_cjk_splits() {
    assert_eq!(words_split("100世界", &default()), vec!["100", "世", "界"]);
}

#[test]
fn matched_token_drops_unmatched_remainder() {
    // The Latin prefix matches, so the Arabic suffix is dropped.
    assert_eq!(
        words_split(
            "abc\u{0627}\u{0644}\u{0639}\u{0631}\u{0628}\u{064A}\u{0629}",
            &default()
        ),
        vec!["abc"]
    );
}

#[test]
fn en_dash_stripped_by_symbol_block() {
    // U+2013 sits in General Punctuation and is removed, joining the letters.
    assert_eq!(words_split("a\u{2013}b", &default()), vec!["ab"]);
}

#[test]
fn curly_quotes_stripped_by_symbol_block() {
    // U+2018 and U+2019 are removed by the symbol-block pass.
    assert_eq!(words_split("\u{2018}x\u{2019}", &default()), vec!["x"]);
}

#[test]
fn disable_default_punctuation_keeps_bare_comma() {
    // With defaults off, the comma is not stripped, but the tokenizer still finds
    // two Latin runs and skips the comma between them.
    let cfg = Config {
        disable_default_punctuation: true,
        ..Default::default()
    };
    assert_eq!(words_split("a,b", &cfg), vec!["a", "b"]);
    assert_eq!(words_count("a,b", &cfg), 2);
}

#[test]
fn custom_punctuation_removes_marks() {
    let cfg = Config {
        punctuation: vec!["a".into(), "b".into()],
        ..Default::default()
    };
    // "Googles" has no 'a' or 'b', so it is unchanged.
    assert_eq!(words_split("Googles", &cfg), vec!["Googles"]);
    // "Gabble" loses a and b and the rest joins into one run.
    assert_eq!(words_split("Gabble", &cfg), vec!["Gle"]);
}

#[test]
fn custom_punctuation_as_breaker_splits_token() {
    let cfg = Config {
        punctuation: vec!["a".into(), "b".into()],
        punctuation_as_breaker: true,
        ..Default::default()
    };
    // Each removed mark becomes a space, so "Gabble" breaks into two runs.
    assert_eq!(words_split("Gabble", &cfg), vec!["G", "le"]);
}

#[test]
fn tab_between_uncovered_words_stays_one_token() {
    // The leading space is the first whitespace run and becomes the split point.
    // The tab survives, and the Arabic token matches nothing, so the two Arabic
    // words separated by a tab count as one word.
    let input = "hi \u{0627}\u{0644}\u{0644}\u{063A}\u{0629}\t\u{0627}\u{0644}\u{0639}\u{0631}\u{0628}\u{064A}\u{0629}";
    let r = words_split(input, &default());
    assert_eq!(r.len(), 2);
    assert_eq!(r[0], "hi");
    assert!(r[1].contains('\t'), "tab stays embedded in the token");
}
