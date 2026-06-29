# count-words

Count words in text that mixes scripts: Chinese, Japanese, Korean, Latin,
Cyrillic, Malayalam, numbers, and punctuation. One heuristic handles every
language.

## Rules

- A run of Latin, Cyrillic, or Malayalam letters between separators is one word.
- A run of ASCII digits is one word.
- Each CJK ideograph, Japanese kana, and Korean Hangul character is its own word.
- A character outside the known blocks falls through to a whole-token fallback.
  This counts Arabic, Hebrew, Thai, and most Indic text by whitespace splitting.

```rust
use count_words::words_count;

assert_eq!(words_count("Hello World", &Default::default()), 2);
assert_eq!(words_count("你好，世界", &Default::default()), 4);
assert_eq!(words_count("Hello \"世界\"", &Default::default()), 3);
```

## Installation

```toml
[dependencies]
count-words = "0.1"
```

## API

Three functions share one core.

```rust
use count_words::{words_count, words_split, words_detect, Config};

let cfg = Config::default();

// Just the count.
let n: usize = words_count("Hello, 你好。", &cfg);
assert_eq!(n, 3);

// Just the word list.
let words: Vec<String> = words_split("100世界", &cfg);
assert_eq!(words, vec!["100", "世", "界"]);

// Both at once.
let result = words_detect("Hello, 世界 100", &cfg);
assert_eq!(result.count, 4);
assert_eq!(result.words, vec!["Hello", "世", "界", "100"]);
```

`count` always equals `words.len()`.

## Config

`Config` changes how punctuation is handled. All fields default to off.

```rust
use count_words::{words_count, Config};

// Replace punctuation with a space instead of deleting it.
let breaker = Config { punctuation_as_breaker: true, ..Default::default() };
assert_eq!(words_count("Google's home", &breaker), 3);

// Skip the built-in punctuation list.
let bare = Config { disable_default_punctuation: true, ..Default::default() };

// Add custom marks. Each entry is a literal substring.
let dash = Config { punctuation: vec!["-".into()], ..Default::default() };
```

- `punctuation_as_breaker`: when on, each punctuation mark becomes a space and can
  split a token in two. When off, marks are deleted and the surrounding text joins.
- `disable_default_punctuation`: when on, the built-in marks are left alone and
  only the custom list is handled.
- `punctuation`: extra marks, applied after the built-in list, in order.

## Behavior notes

- Empty input and whitespace-only input return zero words.
- General Punctuation (U+2000-U+206F) and Halfwidth/Fullwidth Forms
  (U+FF00-U+FFEF) are always stripped. This removes smart quotes, dashes,
  ellipses, fullwidth commas, and fullwidth digits.
- Classification uses Basic Multilingual Plane ranges. An astral ideograph or
  emoji is never counted as a CJK word. It survives as a whole token when alone
  and drops out when mixed with covered characters.

## License

Licensed under the [MIT license](LICENSE).
