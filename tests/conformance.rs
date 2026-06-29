//! Golden corpus. Every input and expected value comes from a fixed table of
//! known-good results across many languages and config combinations. The table
//! lives in `fixtures/cases.json` and is embedded at compile time.
//!
//! Each count case checks `words_count`. Each words case checks the full word
//! list from all three entry points, which is stricter than checking length.

use count_words::{words_count, words_detect, words_split, Config};
use serde::Deserialize;

#[derive(Deserialize)]
struct JsonConfig {
    #[serde(default)]
    #[serde(rename = "punctuationAsBreaker")]
    punctuation_as_breaker: bool,
    #[serde(default)]
    #[serde(rename = "disableDefaultPunctuation")]
    disable_default_punctuation: bool,
    #[serde(default)]
    punctuation: Vec<String>,
}

#[derive(Deserialize)]
struct CountCase {
    name: String,
    input: String,
    config: Option<JsonConfig>,
    expected: usize,
}

#[derive(Deserialize)]
struct WordsCase {
    name: String,
    input: String,
    config: Option<JsonConfig>,
    expected: Vec<String>,
}

#[derive(Deserialize)]
struct Corpus {
    count_cases: Vec<CountCase>,
    words_cases: Vec<WordsCase>,
}

fn to_config(json: &Option<JsonConfig>) -> Config {
    match json {
        None => Config::default(),
        Some(j) => Config {
            punctuation_as_breaker: j.punctuation_as_breaker,
            disable_default_punctuation: j.disable_default_punctuation,
            punctuation: j.punctuation.clone(),
        },
    }
}

const CASES: &str = include_str!("fixtures/cases.json");

fn corpus() -> Corpus {
    serde_json::from_str(CASES).expect("cases.json parses")
}

#[test]
fn count_cases_match() {
    let corpus = corpus();
    assert!(corpus.count_cases.len() >= 110, "expected the full corpus");
    for case in &corpus.count_cases {
        let cfg = to_config(&case.config);
        let got = words_count(&case.input, &cfg);
        assert_eq!(got, case.expected, "count mismatch for {}", case.name);
    }
}

#[test]
fn words_cases_match() {
    let corpus = corpus();
    assert!(!corpus.words_cases.is_empty());
    for case in &corpus.words_cases {
        let cfg = to_config(&case.config);

        let split = words_split(&case.input, &cfg);
        assert_eq!(split, case.expected, "split mismatch for {}", case.name);

        let detected = words_detect(&case.input, &cfg);
        assert_eq!(
            detected.words, case.expected,
            "detect.words mismatch for {}",
            case.name
        );
        assert_eq!(
            detected.count,
            case.expected.len(),
            "detect.count mismatch for {}",
            case.name
        );
    }
}
