use bva::{Bit, BitVector, Bvd};
use std::collections::HashMap;

use crate::preprocess::PreprocessingOptions;

pub fn get_edit_distance(
    str1_og: String,
    str2_og: String,
    preprocessing_options: &PreprocessingOptions,
) -> usize {
    // Hyyro's 2003 algo
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

    if m == 0 {
        return n;
    }

    if m > 64 {
        return _get_edit_distance_extended(str1, str2);
    }

    let mut peq: HashMap<char, u64> = HashMap::new();
    for ch in str1.chars().chain(str2.chars()) {
        peq.insert(ch, 0u64);
    }
    for i in 0..m {
        peq.insert(p[i], peq[&p[i]] | (1u64 << i));
    }
    let mut pv;
    let mut mv;

    let mut pv_last = u64::MAX;
    let mut mv_last = 0u64;

    // calc
    let mut dmj = m;
    for j in 0..n {
        let pmj = peq[&t[j]];
        let d0 = (((pmj & pv_last).wrapping_add(pv_last)) ^ pv_last) | pmj | mv_last;
        let hp = mv_last | !(d0 | pv_last);
        let mh = d0 & pv_last;

        if (hp & (1u64 << m - 1)) != 0 {
            dmj = dmj + 1
        }

        if (mh & (1u64 << m - 1)) != 0 {
            dmj = dmj - 1
        }

        let hp_shifted = (hp << 1) | 1;
        pv = (mh << 1) | !(d0 | hp_shifted);
        mv = d0 & hp_shifted;

        mv_last = mv;
        pv_last = pv;
    }
    dmj
}

