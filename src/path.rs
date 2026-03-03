#[derive(PartialEq, Clone, Debug)]
pub enum PathEntry {
    Field(String),
    Index(usize),
    IndexNew,
}

pub fn decode(path: &str) -> Option<Vec<PathEntry>> {
    let mut result = vec![];
    let mut current = String::new();
    let mut is_parsing_index = false;
    let mut last_char = '0';

    for (i, c) in path.chars().enumerate() {
        if last_char == '.' && (c == '[' || c == ']') {
            return None;
        }

        last_char = c;

        if c == '.' {
            if i != 0 && path.chars().nth(i - 1).unwrap() != ']' {
                result.push(PathEntry::Field(current.clone()));
            }

            current.clear();
            is_parsing_index = false;

            continue;
        }

        if c == '[' {
            if !current.is_empty() {
                result.push(PathEntry::Field(current.clone()));
                current.clear();
            }

            is_parsing_index = true;

            continue;
        }

        if c == ']' {
            is_parsing_index = false;

            if current.is_empty() {
                result.push(PathEntry::IndexNew);
            } else {
                result.push(PathEntry::Index(current.parse::<usize>().unwrap()));
            }

            if i < (path.len() - 1) {
                current.clear();
            }

            continue;
        }

        if is_parsing_index && !is_number(&c) {
            return None;
        }

        if is_parsing_index {
            current.push(c);

            continue;
        }

        current.push(c);
    }

    if last_char != ']' {
        result.push(PathEntry::Field(current));
    }

    Some(result)
}

pub fn encode(path: &[PathEntry]) -> String {
    path.iter()
        .enumerate()
        .map(|(i, entry)| match entry {
            PathEntry::Field(field_name) => format!(
                "{}{}",
                match i {
                    0 => "",
                    _ => ".",
                },
                field_name
            ),
            PathEntry::Index(index) => format!("[{}]", index),
            PathEntry::IndexNew => "[]".to_string(),
        })
        .collect::<Vec<String>>()
        .join("")
}

fn is_number(c: &char) -> bool {
    c.to_string().parse::<usize>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(
            encode(&vec![
                PathEntry::Field("foo".to_string()),
                PathEntry::Field("bar".to_string())
            ]),
            "foo.bar",
        );

        assert_eq!(
            encode(&vec![
                PathEntry::Field("foo".to_string()),
                PathEntry::IndexNew,
            ]),
            "foo[]",
        );

        assert_eq!(
            encode(&vec![
                PathEntry::Field("foo".to_string()),
                PathEntry::Index(10),
                PathEntry::Field("bar".to_string())
            ]),
            "foo[10].bar",
        );

        assert_eq!(
            encode(&vec![PathEntry::Index(1), PathEntry::Index(2),]),
            "[1][2]",
        );
    }

    #[test]
    fn test_decode_valid_cases() {
        assert_eq!(
            decode("foo.bar"),
            Some(vec![
                PathEntry::Field("foo".to_string()),
                PathEntry::Field("bar".to_string())
            ]),
        );

        assert_eq!(
            decode("foo.bar[10]"),
            Some(vec![
                PathEntry::Field("foo".to_string()),
                PathEntry::Field("bar".to_string()),
                PathEntry::Index(10),
            ]),
        );

        assert_eq!(
            decode("foo.bar[]"),
            Some(vec![
                PathEntry::Field("foo".to_string()),
                PathEntry::Field("bar".to_string()),
                PathEntry::IndexNew,
            ]),
        );

        assert_eq!(
            decode("foo.bar[10][20]"),
            Some(vec![
                PathEntry::Field("foo".to_string()),
                PathEntry::Field("bar".to_string()),
                PathEntry::Index(10),
                PathEntry::Index(20),
            ]),
        );

        assert_eq!(
            decode("foo.bar[10][20].foo"),
            Some(vec![
                PathEntry::Field("foo".to_string()),
                PathEntry::Field("bar".to_string()),
                PathEntry::Index(10),
                PathEntry::Index(20),
                PathEntry::Field("foo".to_string()),
            ]),
        );

        assert_eq!(
            decode("[0][3].foo"),
            Some(vec![
                PathEntry::Index(0),
                PathEntry::Index(3),
                PathEntry::Field("foo".to_string()),
            ]),
        );
    }

    #[test]
    fn test_decode_invalid_cases() {
        for invalid_case in vec![
            "foo[bar]",
            "foo[1bar2]",
            "foo.[",
            "foo.]",
            "foo.[0]",
            "foo.]bar",
        ] {
            assert_eq!(decode(invalid_case), None);
        }
    }
}
