use std::process::Command;

pub fn mktemp() -> Result<String, String> {
    let out = Command::new("mktemp")
        .output()
        .map_err(|err| err.to_string())?;

    let path = String::from_utf8(out.stdout).map_err(|err| err.to_string())?;

    Ok(path.trim_end().to_owned())
}

pub fn regex_test(regex_str: &str, subject: &str) -> bool {
    match regex::Regex::new(regex_str) {
        Err(_) => false,
        Ok(re) => re.is_match(subject),
    }
}
