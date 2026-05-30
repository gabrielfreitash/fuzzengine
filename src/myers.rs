use crate::preprocess::PreprocessingOptions;
use bva::{Bit, BitVector, Bvd};
use std::collections::HashMap;

/// The outcome of aligning a pattern against a text: the edit cost and where in
/// the text the best alignment begins.
pub struct AlignmentResult {
    pos_start_t: usize,
    score: usize,
    len_sum: usize,
}

impl AlignmentResult {
    /// Edit distance of this alignment (lower is a closer match).
    pub fn score(&self) -> usize {
        self.score
    }

    /// Character index in the (longer) text where the matched region begins.
    pub fn pos_start_t(&self) -> usize {
        self.pos_start_t
    }

    /// Combined character length of both strings after preprocessing.
    pub fn len_sum(&self) -> usize {
        self.len_sum
    }
}

/// Find every alignment of the shorter string against the longer one whose edit
/// distance is at most `max_score`.
///
/// Each text position that ends a sufficiently-close match of the pattern yields
/// one [`AlignmentResult`]. Pass `max_score == usize::MAX` to keep them all. The
/// returned vector is in ascending order of text position, not score.
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

    // The fast path packs the pattern into a single u64, so it only handles
    // patterns up to 64 characters. Longer patterns use the Bvd-backed version.
    if m > 64 {
        return _get_all_alignments_extended(str1, str2, max_score, match_full);
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

fn _get_all_alignments_extended(
    str1_preprocessed: String,
    str2_preprocessed: String,
    max_score: usize,
    match_full: bool,
) -> Vec<AlignmentResult> {
    // Myers 1999 algo - this version supports t > usize in length
    // setup
    let (p, t) = if str1_preprocessed.len() > str2_preprocessed.len() {
        (
            str2_preprocessed.chars().collect::<Vec<char>>(),
            str1_preprocessed.chars().collect::<Vec<char>>(),
        )
    } else {
        (
            str1_preprocessed.chars().collect::<Vec<char>>(),
            str2_preprocessed.chars().collect::<Vec<char>>(),
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

    let mut peq: HashMap<char, Bvd> = HashMap::new();
    for ch in str1_preprocessed.chars().chain(str2_preprocessed.chars()) {
        peq.insert(ch, Bvd::zeros(m));
    }
    for i in 0..m {
        peq.get_mut(&p[i]).unwrap().set(i, Bit::One);
    }
    let mut pv = Bvd::ones(m);
    let mut mv = Bvd::zeros(m);
    let mut score = m;

    let mut alignments = Vec::new();

    // search
    for j in 0..n {
        let eq = &peq[&t[j]];
        let xv = eq | &mv;
        let eq_and_pv = eq & &pv;
        let sum = &pv + &eq_and_pv;
        let xh = &(&sum ^ &pv) | eq;
        let mut ph = &mv | &!(&xh | &pv);
        let mut mh = &pv & &xh;

        if ph.get(m - 1) == Bit::One {
            score += 1;
        } else if mh.get(m - 1) == Bit::One {
            score -= 1;
        }

        ph <<= 1u8;
        mh <<= 1u8;
        pv = &mh | &!(&xv | &ph);
        mv = &ph & &xv;

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

/// Compute the full (end-to-end) Levenshtein distance between the two strings.
///
/// Unlike [`get_levenshtein_distance_partial`], the whole pattern is aligned
/// against the whole text. The distance is available via
/// [`AlignmentResult::score`].
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

/// Find the best partial (substring) match: the lowest edit distance between the
/// shorter string and any aligned window of the longer one.
///
/// This is the basis of the `partial_*` ratios — it rewards the pattern occurring
/// inside the text without penalizing the surrounding characters.
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

/// Return the single best alignment of the shorter string within the longer one
/// — the lowest-scoring [`AlignmentResult`], including where the match starts.
///
/// # Example
///
/// ```
/// use fuzzengine::{myers::get_best_alignment, PreprocessingOptions};
///
/// let opts = PreprocessingOptions::default();
/// let r = get_best_alignment("cat".to_string(), "concatenate".to_string(), &opts);
/// assert_eq!(r.score(), 0); // exact substring
/// assert_eq!(r.pos_start_t(), 3); // "cat" starts at index 3
/// ```
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

    #[test]
    fn long_identical_pattern_score_zero() {
        // A 100-char pattern (> 64) routes through the Bvd-backed path.
        let s: String = "abcdefghij".repeat(10); // 100 chars
        assert_eq!(score(&s, &s), 0);
    }

    #[test]
    fn long_pattern_exact_substring_score_zero() {
        // 70-char pattern (> 64) found verbatim inside a longer text.
        let pattern: String = "abcdefghij".repeat(7); // 70 chars
        let text = format!("xyz{pattern}xyz");
        assert_eq!(score(&pattern, &text), 0);
    }

    /// Assert the u64 fast path and the Bvd extended path produce identical
    /// alignment vectors for the same input. Only meaningful when the pattern
    /// (shorter side) is <= 64 chars, otherwise `_get_all_alignments` would
    /// itself delegate to the extended path and the comparison would be trivial.
    fn assert_paths_match(a: &str, b: &str, max_score: usize, match_full: bool) {
        assert!(
            a.chars().count().min(b.chars().count()) <= 64,
            "test input pattern must be <= 64 chars to exercise the u64 path"
        );
        let fast =
            _get_all_alignments(a.to_string(), b.to_string(), max_score, match_full, &opts());
        // The extended fn expects already-preprocessed strings (no opts arg).
        let (pa, pb) = opts().process(a.to_string(), b.to_string());
        let ext = _get_all_alignments_extended(pa, pb, max_score, match_full);

        assert_eq!(
            fast.len(),
            ext.len(),
            "alignment count differs for ({a:?}, {b:?}) max_score={max_score} match_full={match_full}"
        );
        for (i, (f, e)) in fast.iter().zip(ext.iter()).enumerate() {
            assert_eq!(
                (f.score, f.pos_start_t, f.len_sum),
                (e.score, e.pos_start_t, e.len_sum),
                "alignment #{i} differs for ({a:?}, {b:?}) max_score={max_score} match_full={match_full}"
            );
        }
    }

    /// A spread of inputs covering identical strings, substrings, edits at every
    /// position, repeats, digits, the empty cases, and a 64-char boundary pattern.
    fn parity_cases() -> Vec<(String, String)> {
        let boundary: String = "abcdefgh".repeat(8); // exactly 64 chars (u64 path)
        vec![
            ("hello".into(), "hello".into()),
            ("".into(), "".into()),
            ("".into(), "abcde".into()),
            ("abcde".into(), "".into()),
            ("cat".into(), "concatenate".into()),
            ("con".into(), "concatenate".into()),
            ("nate".into(), "concatenate".into()),
            ("abcde".into(), "abxde".into()),
            ("abd".into(), "abcd".into()),
            ("abc".into(), "abxc".into()),
            ("abc".into(), "xbx".into()),
            ("abc".into(), "xyz".into()),
            ("a1c".into(), "xa1cy".into()),
            ("banana".into(), "ananas".into()),
            ("aaaa".into(), "aabaa".into()),
            ("kitten".into(), "sitting".into()),
            ("levenshtein".into(), "distance".into()),
            (boundary.clone(), format!("zz{boundary}zz")),
            (boundary.clone(), boundary),
        ]
    }

    #[test]
    fn paths_match_default() {
        for (a, b) in parity_cases() {
            assert_paths_match(&a, &b, usize::MAX, false);
        }
    }

    #[test]
    fn paths_match_match_full() {
        // match_full=true keeps only the alignment of the whole text (Levenshtein).
        for (a, b) in parity_cases() {
            assert_paths_match(&a, &b, usize::MAX, true);
        }
    }

    #[test]
    fn paths_match_under_max_score() {
        // Exercise the `score <= max_score` filter identically on both paths.
        for max_score in [0usize, 1, 2, 3] {
            for (a, b) in parity_cases() {
                assert_paths_match(&a, &b, max_score, false);
            }
        }
    }

    #[test]
    fn paths_match_argument_order_swapped() {
        // The internal shorter/longer swap must land both paths in the same place.
        for (a, b) in parity_cases() {
            assert_paths_match(&b, &a, usize::MAX, false);
        }
    }
}
