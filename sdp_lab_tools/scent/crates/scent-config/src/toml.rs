//! A minimal, dependency-free parser for the TOML subset `smell_detector.toml`
//! actually uses (`docs/prompt.md` §36): `[section]` and `[section.sub]`
//! headers, `key = value` scalars (string/integer/float/bool), and string
//! arrays. No dates, inline tables, or multi-line strings — this schema
//! doesn't use them, and adding support would be speculative generality by
//! the project's own rule.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum TomlValue {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<TomlValue>),
}

impl TomlValue {
    /// Threshold/count values in this schema are small enough (well under
    /// 2^52) that converting a TOML integer to `f64` never actually loses
    /// precision in practice.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Integer(value) => Some(*value as f64),
            Self::Float(value) => Some(*value),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }
}

/// A flattened document: every `[a.b.c]` section header becomes the key
/// prefix `"a.b.c."` for the scalars that follow it, up to the next header.
/// Top-level keys (before any header) use an empty prefix.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TomlDocument {
    entries: BTreeMap<String, TomlValue>,
}

impl TomlDocument {
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&TomlValue> {
        self.entries.get(key)
    }

    /// Every entry whose key starts with `prefix` (e.g. `"rules."`), paired
    /// with the remainder of its key after the prefix.
    pub fn entries_under<'a>(
        &'a self,
        prefix: &'a str,
    ) -> impl Iterator<Item = (&'a str, &'a TomlValue)> {
        self.entries
            .iter()
            .filter_map(move |(key, value)| key.strip_prefix(prefix).map(|rest| (rest, value)))
    }
}

/// Parses `text` into a [`TomlDocument`]. Malformed lines are skipped rather
/// than erroring: a config file is optional supporting data, not a
/// correctness-critical input — a typo in one line should not take down
/// the whole analysis.
#[must_use]
pub fn parse(text: &str) -> TomlDocument {
    let mut entries = BTreeMap::new();
    let mut section = String::new();
    for raw_line in text.lines() {
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if let Some(header) = line
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            section = format!("{}.", header.trim());
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let Some(value) = parse_value(value.trim()) else {
            continue;
        };
        entries.insert(format!("{section}{key}"), value);
    }
    TomlDocument { entries }
}

fn strip_comment(line: &str) -> &str {
    // A `#` inside a quoted string must not be treated as a comment marker.
    let mut in_string = false;
    for (index, ch) in line.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..index],
            _ => {}
        }
    }
    line
}

fn parse_value(text: &str) -> Option<TomlValue> {
    if let Some(inner) = text
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        return Some(TomlValue::String(inner.to_owned()));
    }
    if text == "true" {
        return Some(TomlValue::Bool(true));
    }
    if text == "false" {
        return Some(TomlValue::Bool(false));
    }
    if let Some(inner) = text
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    {
        let items = split_array_items(inner)
            .into_iter()
            .filter_map(|item| parse_value(item.trim()))
            .collect();
        return Some(TomlValue::Array(items));
    }
    if let Ok(value) = text.parse::<i64>() {
        return Some(TomlValue::Integer(value));
    }
    if let Ok(value) = text.parse::<f64>() {
        return Some(TomlValue::Float(value));
    }
    None
}

/// Splits `"a", "b", "c"` on top-level commas, ignoring commas inside quoted
/// strings.
fn split_array_items(inner: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let mut start = 0;
    let mut in_string = false;
    for (index, ch) in inner.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            ',' if !in_string => {
                items.push(&inner[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    let last = inner[start..].trim();
    if !last.is_empty() {
        items.push(last);
    }
    items
}

#[cfg(test)]
mod tests {
    use super::{parse, TomlValue};

    #[test]
    fn parses_top_level_scalars() {
        let doc = parse("max_parallel_jobs = 8\ncache_directory = \".scent_cache\"\n");
        assert_eq!(doc.get("max_parallel_jobs"), Some(&TomlValue::Integer(8)));
        assert_eq!(
            doc.get("cache_directory"),
            Some(&TomlValue::String(".scent_cache".into()))
        );
    }

    #[test]
    fn parses_nested_sections() {
        let doc = parse(
            "[rules.LONG_METHOD]\nenabled = true\n\n[rules.LONG_METHOD.thresholds]\nloc = 30\n",
        );
        assert_eq!(
            doc.get("rules.LONG_METHOD.enabled"),
            Some(&TomlValue::Bool(true))
        );
        assert_eq!(
            doc.get("rules.LONG_METHOD.thresholds.loc"),
            Some(&TomlValue::Integer(30))
        );
    }

    #[test]
    fn parses_a_float_threshold() {
        let doc = parse("[rules.DUPLICATED_CODE.thresholds]\nsimilarity = 0.85\n");
        assert_eq!(
            doc.get("rules.DUPLICATED_CODE.thresholds.similarity")
                .and_then(TomlValue::as_f64),
            Some(0.85)
        );
    }

    #[test]
    fn parses_a_string_array() {
        let doc = parse("[exclusions]\npaths = [\"**/bin/**\", \"**/obj/**\"]\n");
        let TomlValue::Array(items) = doc.get("exclusions.paths").unwrap() else {
            panic!("expected an array");
        };
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].as_str(), Some("**/bin/**"));
    }

    #[test]
    fn ignores_comments_and_blank_lines() {
        let doc = parse("# a comment\n\nmax_parallel_jobs = 8 # trailing comment\n");
        assert_eq!(doc.get("max_parallel_jobs"), Some(&TomlValue::Integer(8)));
    }

    #[test]
    fn entries_under_a_prefix_strips_it() {
        let doc =
            parse("[rules.LONG_METHOD]\nenabled = true\n[rules.LARGE_CLASS]\nenabled = false\n");
        let under: Vec<_> = doc.entries_under("rules.").collect();
        assert_eq!(under.len(), 2);
    }

    #[test]
    fn skips_a_malformed_line_without_failing_the_whole_parse() {
        let doc = parse("not a valid line\nmax_parallel_jobs = 8\n");
        assert_eq!(doc.get("max_parallel_jobs"), Some(&TomlValue::Integer(8)));
    }
}
