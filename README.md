# fuzzengine

Fast fuzzy string matching in Rust, built on bit-parallel edit-distance
algorithms with a RapidFuzz/FuzzyWuzzy-style ratio API.

## Authorship

The library implementation — the algorithms and the public API — is **written by
hand and is not AI-generated**. The test suite and the documentation (this
README and the in-source doc comments) were produced with AI assistance.

## Features

- **Bit-parallel edit distance** — Hyyrö 2003 global Levenshtein distance.
- **Approximate search** — Myers 1999 bit-parallel alignment for finding the best
  matching substring (`partial_*` ratios).
- **No length limit** — a `u64` fast path for patterns up to 64 characters, with
  an automatic heap-backed fallback (via [`bva`]) for longer ones.
- **Ratios** — `ratio`, `partial_ratio`, and the `token_sort` / `token_set` /
  `token_sort_set` variants, each returning a similarity score in `0.0..=1.0`.
- **Preprocessing** — optional non-ASCII folding (via [`any_ascii`]) and
  whitespace stripping.
- **Best-match helpers** — `get_best_option` and `get_best_option_with_ratio`
  pick the closest candidate from a list.

[`any_ascii`]: https://crates.io/crates/any_ascii

## Usage

```rust
use fuzzengine::{ratio, get_best_option, PreprocessingOptions};

let opts = PreprocessingOptions::default(); // fold non-ASCII + trim

// Similarity score between two strings (0.0 ..= 1.0)
let score = ratio("hello", "hella", &opts);
assert_eq!(score, 0.9);

// Pick the closest candidate from a list
let best = get_best_option(
    "new york",
    vec!["york new".to_string(), "los angeles".to_string()],
    &opts,
);
assert_eq!(best, Some(("york new".to_string(), 0.5)));
```

### Edit distance directly

```rust
use fuzzengine::{get_edit_distance, PreprocessingOptions};

let opts = PreprocessingOptions::default();
let edits = get_edit_distance("kitten".to_string(), "sitting".to_string(), &opts);
assert_eq!(edits, 3);
```

### Order- and duplicate-insensitive matching

```rust
use fuzzengine::{token_sort_ratio, token_set_ratio, PreprocessingOptions};

let opts = PreprocessingOptions::default();
assert_eq!(token_sort_ratio("new york mets", "mets new york", &opts), 1.0);
assert_eq!(token_set_ratio("apple apple banana", "apple banana banana", &opts), 1.0);
```

## Strings of any length

The bit-parallel core packs the pattern (the shorter of the two inputs) into a
single `u64`, which keeps the common case fast. When the pattern exceeds 64
characters, the algorithms transparently fall back to a heap-allocated bit
vector (via [`bva`]), so there is **no length limit** — only a modest speed
difference between the two paths. Callers don't need to do anything; the
dispatch is automatic.

[`bva`]: https://crates.io/crates/bva

## Testing

The crate has **over 98% line coverage**. Beyond example-based unit tests, the
suite cross-checks the bit-parallel algorithms against an independent
dynamic-programming oracle, asserts that the `u64` fast path and the long-string
fallback produce identical results, and runs every documentation example as a
doctest.

Run the tests with:

```sh
cargo test
```

## License

Licensed under the [MIT License](LICENSE).
