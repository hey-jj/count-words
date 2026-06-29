//! Edge cases that pin behavior the language corpus does not reach: empty and
//! whitespace input, tabs and newlines, fullwidth and Arabic-Indic digits,
//! emoji, astral characters, symbol-block stripping, and custom punctuation.

use count_words::{count_words, detect_words, split_words, Config};

fn default() -> Config {
    Config::default()
}

#[test]
fn empty_string_is_zero() {
    let r = detect_words("", &default());
    assert_eq!(r.count(), 0);
    assert!(r.words.is_empty());
}

#[test]
fn whitespace_only_is_zero() {
    let r = detect_words("   ", &default());
    assert_eq!(r.count(), 0);
    assert!(r.words.is_empty());

    assert_eq!(count_words("\t\n  ", &default()), 0);
}

#[test]
fn tabs_and_newlines_separate_words() {
    assert_eq!(
        split_words("a\nb\tc", &default()),
        vec!["a", "b", "c"],
        "tabs and newlines split covered-script tokens"
    );
}

#[test]
fn multiple_whitespace_runs_collapse() {
    assert_eq!(split_words("a\t\tb   c", &default()), vec!["a", "b", "c"]);
}

#[test]
fn readme_example() {
    // Hello plus two Han characters, with the quotes stripped.
    assert_eq!(
        split_words("Hello \"世界\"", &default()),
        vec!["Hello", "世", "界"]
    );
    assert_eq!(count_words("Hello \"世界\"", &default()), 3);
}

#[test]
fn fullwidth_digits_are_stripped() {
    // Fullwidth digits live in U+FF10-U+FF19, inside the always-stripped block.
    // They are not ASCII digits, so they vanish and leave no word.
    assert_eq!(count_words("１２３", &default()), 0);
}

#[test]
fn arabic_indic_digits_stay_one_token() {
    // Arabic-Indic digits are not ASCII digits and not in any stripped block.
    // The token matches nothing, so it survives whole.
    assert_eq!(split_words("٢٠٢٠", &default()), vec!["٢٠٢٠"]);
    assert_eq!(count_words("٢٠٢٠", &default()), 1);
}

#[test]
fn emoji_survives_as_its_own_token() {
    assert_eq!(
        split_words("hi 😀 bye", &default()),
        vec!["hi", "😀", "bye"]
    );
}

#[test]
fn lone_emoji_is_one_word() {
    assert_eq!(count_words("😀", &default()), 1);
    // Two emoji with no separator stay one token.
    assert_eq!(split_words("😀😀", &default()), vec!["😀😀"]);
}

#[test]
fn astral_characters_drop_when_mixed() {
    // U+1F600 emoji between two ASCII letters: the emoji is not in any range and
    // is discarded because the token has other matches.
    assert_eq!(split_words("a😀b", &default()), vec!["a", "b"]);
    // U+20000 is a Han ideograph but lives above the BMP, so it is not counted
    // as CJK. It drops out the same way.
    assert_eq!(split_words("a\u{20000}b", &default()), vec!["a", "b"]);
}

#[test]
fn number_only_token() {
    assert_eq!(count_words("100", &default()), 1);
}

#[test]
fn glued_digit_letter_runs_split() {
    assert_eq!(
        split_words("abc123def", &default()),
        vec!["abc", "123", "def"]
    );
}

#[test]
fn glued_digit_cjk_splits() {
    assert_eq!(split_words("100世界", &default()), vec!["100", "世", "界"]);
}

#[test]
fn matched_token_drops_unmatched_remainder() {
    // The Latin prefix matches, so the Arabic suffix is dropped.
    assert_eq!(
        split_words(
            "abc\u{0627}\u{0644}\u{0639}\u{0631}\u{0628}\u{064A}\u{0629}",
            &default()
        ),
        vec!["abc"]
    );
}

#[test]
fn en_dash_stripped_by_symbol_block() {
    // U+2013 sits in General Punctuation and is removed, joining the letters.
    assert_eq!(split_words("a\u{2013}b", &default()), vec!["ab"]);
}

#[test]
fn curly_quotes_stripped_by_symbol_block() {
    // U+2018 and U+2019 are removed by the symbol-block pass.
    assert_eq!(split_words("\u{2018}x\u{2019}", &default()), vec!["x"]);
}

#[test]
fn disable_default_punctuation_keeps_bare_comma() {
    // With defaults off, the comma is not stripped, but the tokenizer still finds
    // two Latin runs and skips the comma between them.
    let cfg = Config {
        disable_default_punctuation: true,
        ..Default::default()
    };
    assert_eq!(split_words("a,b", &cfg), vec!["a", "b"]);
    assert_eq!(count_words("a,b", &cfg), 2);
}

