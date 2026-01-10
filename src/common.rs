use std::process::Command;

pub const UNCHANGED_CONTENT: &str = "### MODIFY THIS FILE TO CHANGE A VALUE. ###";

pub fn exec(script: &str, env: &[(&str, &str)]) -> Result<bool, std::io::Error> {
    let mut cmd = Command::new("sh");

    for (key, value) in env {
        cmd.env(key, value);
    }

    let cmd_result = cmd.arg("-c").arg(script).output()?;

    Ok(cmd_result.status.success())
}

pub fn mktemp() -> Result<String, String> {
    let out = Command::new("mktemp")
        .output()
        .map_err(|err| err.to_string())?;

    let path = String::from_utf8(out.stdout).map_err(|err| err.to_string())?;

    Ok(path.trim_end().to_owned())
}

pub fn trim_new_line(s: &str) -> &str {
    s.strip_suffix("\r\n").or(s.strip_suffix("\n")).unwrap_or(s)
}

pub fn file_has_been_modified(file_path: &str) -> Result<bool, std::io::Error> {
    Ok(std::fs::read_to_string(file_path)? != UNCHANGED_CONTENT)
}

pub fn unquote(s: &str) -> &str {
    let s = s.strip_prefix(r#"""#).unwrap_or(s);

    s.strip_suffix(r#"""#).unwrap_or(s)
}

pub fn regex_test(regex_str: &str, subject: &str) -> bool {
    match regex::Regex::new(regex_str) {
        Err(_) => false,
        Ok(re) => re.is_match(subject),
    }
}
