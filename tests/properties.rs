//! Invariants that must hold for every input. Inputs come from a small
//! deterministic generator so the suite stays std-only and reproducible. Each
//! property runs over many random strings drawn from the BMP, which matches the
//! code-unit behavior the heuristic targets.

use count_words::{count_words, detect_words, split_words, Config};

/// Tiny xorshift generator. Deterministic so failures reproduce.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

/// Pull of characters: ASCII letters, ASCII digits, spaces, default punctuation,
/// and a few BMP CJK and Cyrillic letters. No astral characters.
const POOL: &[char] = &[
    'a', 'b', 'c', 'z', 'A', 'Q', '0', '1', '9', ' ', '\t', '\n', ',', '.', ':', '\'', '-', '你',
    '好', '世', 'カ', 'д', 'ع',
];

fn random_string(rng: &mut Rng, max_len: usize) -> String {
    let len = rng.below(max_len + 1);
    (0..len).map(|_| POOL[rng.below(POOL.len())]).collect()
}

fn configs() -> Vec<Config> {
    vec![
        Config::default(),
        Config {
            punctuation_as_breaker: true,
            ..Default::default()
        },
        Config {
            disable_default_punctuation: true,
            ..Default::default()
        },
        Config {
            punctuation: vec!["-".into()],
            ..Default::default()
        },
    ]
}

#[test]
fn count_equals_split_len() {
    let mut rng = Rng(0x1234_5678_9abc_def0);
    for _ in 0..2000 {
        let s = random_string(&mut rng, 24);
        for cfg in configs() {
            assert_eq!(
                count_words(&s, &cfg),
                split_words(&s, &cfg).len(),
                "count != split.len() for {s:?}"
            );
        }
    }
}

#[test]
fn three_functions_consistent() {
    let mut rng = Rng(0xdead_beef_cafe_babe);
    for _ in 0..2000 {
        let s = random_string(&mut rng, 24);
        for cfg in configs() {
            let d = detect_words(&s, &cfg);
            assert_eq!(count_words(&s, &cfg), d.count());
            assert_eq!(split_words(&s, &cfg), d.words);
            assert_eq!(d.count(), d.words.len());
        }
    }
}

#[test]
fn ascii_letters_split_on_whitespace() {
    let mut rng = Rng(0x0f0f_0f0f_0f0f_0f0f);
    let letters: &[char] = &['a', 'b', 'c', 'X', 'Y', 'Z'];
    for _ in 0..1000 {
        // Build a string of letter runs joined by single spaces.
        let runs = 1 + rng.below(6);
        let mut parts: Vec<String> = Vec::new();
        for _ in 0..runs {
            let run_len = 1 + rng.below(5);
            let run: String = (0..run_len)
                .map(|_| letters[rng.below(letters.len())])
                .collect();
            parts.push(run);
        }
        let s = parts.join(" ");
        assert_eq!(
            count_words(&s, &Config::default()),
            parts.len(),
            "{s:?} should split into {} runs",
            parts.len()
        );
    }
}

#[test]
fn each_han_char_is_one_word() {
    let mut rng = Rng(0xa5a5_5a5a_a5a5_5a5a);
    let han: &[char] = &['你', '好', '世', '界', '勤', '勉', '中', '文'];
    for _ in 0..1000 {
        let n = 1 + rng.below(10);
        let s: String = (0..n).map(|_| han[rng.below(han.len())]).collect();
        assert_eq!(
            count_words(&s, &Config::default()),
            n,
            "{s:?} has {n} Han chars"
        );
    }
}

#[test]
fn appending_punctuation_never_raises_default_count() {
    // Under default config, punctuation is removed, so adding a default mark to
    // any string cannot create a new word. It can only join runs or vanish.
    let mut rng = Rng(0x1111_2222_3333_4444);
    let default = Config::default();
    let marks: &[char] = &[',', '.', ':', '\'', '"'];
    for _ in 0..2000 {
        let s = random_string(&mut rng, 24);
        let mark = marks[rng.below(marks.len())];
        let with_mark = format!("{s}{mark}");
        assert!(
            count_words(&with_mark, &default) <= count_words(&s, &default),
            "appending {mark:?} raised the count for {s:?}"
        );
    }
}
