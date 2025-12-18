use std::process::Command;

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

pub fn file_has_been_modified(
    file_path: &str,
    time: &std::time::SystemTime,
) -> Result<bool, std::io::Error> {
    let time2 = std::fs::metadata(file_path)?.modified()?;

    Ok(time.cmp(&time2) != std::cmp::Ordering::Equal)
}
