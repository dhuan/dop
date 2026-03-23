use std::collections::HashMap;
use std::hash::Hash;

pub fn regex_test(regex_str: &str, subject: &str) -> bool {
    match regex::Regex::new(regex_str) {
        Err(_) => false,
        Ok(re) => re.is_match(subject),
    }
}

pub fn map_from_list<K, V>(list: &[(K, V)]) -> HashMap<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    list.iter().cloned().collect()
}
