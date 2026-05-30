# fuzzengine

Fast fuzzy string matching in Rust, built on bit-parallel edit-distance
algorithms with a RapidFuzz/FuzzyWuzzy-style ratio API.

## Features

- **Bit-parallel edit distance** — Hyyrö 2003 global Levenshtein distance over a
  64-bit word.
- **Approximate search** — Myers 1999 bit-parallel alignment for finding the best
  matching substring (`partial_*` ratios).
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

## Limitations

The bit-parallel core uses a single `u64`, so inputs are compared over their
first 64 characters (after preprocessing). Longer strings are not yet handled by
a blocked/multi-word implementation.

## License

Licensed under the [MIT License](LICENSE).
