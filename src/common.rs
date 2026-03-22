pub fn regex_test(regex_str: &str, subject: &str) -> bool {
    match regex::Regex::new(regex_str) {
        Err(_) => false,
        Ok(re) => re.is_match(subject),
    }
}
