//! The three entry points share one core. These tests pin that relationship and
//! the default-config ergonomics.

use count_words::{count_words, detect_words, split_words, Config};

#[test]
fn three_functions_agree() {
    let inputs = [
        "Hello, 世界 100",
        "Google's free service",
        "你好，世界",
        "",
        "   ",
        "abc123def 😀",
    ];
    for input in inputs {
        let cfg = Config::default();
        let detected = detect_words(input, &cfg);
        assert_eq!(
            count_words(input, &cfg),
            detected.count(),
            "count for {input:?}"
        );
        assert_eq!(
            split_words(input, &cfg),
            detected.words,
            "split for {input:?}"
        );
        assert_eq!(
            detected.count(),
            detected.words.len(),
            "count is len for {input:?}"
        );
    }
}

#[test]
fn default_config_is_all_off() {
    let cfg = Config::default();
    assert!(!cfg.punctuation_as_breaker);
    assert!(!cfg.disable_default_punctuation);
    assert!(cfg.punctuation.is_empty());
}

#[test]
fn config_is_constructible_with_struct_update() {
    let cfg = Config {
        punctuation_as_breaker: true,
        ..Default::default()
    };
    assert_eq!(count_words("Google's home", &cfg), 3);
}
