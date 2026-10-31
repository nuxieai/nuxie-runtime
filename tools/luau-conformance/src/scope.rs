//! The scope file: which upstream conformance scripts must pass.
//!
//! One entry per line, `#` starts a comment:
//!
//! ```text
//! required <script>
//! expected-fail <script> <reason>
//! permanent-non-goal <script> <reason>
//! ```
//!
//! `required` scripts must return "OK". `expected-fail` scripts are known
//! gaps that are not permanent: one that starts passing fails the run until
//! it is promoted to `required`, so the pass set only grows.
//! `permanent-non-goal` scripts are outside the Rive profile; they still run
//! (for the record) but never gate. Every non-required entry carries a reason.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Required,
    ExpectedFail,
    PermanentNonGoal,
}

impl Kind {
    fn parse(word: &str) -> Option<Kind> {
        match word {
            "required" => Some(Kind::Required),
            "expected-fail" => Some(Kind::ExpectedFail),
            "permanent-non-goal" => Some(Kind::PermanentNonGoal),
            _ => None,
        }
    }

    /// Short column label for result lines.
    pub fn label(self) -> &'static str {
        match self {
            Kind::Required => "req",
            Kind::ExpectedFail => "xfail",
            Kind::PermanentNonGoal => "nongoal",
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Kind::Required => "required",
            Kind::ExpectedFail => "expected-fail",
            Kind::PermanentNonGoal => "permanent-non-goal",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub kind: Kind,
    pub reason: Option<String>,
}

#[derive(Debug, Default)]
pub struct Scope {
    pub entries: BTreeMap<String, Entry>,
}

impl Scope {
    pub fn parse(text: &str) -> Result<Scope, String> {
        let mut scope = Scope::default();
        let mut errors = Vec::new();
        for (index, raw) in text.lines().enumerate() {
            let line_no = index + 1;
            let line = raw.split('#').next().unwrap_or_default().trim();
            if line.is_empty() {
                continue;
            }
            let mut words = line.splitn(3, char::is_whitespace);
            let kind_word = words.next().unwrap_or_default();
            let script = words.next().unwrap_or_default().trim();
            let reason = words
                .next()
                .map(str::trim)
                .filter(|reason| !reason.is_empty());
            let Some(kind) = Kind::parse(kind_word) else {
                errors.push(format!("line {line_no}: unknown kind '{kind_word}'"));
                continue;
            };
            if script.is_empty() {
                errors.push(format!("line {line_no}: missing script name"));
                continue;
            }
            match (kind, reason) {
                (Kind::Required, Some(_)) => {
                    errors.push(format!("line {line_no}: required entries take no reason"));
                    continue;
                }
                (Kind::ExpectedFail | Kind::PermanentNonGoal, None) => {
                    errors.push(format!("line {line_no}: {kind} {script} needs a reason"));
                    continue;
                }
                _ => {}
            }
            let entry = Entry {
                kind,
                reason: reason.map(str::to_owned),
            };
            if scope.entries.insert(script.to_owned(), entry).is_some() {
                errors.push(format!("line {line_no}: {script} is listed twice"));
            }
        }
        if errors.is_empty() {
            Ok(scope)
        } else {
            Err(errors.join("\n"))
        }
    }

    /// Every corpus script must be scoped and every scoped script must exist,
    /// so a pin roll that adds or removes scripts forces a classification.
    pub fn check_against(&self, corpus: &[String]) -> Result<(), String> {
        let corpus: BTreeSet<&str> = corpus.iter().map(String::as_str).collect();
        let mut errors = Vec::new();
        for script in &corpus {
            if !self.entries.contains_key(*script) {
                errors.push(format!(
                    "{script} is in the corpus but not in the scope file"
                ));
            }
        }
        for script in self.entries.keys() {
            if !corpus.contains(script.as_str()) {
                errors.push(format!(
                    "{script} is in the scope file but not in the corpus"
                ));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n"))
        }
    }

    pub fn count(&self, kind: Kind) -> usize {
        self.entries
            .values()
            .filter(|entry| entry.kind == kind)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_three_kinds_with_comments() {
        let scope = Scope::parse(
            "# header\nrequired a.luau\nexpected-fail b.luau b.luau:3 chunk names # trailing\n\
             permanent-non-goal c.luau needs newproxy\n",
        )
        .unwrap();
        assert_eq!(
            scope.entries["a.luau"],
            Entry {
                kind: Kind::Required,
                reason: None
            }
        );
        assert_eq!(
            scope.entries["b.luau"].reason.as_deref(),
            Some("b.luau:3 chunk names")
        );
        assert_eq!(scope.entries["c.luau"].kind, Kind::PermanentNonGoal);
        assert_eq!(scope.count(Kind::Required), 1);
    }

    #[test]
    fn rejects_missing_reasons_duplicates_and_unknown_kinds() {
        let error = Scope::parse(
            "expected-fail a.luau\nrequired b.luau\nrequired b.luau\nflaky c.luau x\n",
        )
        .unwrap_err();
        assert!(error.contains("a.luau needs a reason"), "{error}");
        assert!(error.contains("b.luau is listed twice"), "{error}");
        assert!(error.contains("unknown kind 'flaky'"), "{error}");
    }

    #[test]
    fn corpus_and_scope_must_agree() {
        let scope = Scope::parse("required a.luau\nrequired gone.luau\n").unwrap();
        let error = scope
            .check_against(&["a.luau".into(), "new.luau".into()])
            .unwrap_err();
        assert!(error.contains("new.luau is in the corpus"), "{error}");
        assert!(error.contains("gone.luau is in the scope file"), "{error}");
    }
}
