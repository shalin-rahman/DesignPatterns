//! Source-comment suppression scanning. Scanning is a plain, language-neutral
//! text pass over `// scent:disable RULE` / `// scent:enable RULE` comments;
//! rules only ever query the resulting [`SuppressionMap`], never comments
//! themselves.

use std::collections::BTreeMap;

use crate::NormalizedPath;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuppressedRange {
    pub rule_id: String,
    pub start_line: u32,
    /// `None` means the suppression runs to the end of the file (an
    /// unmatched `disable` with no later `enable`).
    pub end_line: Option<u32>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SuppressionMap {
    files: BTreeMap<NormalizedPath, Vec<SuppressedRange>>,
}

impl SuppressionMap {
    /// Scans one file's raw source text for suppression directives.
    #[must_use]
    pub fn scan(path: &NormalizedPath, contents: &str) -> Self {
        let mut open: BTreeMap<String, u32> = BTreeMap::new();
        let mut ranges = Vec::new();
        for (line_index, line) in contents.lines().enumerate() {
            let line_number = u32::try_from(line_index).unwrap_or(u32::MAX);
            if let Some(rule_id) = directive(line, "disable") {
                open.entry(rule_id.to_owned()).or_insert(line_number);
            } else if let Some(rule_id) = directive(line, "enable") {
                if let Some(start_line) = open.remove(rule_id) {
                    ranges.push(SuppressedRange {
                        rule_id: rule_id.to_owned(),
                        start_line,
                        end_line: Some(line_number),
                    });
                }
            }
        }
        for (rule_id, start_line) in open {
            ranges.push(SuppressedRange {
                rule_id,
                start_line,
                end_line: None,
            });
        }
        ranges.sort_by(|left, right| {
            left.start_line
                .cmp(&right.start_line)
                .then(left.rule_id.cmp(&right.rule_id))
        });

        let mut files = BTreeMap::new();
        if !ranges.is_empty() {
            files.insert(path.clone(), ranges);
        }
        Self { files }
    }

    #[must_use]
    pub fn merge(mut self, other: Self) -> Self {
        for (path, ranges) in other.files {
            self.files.entry(path).or_default().extend(ranges);
        }
        self
    }

    #[must_use]
    pub fn is_suppressed(&self, path: &NormalizedPath, rule_id: &str, line: u32) -> bool {
        self.files.get(path).is_some_and(|ranges| {
            ranges.iter().any(|range| {
                range.rule_id == rule_id
                    && range.start_line <= line
                    && range.end_line.is_none_or(|end| line <= end)
            })
        })
    }
}

fn directive<'a>(line: &'a str, verb: &str) -> Option<&'a str> {
    let comment = line.trim_start().strip_prefix("//")?.trim_start();
    let rule_id = comment.strip_prefix("scent:")?.strip_prefix(verb)?.trim();
    (!rule_id.is_empty()).then_some(rule_id)
}

#[cfg(test)]
mod tests {
    use super::SuppressionMap;
    use crate::NormalizedPath;

    fn path() -> NormalizedPath {
        NormalizedPath::parse("src/Order.cs").unwrap()
    }

    #[test]
    fn suppresses_only_lines_between_a_matched_disable_and_enable() {
        let contents = "a\n// scent:disable LONG_METHOD\nb\nc\n// scent:enable LONG_METHOD\nd";
        let map = SuppressionMap::scan(&path(), contents);
        assert!(!map.is_suppressed(&path(), "LONG_METHOD", 0));
        assert!(map.is_suppressed(&path(), "LONG_METHOD", 1));
        assert!(map.is_suppressed(&path(), "LONG_METHOD", 4));
        assert!(!map.is_suppressed(&path(), "LONG_METHOD", 5));
    }

    #[test]
    fn an_unmatched_disable_suppresses_to_end_of_file() {
        let contents = "// scent:disable LARGE_CLASS\na\nb";
        let map = SuppressionMap::scan(&path(), contents);
        assert!(map.is_suppressed(&path(), "LARGE_CLASS", 2));
    }

    #[test]
    fn an_unmatched_enable_is_ignored() {
        let contents = "// scent:enable LARGE_CLASS\na";
        let map = SuppressionMap::scan(&path(), contents);
        assert!(!map.is_suppressed(&path(), "LARGE_CLASS", 1));
    }

    #[test]
    fn a_different_rule_id_is_not_suppressed() {
        let contents = "// scent:disable LONG_METHOD\na";
        let map = SuppressionMap::scan(&path(), contents);
        assert!(!map.is_suppressed(&path(), "LARGE_CLASS", 1));
    }
}