fn _get_edit_distance_extended(str1_preprocessed: String, str2_preprocessed: String) -> usize {
    // Hyyro's 2003 algo
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

    if m == 0 {
        return n;
    }

    let mut peq: HashMap<char, Bvd> = HashMap::new();
    for ch in str1_preprocessed.chars().chain(str2_preprocessed.chars()) {
        peq.insert(ch, Bvd::zeros(n));
    }
    for i in 0..m {
        peq.get_mut(&p[i]).unwrap().set(i, Bit::One);
    }
    let mut pv;
    let mut mv;

    let mut pv_last = Bvd::ones(m);
    let mut mv_last = Bvd::zeros(m);

    // calc
    let mut dmj = m;
    for j in 0..n {
        let pmj = &peq[&t[j]];
        let d0 = (((pmj & &pv_last) + (&pv_last)) ^ &pv_last) | pmj | &mv_last;
        let hp = &mv_last | !(&d0 | &pv_last);
        let mh = &d0 & pv_last;

        if hp.get(m - 1) == Bit::One {
            dmj = dmj + 1
        }

        if mh.get(m - 1) == Bit::One {
            dmj = dmj - 1
        }

        let hp_shifted = (hp << 1u8) | 1u8;
        pv = (mh << 1u8) | !(&d0 | &hp_shifted);
        mv = &d0 & &hp_shifted;

        mv_last = mv;
        pv_last = pv;
    }
    dmj
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

    /// Edit distance via the function under test.
    fn dist(a: &str, b: &str) -> usize {
        get_edit_distance(a.to_string(), b.to_string(), &opts())
    }

    /// Straightforward O(m*n) dynamic-programming Levenshtein, used as an
    /// independent oracle to check the bit-parallel implementation against.
    fn dp_levenshtein(a: &str, b: &str) -> usize {
        let a: Vec<char> = a.chars().collect();
        let b: Vec<char> = b.chars().collect();
        let (m, n) = (a.len(), b.len());
        let mut prev: Vec<usize> = (0..=n).collect();
        let mut curr = vec![0usize; n + 1];
        for i in 1..=m {
            curr[0] = i;
            for j in 1..=n {
                let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
                curr[j] = (prev[j] + 1) // deletion
                    .min(curr[j - 1] + 1) // insertion
                    .min(prev[j - 1] + cost); // substitution/match
            }
            std::mem::swap(&mut prev, &mut curr);
        }
        prev[n]
    }

    #[test]
    fn both_empty_is_zero() {
        assert_eq!(dist("", ""), 0);
    }

    #[test]
    fn empty_pattern_is_length_of_other() {
        assert_eq!(dist("", "abcde"), 5);
        assert_eq!(dist("abcde", ""), 5);
    }

    #[test]
    fn identical_strings_are_zero() {
        assert_eq!(dist("hello", "hello"), 0);
        assert_eq!(dist("a", "a"), 0);
    }

    #[test]
    fn single_substitution() {
        assert_eq!(dist("abcde", "abxde"), 1);
    }

    #[test]
    fn single_insertion() {
        // "abc" -> "abxc": one inserted character.
        assert_eq!(dist("abc", "abxc"), 1);
    }

    #[test]
    fn single_deletion() {
        // "abcd" -> "abd": one deleted character.
        assert_eq!(dist("abcd", "abd"), 1);
    }

    #[test]
    fn all_characters_differ() {
        assert_eq!(dist("abc", "xyz"), 3);
    }

    #[test]
    fn classic_kitten_sitting() {
        // The textbook example: kitten -> sitting is 3 edits.
        assert_eq!(dist("kitten", "sitting"), 3);
    }

    #[test]
    fn classic_saturday_sunday() {
        assert_eq!(dist("saturday", "sunday"), 3);
    }

    #[test]
    fn flaw_lawn() {
        assert_eq!(dist("flaw", "lawn"), 2);
    }

    #[test]
    fn differing_lengths() {
        // "ab" needs 4 insertions to reach "abcdef".
        assert_eq!(dist("ab", "abcdef"), 4);
    }

    #[test]
    fn is_symmetric() {
        // Edit distance does not depend on argument order.
        let pairs = [
            ("kitten", "sitting"),
            ("flaw", "lawn"),
            ("abc", "xyz"),
            ("", "nonempty"),
            ("short", "a much longer string"),
        ];
        for (a, b) in pairs {
            assert_eq!(dist(a, b), dist(b, a), "asymmetry for ({a:?}, {b:?})");
        }
    }

    #[test]
    fn non_ascii_is_normalized() {
        // force_ascii folds "café" -> "cafe", so the distance is zero.
        assert_eq!(dist("cafe", "café"), 0);
        // "naïve" -> "naive" after folding.
        assert_eq!(dist("naive", "naïve"), 0);
    }

    #[test]
    fn matches_dp_oracle_on_many_pairs() {
        // Exhaustively compare against the DP oracle over a spread of words,
        // including empties, prefixes, anagrams, and repeated characters.
        let words = [
            "",
            "a",
            "ab",
            "abc",
            "abcd",
            "cat",
            "concatenate",
            "kitten",
            "sitting",
            "saturday",
            "sunday",
            "flaw",
            "lawn",
            "banana",
            "ananas",
            "aaaa",
            "aaab",
            "xyzzy",
            "levenshtein",
            "distance",
        ];
        for a in words {
            for b in words {
                assert_eq!(
                    dist(a, b),
                    dp_levenshtein(a, b),
                    "mismatch for ({a:?}, {b:?})"
                );
            }
        }
    }

    #[test]
    fn matches_dp_oracle_at_word_length_boundary() {
        // Strings whose length straddles the 64-bit word boundary still match
        // the oracle (kept within a single u64: <= 64 chars).
        let base: String = "abcdefghij".repeat(6); // 60 chars
        let mut mutated = base.clone();
        // Flip a handful of characters to introduce known edits.
        unsafe {
            let bytes = mutated.as_bytes_mut();
            bytes[0] = b'Z';
            bytes[30] = b'Z';
            bytes[59] = b'Z';
        }
        assert_eq!(dist(&base, &mutated), dp_levenshtein(&base, &mutated));
    }

    /// Assert the u64 fast path and the Bvd extended path return the same
    /// distance for the same input. Only meaningful when the pattern (shorter
    /// side) is <= 64 chars; otherwise `get_edit_distance` would itself delegate
    /// to the extended path and the comparison would be trivial.
    fn assert_paths_match(a: &str, b: &str) {
        assert!(
            a.chars().count().min(b.chars().count()) <= 64,
            "test input pattern must be <= 64 chars to exercise the u64 path"
        );
        let fast = get_edit_distance(a.to_string(), b.to_string(), &opts());
        // The extended fn expects already-preprocessed strings (no opts arg).
        let (pa, pb) = opts().process(a.to_string(), b.to_string());
        let ext = _get_edit_distance_extended(pa, pb);
        assert_eq!(fast, ext, "u64 vs extended mismatch for ({a:?}, {b:?})");
    }

    #[test]
    fn paths_match_u64_vs_extended() {
        // For patterns <= 64 chars the two implementations must agree exactly.
        let words = [
            "", "a", "ab", "abc", "abcd", "cat", "concatenate", "kitten", "sitting",
            "saturday", "sunday", "flaw", "lawn", "banana", "ananas", "aaaa", "aaab",
            "xyzzy", "levenshtein", "distance",
        ];
        for a in words {
            for b in words {
                assert_paths_match(a, b);
            }
        }
    }

    #[test]
    fn paths_match_at_64_boundary() {
        // A pattern of exactly 64 chars still runs the u64 path; the extended
        // path must agree with it.
        let boundary: String = "abcdefgh".repeat(8); // 64 chars
        let mut mutated = boundary.clone();
        unsafe {
            let bytes = mutated.as_bytes_mut();
            bytes[0] = b'Z';
            bytes[63] = b'Z';
        }
        assert_paths_match(&boundary, &boundary);
        assert_paths_match(&boundary, &mutated);
        assert_paths_match(&boundary, &format!("{boundary}tail"));
    }

    #[test]
    fn extended_matches_dp_oracle_for_long_patterns() {
        // Patterns > 64 chars route through the Bvd path. Compare against the
        // independent DP oracle across several lengths and edit patterns.
        for &len in &[65usize, 70, 80, 100, 150] {
            let a: String = "abcdefghijklmnopqrstuvwxyz".chars().cycle().take(len).collect();

            // identical
            assert_eq!(dist(&a, &a), 0, "identical len={len}");

            // a few substitutions to characters outside the alphabet
            let mut b = a.clone();
            unsafe {
                let by = b.as_bytes_mut();
                by[3] = b'0';
                by[len / 2] = b'1';
                by[len - 1] = b'2';
            }
            assert_eq!(dist(&a, &b), dp_levenshtein(&a, &b), "substitutions len={len}");

            // insertion (longer counterpart)
            let c = format!("{a}tail");
            assert_eq!(dist(&a, &c), dp_levenshtein(&a, &c), "insertion len={len}");

            // deletion (drop a middle chunk)
            let d: String = a
                .chars()
                .enumerate()
                .filter(|(i, _)| !(len / 3..len / 3 + 5).contains(i))
                .map(|(_, ch)| ch)
                .collect();
            assert_eq!(dist(&a, &d), dp_levenshtein(&a, &d), "deletion len={len}");
        }
    }
}
