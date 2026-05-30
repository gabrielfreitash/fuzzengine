use any_ascii::any_ascii;
pub struct PreprocessingOptions {
    pub force_ascii: bool,
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

pub fn token_sort(str1: String) -> String {
    let mut splitted = str1.split(" ").collect::<Vec<&str>>();
    splitted.sort();
    splitted.join(" ")
}

pub fn token_set(str1: String) -> String {
    let mut seen = std::collections::HashSet::new();
    str1.split(" ")
        .filter(|token| seen.insert(*token))
        .collect::<Vec<&str>>()
        .join(" ")
}

pub fn token_sort_set(str1: String) -> String {
    let mut splitted = str1.split(" ").collect::<Vec<&str>>();
    splitted.sort();
    splitted.dedup();
    splitted.join(" ")
}
