use crate::hyyro;
use crate::myers;
use crate::preprocess::{PreprocessingOptions, token_set, token_sort, token_sort_set};

/// Similarity score in `0.0..=1.0` based on the full Levenshtein distance.
///
/// Computed as `1 - distance / (len(str1) + len(str2))`, where lengths are
/// measured after preprocessing. `1.0` means identical; two empty strings also
/// score `1.0`.
///
/// # Example
///
/// ```
/// use fuzzengine::{ratio, PreprocessingOptions};
///
/// let opts = PreprocessingOptions::default();
/// assert_eq!(ratio("hello", "hello", &opts), 1.0);
/// assert_eq!(ratio("hello", "hella", &opts), 0.9);
/// ```
pub fn ratio(str1: &str, str2: &str, preprocessing_options: &PreprocessingOptions) -> f64 {
    let (str1, str2) = preprocessing_options.process(str1.to_string(), str2.to_string());
    let len_sum = str1.chars().count() + str2.chars().count();
    if len_sum == 0 {
        return 1f64;
    }
    let distance = hyyro::get_edit_distance(str1, str2, preprocessing_options);
    1f64 - (distance as f64 / len_sum as f64)
}

/// Similarity score in `0.0..=1.0` based on the best *partial* (substring) match.
///
/// Like [`ratio`], but uses the lowest edit distance between the shorter string
/// and any aligned window of the longer one. This rewards one string occurring
/// inside the other — e.g. `partial_ratio("cat", "concatenate", ..)` is `1.0`.
///
/// # Example
///
/// ```
/// use fuzzengine::{partial_ratio, PreprocessingOptions};
///
/// let opts = PreprocessingOptions::default();
/// assert_eq!(partial_ratio("cat", "concatenate", &opts), 1.0);
/// ```
pub fn partial_ratio(str1: &str, str2: &str, preprocessing_options: &PreprocessingOptions) -> f64 {
    let result = myers::get_levenshtein_distance_partial(
        str1.to_string(),
        str2.to_string(),
        preprocessing_options,
    );
    if result.len_sum() == 0 {
        return 1f64;
    }
    1f64 - (result.score() as f64 / result.len_sum() as f64)
}

/// [`ratio`] applied after sorting each string's tokens, making the score
/// insensitive to word order.
///
/// # Example
///
/// ```
/// use fuzzengine::{token_sort_ratio, PreprocessingOptions};
///
/// let opts = PreprocessingOptions::default();
/// assert_eq!(token_sort_ratio("new york mets", "mets new york", &opts), 1.0);
/// ```
pub fn token_sort_ratio(
    str1: &str,
    str2: &str,
    preprocessing_options: &PreprocessingOptions,
) -> f64 {
    ratio(
        &token_sort(str1.to_string()),
        &token_sort(str2.to_string()),
        preprocessing_options,
    )
}

/// [`partial_ratio`] applied after sorting each string's tokens: order-insensitive
/// substring matching.
pub fn partial_token_sort_ratio(
    str1: &str,
    str2: &str,
    preprocessing_options: &PreprocessingOptions,
) -> f64 {
    partial_ratio(
        &token_sort(str1.to_string()),
        &token_sort(str2.to_string()),
        preprocessing_options,
    )
}

/// [`ratio`] applied after removing duplicate tokens from each string, making
/// the score insensitive to repeated words.
///
/// # Example
///
/// ```
/// use fuzzengine::{token_set_ratio, PreprocessingOptions};
///
/// let opts = PreprocessingOptions::default();
/// assert_eq!(
///     token_set_ratio("apple apple banana", "apple banana banana", &opts),
///     1.0
/// );
/// ```
pub fn token_set_ratio(
    str1: &str,
    str2: &str,
    preprocessing_options: &PreprocessingOptions,
) -> f64 {
    ratio(
        &token_set(str1.to_string()),
        &token_set(str2.to_string()),
        preprocessing_options,
    )
}

/// [`partial_ratio`] applied after removing duplicate tokens: duplicate-insensitive
/// substring matching.
pub fn partial_token_set_ratio(
    str1: &str,
    str2: &str,
    preprocessing_options: &PreprocessingOptions,
) -> f64 {
    partial_ratio(
        &token_set(str1.to_string()),
        &token_set(str2.to_string()),
        preprocessing_options,
    )
}

/// [`ratio`] applied after both sorting and de-duplicating each string's tokens:
/// insensitive to word order *and* repeats.
pub fn token_sort_set_ratio(
    str1: &str,
    str2: &str,
    preprocessing_options: &PreprocessingOptions,
) -> f64 {
    ratio(
        &token_sort_set(str1.to_string()),
        &token_sort_set(str2.to_string()),
        preprocessing_options,
    )
}

/// [`partial_ratio`] applied after both sorting and de-duplicating each string's
/// tokens: substring matching that ignores word order and repeats.
pub fn partial_token_sort_set_ratio(
    str1: &str,
    str2: &str,
    preprocessing_options: &PreprocessingOptions,
) -> f64 {
    partial_ratio(
        &token_sort_set(str1.to_string()),
        &token_sort_set(str2.to_string()),
        preprocessing_options,
    )
}

/// Return the candidate from `options` most similar to `str1`, together with its
/// [`ratio`] score.
///
/// Scores every option with [`ratio`] and returns the highest-scoring one.
/// Returns `None` if `options` is empty.
///
/// # Example
///
/// ```
/// use fuzzengine::{get_best_option, PreprocessingOptions};
///
/// let opts = PreprocessingOptions::default();
/// let best = get_best_option(
///     "cat",
///     vec!["dog".to_string(), "cat".to_string(), "bird".to_string()],
///     &opts,
/// );
/// assert_eq!(best, Some(("cat".to_string(), 1.0)));
/// ```
pub fn get_best_option(
    str1: &str,
    options: Vec<String>,
    preprocessing_options: &PreprocessingOptions,
) -> Option<(String, f64)> {
    if options.len() == 0 {
        return None;
    }
    let mut results: Vec<(String, f64)> = options
        .iter()
        .map(|x| (x.clone(), ratio(str1, x, preprocessing_options)))
        .collect();

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    Some((results[0].0.clone(), results[0].1))
}

/// Like [`get_best_option`], but scores candidates with the supplied `ratio_fn`
/// instead of [`ratio`], and returns only the winning string.
///
/// Pass any of the ratio functions (e.g. [`partial_ratio`] or [`token_sort_ratio`])
/// to control how similarity is measured. Returns `None` if `options` is empty.
///
/// # Example
///
/// ```
/// use fuzzengine::{get_best_option_with_ratio, partial_ratio, PreprocessingOptions};
///
/// let opts = PreprocessingOptions::default();
/// let best = get_best_option_with_ratio(
///     "cat",
///     vec!["dog".to_string(), "concatenate".to_string()],
///     &opts,
///     partial_ratio,
/// );
/// assert_eq!(best, Some("concatenate".to_string()));
/// ```
pub fn get_best_option_with_ratio(
    str1: &str,
    options: Vec<String>,
    preprocessing_options: &PreprocessingOptions,
    ratio_fn: fn(str1: &str, str2: &str, preprocessing_options: &PreprocessingOptions) -> f64,
) -> Option<String> {
    if options.len() == 0 {
        return None;
    }
    let mut results: Vec<(String, f64)> = options
        .iter()
        .map(|x| (x.clone(), ratio_fn(str1, x, preprocessing_options)))
        .collect();

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    Some(results[0].0.clone())
}