#[test]
fn custom_punctuation_removes_marks() {
    let cfg = Config {
        punctuation: vec!["a".into(), "b".into()],
        ..Default::default()
    };
    // "Googles" has no 'a' or 'b', so it is unchanged.
    assert_eq!(split_words("Googles", &cfg), vec!["Googles"]);
    // "Gabble" loses a and b and the rest joins into one run.
    assert_eq!(split_words("Gabble", &cfg), vec!["Gle"]);
}

#[test]
fn custom_punctuation_as_breaker_splits_token() {
    let cfg = Config {
        punctuation: vec!["a".into(), "b".into()],
        punctuation_as_breaker: true,
        ..Default::default()
    };
    // Each removed mark becomes a space, so "Gabble" breaks into two runs.
    assert_eq!(split_words("Gabble", &cfg), vec!["G", "le"]);
}

#[test]
fn tab_between_uncovered_words_stays_one_token() {
    // The leading space is the first whitespace run and becomes the split point.
    // The tab survives, and the Arabic token matches nothing, so the two Arabic
    // words separated by a tab count as one word.
    let input = "hi \u{0627}\u{0644}\u{0644}\u{063A}\u{0629}\t\u{0627}\u{0644}\u{0639}\u{0631}\u{0628}\u{064A}\u{0629}";
    let r = split_words(input, &default());
    assert_eq!(r.len(), 2);
    assert_eq!(r[0], "hi");
    assert!(r[1].contains('\t'), "tab stays embedded in the token");
}

#[test]
fn bom_counts_as_whitespace() {
    // U+FEFF is whitespace for this crate. A BOM-only string trims to empty and
    // returns zero words. Between two uncovered tokens it splits them apart.
    assert_eq!(split_words("\u{FEFF}", &default()), Vec::<String>::new());
    assert_eq!(count_words("\u{FEFF}", &default()), 0);
    assert_eq!(
        split_words("\u{0627}\u{0644}\u{FEFF}\u{0627}\u{0644}", &default()),
        vec!["\u{0627}\u{0644}", "\u{0627}\u{0644}"]
    );
}

#[test]
fn nel_is_not_whitespace() {
    // U+0085 (NEL) is not whitespace for this crate, unlike Rust's str::trim. A
    // NEL-only string survives as one token. Embedded in an uncovered token it
    // stays put and does not split.
    assert_eq!(count_words("\u{0085}", &default()), 1);
    assert_eq!(split_words("\u{0085}", &default()), vec!["\u{0085}"]);
    assert_eq!(
        split_words("\u{0627}\u{0644}\u{0085}\u{0627}\u{0644}", &default()),
        vec!["\u{0627}\u{0644}\u{0085}\u{0627}\u{0644}"]
    );
}

#[test]
fn nel_after_first_run_stays_in_token() {
    // The space is the first whitespace run and becomes the split point. The NEL
    // that follows is not whitespace, so the second token keeps it whole.
    let r = split_words(
        "\u{0627}\u{0644}\u{0085}\u{0627}\u{0644} \u{0627}\u{0644}",
        &default(),
    );
    assert_eq!(
        r,
        vec![
            "\u{0627}\u{0644}\u{0085}\u{0627}\u{0644}",
            "\u{0627}\u{0644}"
        ]
    );
}

#[test]
fn multi_char_custom_punctuation_is_a_literal_substring() {
    // A custom entry of more than one character is matched as a literal
    // substring, replaced everywhere it appears.
    let oo = Config {
        punctuation: vec!["oo".into()],
        ..Default::default()
    };
    assert_eq!(split_words("Google book", &oo), vec!["Ggle", "bk"]);

    let ab = Config {
        punctuation: vec!["ab".into()],
        ..Default::default()
    };
    assert_eq!(split_words("Gabble", &ab), vec!["Gble"]);
}

#[test]
fn regex_metacharacter_custom_punctuation_is_literal() {
    // Each entry is a literal substring, so regex metacharacters match
    // themselves. disable_default_punctuation isolates the custom mark.
    let cases: &[(&str, &str)] = &[
        ("a.b.c", "."),
        ("a$b", "$"),
        ("a*b", "*"),
        ("a\\b", "\\"),
        ("a^b", "^"),
        ("a+b", "+"),
        ("a(b", "("),
        ("a|b", "|"),
    ];
    for (input, mark) in cases {
        let cfg = Config {
            punctuation: vec![(*mark).into()],
            disable_default_punctuation: true,
            ..Default::default()
        };
        let expected = if *input == "a.b.c" {
            vec!["abc"]
        } else {
            vec!["ab"]
        };
        assert_eq!(split_words(input, &cfg), expected, "mark {mark:?}");
    }
}

#[test]
fn empty_custom_punctuation_is_ignored() {
    // An empty entry is skipped, so the text passes through untouched. This
    // avoids splicing the replacer between every character.
    let cfg = Config {
        punctuation: vec!["".into()],
        disable_default_punctuation: true,
        ..Default::default()
    };
    assert_eq!(split_words("abc", &cfg), vec!["abc"]);
}
