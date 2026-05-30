use crate::preprocess::PreprocessingOptions;
use std::collections::HashMap;

pub struct AlignmentResult {
    pos_start_t: usize,
    score: usize,
    len_sum: usize,
}

impl AlignmentResult {
    pub fn score(&self) -> usize {
        self.score
    }

    pub fn pos_start_t(&self) -> usize {
        self.pos_start_t
    }

    /// Combined character length of both strings after preprocessing.
    pub fn len_sum(&self) -> usize {
        self.len_sum
    }
}

pub fn get_all_alignments(
    str1_og: String,
    str2_og: String,
    max_score: usize,
    preprocessing_options: &PreprocessingOptions,
) -> Vec<AlignmentResult> {
    _get_all_alignments(str1_og, str2_og, max_score, false, preprocessing_options)
}

fn _get_all_alignments(
    str1_og: String,
    str2_og: String,
    max_score: usize,
    match_full: bool,
    preprocessing_options: &PreprocessingOptions,
) -> Vec<AlignmentResult> {
    // Myers 1999 algo
    let (str1, str2) = preprocessing_options.process(str1_og, str2_og);
    // setup
    let (p, t) = if str1.len() > str2.len() {
        (
            str2.chars().collect::<Vec<char>>(),
            str1.chars().collect::<Vec<char>>(),
        )
    } else {
        (
            str1.chars().collect::<Vec<char>>(),
            str2.chars().collect::<Vec<char>>(),
        )
    };
    let (m, n) = (p.len(), t.len());

    if m == n && m == 0 {
        return vec![AlignmentResult {
            pos_start_t: 0,
            score: 0,
            len_sum: 0,
        }];
    }

    if m == 0 || n == 0 {
        return vec![AlignmentResult {
            pos_start_t: 0,
            score: n,
            len_sum: m + n,
        }];
    }

    let mut peq: HashMap<char, u64> = HashMap::new();
    for ch in str1.chars().chain(str2.chars()) {
        peq.insert(ch, 0u64);
    }
    for i in 0..m {
        peq.insert(p[i], peq[&p[i]] | (1u64 << i));
    }
    let mut pv = u64::MAX;
    let mut mv = 0u64;
    let mut score = m;

    let mut alignments = Vec::new();

    // search
    for j in 0..n {
        let eq = peq[&t[j]];
        let xv = eq | mv;
        let xh = ((pv.wrapping_add(eq & pv)) ^ pv) | eq;
        let mut ph = mv | !(xh | pv);
        let mut mh = pv & xh;

        if ph & (1u64 << m - 1) != 0 {
            score += 1;
        } else if mh & (1u64 << m - 1) != 0 {
            score -= 1;
        }

        ph <<= 1;
        mh <<= 1;
        pv = mh | !(xv | ph);
        mv = ph & xv;

        if score <= max_score && (!match_full || (j == n - 1 && match_full)) {
            alignments.push(AlignmentResult {
                pos_start_t: (j + 1).saturating_sub(m),
                score,
                len_sum: m + n,
            });
        }
    }
    alignments
}

pub fn get_levenshtein_distance(
    str1_og: String,
    str2_og: String,

    preprocessing_options: &PreprocessingOptions,
) -> AlignmentResult {
    _get_all_alignments(str1_og, str2_og, usize::MAX, true, preprocessing_options)
        .into_iter()
        .next()
        .unwrap()
}

pub fn get_levenshtein_distance_partial(
    str1_og: String,
    str2_og: String,

    preprocessing_options: &PreprocessingOptions,
) -> AlignmentResult {
    let mut all_alignments =
        _get_all_alignments(str1_og, str2_og, usize::MAX, false, preprocessing_options);
    all_alignments.sort_by_key(|x| x.score);
    all_alignments.into_iter().next().unwrap()
}

pub fn get_best_alignment(
    str1_og: String,
    str2_og: String,
    preprocessing_options: &PreprocessingOptions,
) -> AlignmentResult {
    let mut alignments = get_all_alignments(str1_og, str2_og, usize::MAX, preprocessing_options);
    alignments.sort_by_key(|x| x.score);
    alignments.into_iter().next().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> PreprocessingOptions {
        PreprocessingOptions {
            force_ascii: true,
            strip: false,
        }
    }

    fn score(a: &str, b: &str) -> usize {
        get_best_alignment(a.to_string(), b.to_string(), &opts()).score
    }

    #[test]
    fn identical_strings_score_zero() {
        assert_eq!(score("hello", "hello"), 0);
    }

    #[test]
    fn both_empty_score_zero() {
        assert_eq!(score("", ""), 0);
    }

    #[test]
    fn empty_pattern_score_m() {
        // An empty pattern matches the empty substring anywhere: zero edits.
        assert_eq!(score("", "abcde"), 5);
    }

    #[test]
    fn exact_substring_score_zero() {
        // "cat" occurs verbatim inside "concatenate" -> no edits needed.
        assert_eq!(score("cat", "concatenate"), 0);
    }

    #[test]
    fn match_at_start_score_zero() {
        assert_eq!(score("con", "concatenate"), 0);
    }

    #[test]
    fn match_at_end_score_zero() {
        assert_eq!(score("nate", "concatenate"), 0);
    }

    #[test]
    fn single_substitution_score_one() {
        // "abcde" vs "abxde": one character differs.
        assert_eq!(score("abcde", "abxde"), 1);
    }

    #[test]
    fn single_deletion_score_one() {
        // Pattern "abd" matches "abcd" with one deletion (drop the 'c').
        assert_eq!(score("abd", "abcd"), 1);
    }

    #[test]
    fn single_insertion_score_one() {
        // Pattern "abc" matches "abxc" with one insertion ('x').
        assert_eq!(score("abc", "abxc"), 1);
    }

    #[test]
    fn two_substitutions_score_two() {
        // "abc" vs "xbx": first and last characters differ.
        assert_eq!(score("abc", "xbx"), 2);
    }

    #[test]
    fn all_characters_differ_score_is_length() {
        // No alignment is better than substituting every character.
        assert_eq!(score("abc", "xyz"), 3);
    }

    #[test]
    fn digits_are_matched() {
        // "a1c" occurs verbatim inside "xa1cy".
        assert_eq!(score("a1c", "xa1cy"), 0);
    }

    #[test]
    fn argument_order_is_symmetric() {
        // The function picks the shorter argument as the pattern, so swapping
        // the arguments must yield the same score.
        assert_eq!(
            get_best_alignment("cat".to_string(), "concatenate".to_string(), &opts()).score,
            get_best_alignment("concatenate".to_string(), "cat".to_string(), &opts()).score
        );
    }

    #[test]
    fn exact_substring_reports_start_position() {
        // "cat" starts at index 3 of "concatenate" (c-o-n-c-a-t...).
        let r = get_best_alignment("cat".to_string(), "concatenate".to_string(), &opts());
        assert_eq!(r.score, 0);
        assert_eq!(r.pos_start_t, 3);
    }

    #[test]
    fn match_at_start_reports_position_zero() {
        let r = get_best_alignment("con".to_string(), "concatenate".to_string(), &opts());
        assert_eq!(r.score, 0);
        assert_eq!(r.pos_start_t, 0);
    }
}
