pub mod hyyro;
pub mod myers;
pub mod preprocess;
pub mod ratios;

pub use hyyro::get_edit_distance;
pub use preprocess::PreprocessingOptions;
pub use ratios::{
    get_best_option, get_best_option_with_ratio, partial_ratio, partial_token_set_ratio,
    partial_token_sort_ratio, partial_token_sort_set_ratio, ratio, token_set_ratio,
    token_sort_ratio, token_sort_set_ratio,
};

#[cfg(test)]
mod tests {
    use crate::hyyro::get_edit_distance;
    use crate::myers::{get_all_alignments, get_best_alignment};
    use crate::preprocess::PreprocessingOptions;
    use crate::ratios::{
        get_best_option, get_best_option_with_ratio, partial_ratio, partial_token_set_ratio,
        partial_token_sort_ratio, partial_token_sort_set_ratio, ratio, token_set_ratio,
        token_sort_ratio, token_sort_set_ratio,
    };

    fn opts() -> PreprocessingOptions {
        PreprocessingOptions {
            force_ascii: true,
            strip: false,
        }
    }

    #[test]
    fn best_alignment_exact_substring() {
        let r = get_best_alignment("cat".to_string(), "concatenate".to_string(), &opts());
        assert_eq!(r.score(), 0);
        assert_eq!(r.pos_start_t(), 3);
    }

    #[test]
    fn best_alignment_normalizes_non_ascii() {
        // "café" normalizes to "cafe", so the pattern matches exactly.
        let r = get_best_alignment("cafe".to_string(), "a café shop".to_string(), &opts());
        assert_eq!(r.score(), 0);
    }

    #[test]
    fn all_alignments_respect_max_score() {
        let results = get_all_alignments("ab".to_string(), "zzabzz".to_string(), 0, &opts());
        assert!(!results.is_empty());
        assert!(results.iter().all(|r| r.score() == 0));
    }

    #[test]
    fn all_alignments_filters_above_threshold() {
        // No substring of "xyz" is within edit distance 0 of "ab".
        let results = get_all_alignments("ab".to_string(), "xyz".to_string(), 0, &opts());
        assert!(results.is_empty());
    }

    #[test]
    fn edit_distance_kitten_sitting() {
        // The textbook Levenshtein example: kitten -> sitting is 3 edits.
        assert_eq!(
            get_edit_distance("kitten".to_string(), "sitting".to_string(), &opts()),
            3
        );
    }

    #[test]
    fn edit_distance_empty_and_identical() {
        // Empty vs empty has no edits; identical strings have no edits.
        assert_eq!(
            get_edit_distance("".to_string(), "".to_string(), &opts()),
            0
        );
        assert_eq!(
            get_edit_distance("hello".to_string(), "hello".to_string(), &opts()),
            0
        );
        // Empty pattern needs one insertion per character of the other string.
        assert_eq!(
            get_edit_distance("".to_string(), "abcde".to_string(), &opts()),
            5
        );
    }

    #[test]
    fn edit_distance_normalizes_non_ascii() {
        // force_ascii folds "café" -> "cafe", so there are no edits.
        assert_eq!(
            get_edit_distance("cafe".to_string(), "café".to_string(), &opts()),
            0
        );
    }

    #[test]
    fn ratio_identical_is_one() {
        assert_eq!(ratio("hello", "hello", &opts()), 1.0);
    }

    #[test]
    fn ratio_all_different_is_half() {
        // 3 substitutions over a combined length of 6: 1 - 3/6 = 0.5.
        assert_eq!(ratio("abc", "xyz", &opts()), 0.5);
    }

    #[test]
    fn partial_ratio_substring_is_one() {
        // "cat" occurs verbatim in "concatenate": the best partial alignment
        // has zero edits, so the partial ratio is 1.0.
        assert_eq!(partial_ratio("cat", "concatenate", &opts()), 1.0);
    }

    #[test]
    fn token_sort_ratio_ignores_word_order() {
        // Sorting the tokens makes both strings identical.
        assert_eq!(
            token_sort_ratio("new york mets", "mets new york", &opts()),
            1.0
        );
    }

    #[test]
    fn partial_token_sort_ratio_ignores_word_order() {
        assert_eq!(
            partial_token_sort_ratio("new york mets", "mets new york", &opts()),
            1.0
        );
    }

    #[test]
    fn token_set_ratio_ignores_duplicates() {
        // De-duplicating tokens leaves both strings equal to "apple banana".
        assert_eq!(
            token_set_ratio("apple apple banana", "apple banana banana", &opts()),
            1.0
        );
    }

    #[test]
    fn partial_token_set_ratio_ignores_duplicates() {
        assert_eq!(
            partial_token_set_ratio("apple apple", "apple", &opts()),
            1.0
        );
    }

    #[test]
    fn token_sort_set_ratio_ignores_order_and_duplicates() {
        // Sort + de-duplicate collapses both inputs to "a b c".
        assert_eq!(token_sort_set_ratio("c a b a", "a b c", &opts()), 1.0);
    }

    #[test]
    fn partial_token_sort_set_ratio_ignores_order_and_duplicates() {
        assert_eq!(
            partial_token_sort_set_ratio("c a b a", "a b c", &opts()),
            1.0
        );
    }

    fn owned(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn get_best_option_picks_exact_match() {
        // An exact match scores 1.0, beating every other candidate.
        let best = get_best_option("cat", owned(&["dog", "cat", "bird"]), &opts());
        assert_eq!(best, Some(("cat".to_string(), 1.0)));
    }

    #[test]
    fn get_best_option_picks_closest_when_no_exact_match() {
        // "hella" is one substitution away (ratio 0.9); "help" is further.
        let best = get_best_option("hello", owned(&["world", "help", "hella"]), &opts());
        let (word, score) = best.unwrap();
        assert_eq!(word, "hella");
        assert_eq!(score, 0.9);
    }

    #[test]
    fn get_best_option_empty_is_none() {
        assert_eq!(get_best_option("cat", owned(&[]), &opts()), None);
    }

    #[test]
    fn get_best_option_with_ratio_uses_plain_ratio() {
        let best =
            get_best_option_with_ratio("cat", owned(&["dog", "cat", "bird"]), &opts(), ratio);
        assert_eq!(best, Some("cat".to_string()));
    }

    #[test]
    fn get_best_option_with_ratio_respects_the_chosen_function() {
        // With partial_ratio, "cat" is a perfect substring of "concatenate",
        // so it wins — whereas plain `ratio` would score that pair poorly.
        let options = owned(&["dog", "concatenate"]);
        let partial_pick =
            get_best_option_with_ratio("cat", options.clone(), &opts(), partial_ratio);
        assert_eq!(partial_pick, Some("concatenate".to_string()));

        // Sanity check that the function pointer actually changes the outcome:
        // under plain `ratio`, "concatenate" is a much worse match.
        assert!(ratio("cat", "concatenate", &opts()) < 0.5);
    }

    #[test]
    fn get_best_option_with_ratio_token_sort() {
        // token_sort_ratio ignores word order, so the reordered phrase wins.
        let best = get_best_option_with_ratio(
            "new york",
            owned(&["los angeles", "york new"]),
            &opts(),
            token_sort_ratio,
        );
        assert_eq!(best, Some("york new".to_string()));
    }

    #[test]
    fn get_best_option_with_ratio_empty_is_none() {
        assert_eq!(
            get_best_option_with_ratio("cat", owned(&[]), &opts(), ratio),
            None
        );
    }
}
