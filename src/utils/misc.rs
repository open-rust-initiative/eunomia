pub mod regex_utils {
    use regex::Regex;

    /// Return a named match from capture.
    ///
    /// # Example
    ///
    /// ```rust
    /// use regex::Regex;
    /// use eunomia::utils::regex_utils;
    ///
    /// let re = Regex::new(r"clippy::(?P<name>(\w+))").unwrap();
    /// let line = "#![allow(clippy::comparison_to_empty)]";
    /// let res = regex_utils::get_named_match("name", &re, line);
    ///
    /// assert_eq!(res, Some("comparison_to_empty".to_string()))
    /// ```
    pub fn get_named_match(name: &'static str, re: &Regex, s: &str) -> Option<String> {
        re.captures_iter(s).next().map(|cap| cap[name].to_string())
    }

    /// Return the most accurate match in the captures group
    pub fn capture_last<'a>(re: &Regex, s: &'a str) -> Option<&'a str> {
        let captures = re.captures(s)?;

        for idx in (0..captures.len()).rev() {
            if let Some(cap) = captures.get(idx) {
                return Some(cap.as_str());
            }
        }

        None
    }
}
