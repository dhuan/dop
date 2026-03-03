use std::process::Command;

pub const UNCHANGED_CONTENT: &str = "### MODIFY THIS FILE TO CHANGE A VALUE. ###";

pub fn exec(
    script: &str,
    env: &[(&str, &str)],
) -> Result<(bool, Option<String>, Option<String>), std::io::Error> {
    let mut cmd = Command::new("sh");

    for (key, value) in env {
        cmd.env(key, value);
    }

    let cmd_result = cmd.arg("-c").arg(script).output()?;

    Ok((
        cmd_result.status.success(),
        trim_empty_lines(&String::from_utf8(cmd_result.stdout).unwrap_or_default()),
        trim_empty_lines(&String::from_utf8(cmd_result.stderr).unwrap_or_default()),
    ))
}

pub fn mktemp() -> Result<String, String> {
    let out = Command::new("mktemp")
        .output()
        .map_err(|err| err.to_string())?;

    let path = String::from_utf8(out.stdout).map_err(|err| err.to_string())?;

    Ok(path.trim_end().to_owned())
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

pub fn trim_empty_lines(text: &str) -> Option<String> {
    let mut text_started = false;
    let mut lines = vec![];
    let mut last_line_with_lines = None;

    for line in text.lines() {
        let is_empty_line = line.trim() == "";

        if is_empty_line && !text_started {
            continue;
        }

        let len = lines.len();

        if !is_empty_line {
            last_line_with_lines = Some(if len == 0 { 0 } else { len - 1 });
        }

        if !text_started {
            text_started = true;
        }

        lines.push(line);
    }

    if lines.is_empty() {
        return None;
    }

    if lines.len() == 1 {
        return Some(lines.join("\n"));
    }

    let lines = if let Some(last_line_with_lines) = last_line_with_lines {
        lines[0..=last_line_with_lines + 1].join("\n")
    } else {
        lines.join("\n")
    };

    if lines.is_empty() { None } else { Some(lines) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_empty_lines() {
        assert_eq!(
            trim_empty_lines("\n\n\nhello\n\nworld\n\n\n\n\n"),
            Some("hello\n\nworld".to_string()),
        );

        assert_eq!(trim_empty_lines("\n\n\n"), None);

        assert_eq!(trim_empty_lines("ok"), Some("ok".to_string()));
    }
}
