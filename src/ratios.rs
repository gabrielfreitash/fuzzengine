use crate::hyyro;
use crate::myers;
use crate::preprocess::{PreprocessingOptions, token_set, token_sort, token_sort_set};

pub fn ratio(str1: &str, str2: &str, preprocessing_options: &PreprocessingOptions) -> f64 {
    let (str1, str2) = preprocessing_options.process(str1.to_string(), str2.to_string());
    let len_sum = str1.chars().count() + str2.chars().count();
    if len_sum == 0 {
        return 1f64;
    }
    let distance = hyyro::get_edit_distance(str1, str2, preprocessing_options);
    1f64 - (distance as f64 / len_sum as f64)
}

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
