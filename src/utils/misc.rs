pub mod regex_utils {
    use regex::Regex;

    pub fn get_named_match(name: &'static str, re: &Regex, s: &str) -> Option<String> {
        re.captures_iter(s).next().map(|cap| cap[name].to_string())
    }
}
