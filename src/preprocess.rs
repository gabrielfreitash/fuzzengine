use any_ascii::any_ascii;

/// Controls the normalization applied to both inputs before they are compared.
///
/// Pass the same options to every comparison function. Use
/// [`PreprocessingOptions::default`] for the common case (fold non-ASCII and
/// trim).
///
/// # Example
///
/// ```
/// use fuzzengine::PreprocessingOptions;
///
/// let opts = PreprocessingOptions { force_ascii: true, strip: false };
/// let (a, b) = opts.process("Café".to_string(), "cafe".to_string());
/// assert_eq!(a, "Cafe");
/// assert_eq!(b, "cafe");
/// ```
pub struct PreprocessingOptions {
    /// When `true`, transliterate non-ASCII characters to their closest ASCII
    /// equivalent (e.g. `"é"` becomes `"e"`) via the `any_ascii` crate.
    pub force_ascii: bool,
    /// When `true`, trim leading and trailing whitespace from each input.
    pub strip: bool,
}

impl Default for PreprocessingOptions {
    /// Sensible defaults: fold non-ASCII characters and trim surrounding
    /// whitespace before comparing.
    fn default() -> Self {
        PreprocessingOptions {
            force_ascii: true,
            strip: true,
        }
    }
}

impl PreprocessingOptions {
    /// Apply the configured normalization to both strings and return the
    /// processed pair. The order of arguments is preserved.
    pub fn process(&self, str1: String, str2: String) -> (String, String) {
        let mut str1_preproc = if self.force_ascii {
            any_ascii(&str1)
        } else {
            str1
        };

        let mut str2_preproc = if self.force_ascii {
            any_ascii(&str2)
        } else {
            str2
        };

        if self.strip {
            str1_preproc = str1_preproc.trim().to_string();
            str2_preproc = str2_preproc.trim().to_string();
        }
        (str1_preproc, str2_preproc)
    }
}

/// Sort the space-separated tokens of `str1` alphabetically and rejoin them.
///
/// This makes comparisons insensitive to word order. For example,
/// `"new york"` and `"york new"` both become `"new york"`.
pub fn token_sort(str1: String) -> String {
    let mut splitted = str1.split(" ").collect::<Vec<&str>>();
    splitted.sort();
    splitted.join(" ")
}

/// Remove duplicate space-separated tokens from `str1`, preserving the order of
/// first appearance, and rejoin them.
///
/// For example, `"apple apple banana"` becomes `"apple banana"`.
pub fn token_set(str1: String) -> String {
    let mut seen = std::collections::HashSet::new();
    str1.split(" ")
        .filter(|token| seen.insert(*token))
        .collect::<Vec<&str>>()
        .join(" ")
}

/// Sort the space-separated tokens of `str1` alphabetically *and* drop adjacent
/// duplicates, then rejoin them.
///
/// Combines [`token_sort`] and de-duplication: `"c a b a"` becomes `"a b c"`.
pub fn token_sort_set(str1: String) -> String {
    let mut splitted = str1.split(" ").collect::<Vec<&str>>();
    splitted.sort();
    splitted.dedup();
    splitted.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proc(force_ascii: bool, strip: bool, a: &str, b: &str) -> (String, String) {
        PreprocessingOptions { force_ascii, strip }.process(a.to_string(), b.to_string())
    }

    #[test]
    fn default_folds_ascii_and_strips() {
        let opts = PreprocessingOptions::default();
        assert!(opts.force_ascii);
        assert!(opts.strip);
    }

    #[test]
    fn process_folds_non_ascii_when_enabled() {
        let (a, b) = proc(true, false, "café", "naïve");
        assert_eq!(a, "cafe");
        assert_eq!(b, "naive");
    }

    #[test]
    fn process_leaves_non_ascii_when_disabled() {
        let (a, _) = proc(false, false, "café", "x");
        assert_eq!(a, "café");
    }

    #[test]
    fn process_strips_whitespace_when_enabled() {
        let (a, b) = proc(false, true, "  hello \n", "\tworld  ");
        assert_eq!(a, "hello");
        assert_eq!(b, "world");
    }

    #[test]
    fn process_keeps_whitespace_when_disabled() {
        let (a, _) = proc(false, false, "  hello  ", "x");
        assert_eq!(a, "  hello  ");
    }

    #[test]
    fn process_folds_then_strips() {
        // ASCII folding happens first, then trimming.
        let (a, b) = proc(true, true, "  Café  ", " ÀÉÎ ");
        assert_eq!(a, "Cafe");
        assert_eq!(b, "AEI");
    }

    #[test]
    fn process_preserves_argument_order() {
        let (a, b) = proc(true, true, "first", "second");
        assert_eq!((a.as_str(), b.as_str()), ("first", "second"));
    }

    #[test]
    fn token_sort_orders_tokens() {
        assert_eq!(token_sort("york new".to_string()), "new york");
        assert_eq!(token_sort("new york".to_string()), "new york");
    }

    #[test]
    fn token_sort_is_lexicographic_ascii() {
        // Uppercase sorts before lowercase (ASCII code-point order).
        assert_eq!(token_sort("banana Apple".to_string()), "Apple banana");
    }

    #[test]
    fn token_sort_single_token_unchanged() {
        assert_eq!(token_sort("hello".to_string()), "hello");
    }

    #[test]
    fn token_set_removes_duplicates_preserving_order() {
        assert_eq!(token_set("apple apple banana".to_string()), "apple banana");
        // Order of first appearance is kept, not alphabetical.
        assert_eq!(token_set("banana apple banana".to_string()), "banana apple");
    }

    #[test]
    fn token_set_does_not_sort() {
        assert_eq!(token_set("c b a".to_string()), "c b a");
    }

    #[test]
    fn token_sort_set_sorts_and_dedupes() {
        assert_eq!(token_sort_set("c a b a".to_string()), "a b c");
        assert_eq!(token_sort_set("zebra apple zebra".to_string()), "apple zebra");
    }

    #[test]
    fn token_helpers_handle_single_token() {
        assert_eq!(token_set("solo".to_string()), "solo");
        assert_eq!(token_sort_set("solo".to_string()), "solo");
    }
}
