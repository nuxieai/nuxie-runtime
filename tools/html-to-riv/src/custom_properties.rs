//! Token-preserving custom-property resolution. Cascade order is supplied by the caller.
use crate::Diagnostic;
use cssparser::{Parser, ParserInput, Token};
use std::collections::{BTreeMap, BTreeSet};

const MAX_EXPANSION: usize = 65_536;
#[derive(Clone, Debug)]
pub(crate) struct Value(Vec<Part>);
#[derive(Clone, Debug)]
enum Part {
    Raw(String),
    Group(String, Value, char),
    Variable(String, Option<Value>),
}
pub(crate) type Computed = BTreeMap<String, Option<String>>;
fn error(message: &str) -> Diagnostic {
    Diagnostic::new("custom-property-syntax", "css", message)
}
impl Value {
    pub(crate) fn parse(text: &str) -> Result<Self, Diagnostic> {
        let mut input = ParserInput::new(text);
        Self::read(&mut Parser::new(&mut input), 0)
    }
    fn read(p: &mut Parser<'_, '_>, depth: usize) -> Result<Self, Diagnostic> {
        if depth > 64 {
            return Err(error("Custom value nesting exceeds 64 levels"));
        }
        let mut parts = Vec::new();
        while !p.is_exhausted() {
            let start = p.position();
            let token = p
                .next_including_whitespace_and_comments()
                .map_err(|_| error("Invalid custom value"))?
                .clone();
            let prefix = p.slice_from(start).to_owned();
            let close = match &token {
                Token::Function(_) | Token::ParenthesisBlock => Some(')'),
                Token::SquareBracketBlock => Some(']'),
                Token::CurlyBracketBlock => Some('}'),
                _ => None,
            };
            if let Some(close) = close {
                let variable =
                    matches!(&token, Token::Function(name) if name.eq_ignore_ascii_case("var"));
                let body = p
                    .parse_nested_block(|p| {
                        if variable {
                            let name = p.expect_ident()?.to_string();
                            if !name.starts_with("--") || name.len() == 2 {
                                return Err(p.new_custom_error(error(
                                    "var requires a custom property name",
                                )));
                            }
                            let fallback = if p.is_exhausted() {
                                None
                            } else {
                                p.expect_comma()?;
                                Some(Self::read(p, depth + 1).map_err(|e| p.new_custom_error(e))?)
                            };
                            Ok(Part::Variable(name, fallback))
                        } else {
                            Ok(Part::Group(
                                prefix,
                                Self::read(p, depth + 1).map_err(|e| p.new_custom_error(e))?,
                                close,
                            ))
                        }
                    })
                    .map_err(|e: cssparser::ParseError<'_, Diagnostic>| match e.kind {
                        cssparser::ParseErrorKind::Custom(e) => e,
                        _ => error("Malformed var function"),
                    })?;
                if !p.slice_from(start).ends_with(close) {
                    return Err(error("Unclosed custom value block"));
                }
                parts.push(body);
            } else {
                if token.is_parse_error() {
                    return Err(error("Invalid custom value token"));
                }
                parts.push(Part::Raw(prefix));
            }
        }
        Ok(Self(parts))
    }
    pub(crate) fn validate_custom(&self) -> Result<(), Diagnostic> {
        if matches!(self.keyword().as_deref(), Some("revert" | "revert-layer")) {
            return Err(error("revert and revert-layer are not yet supported"));
        }
        Ok(())
    }
    pub(crate) fn has_variables(&self) -> bool {
        self.0.iter().any(|p| match p {
            Part::Variable(..) => true,
            Part::Group(_, value, _) => value.has_variables(),
            _ => false,
        })
    }
    fn keyword(&self) -> Option<String> {
        let raw = self
            .0
            .iter()
            .map(|p| match p {
                Part::Raw(s) => Some(s.as_str()),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?
            .concat();
        let mut input = ParserInput::new(&raw);
        let mut p = Parser::new(&mut input);
        let name = p.expect_ident().ok()?.to_ascii_lowercase();
        p.expect_exhausted().ok()?;
        Some(name)
    }
    fn substitute(&self, lookup: &mut impl FnMut(&str) -> Option<String>) -> Option<String> {
        let mut out = String::new();
        for part in &self.0 {
            match part {
                Part::Raw(raw) => out.push_str(raw),
                Part::Group(prefix, body, close) => {
                    out.push_str(prefix);
                    out.push_str(&body.substitute(lookup)?);
                    out.push(*close);
                }
                Part::Variable(name, fallback) => {
                    let value = lookup(name).or_else(|| fallback.as_ref()?.substitute(lookup))?;
                    // Substitution inserts tokens, never concatenates neighboring tokens.
                    out.push_str("/**/");
                    out.push_str(&value);
                    out.push_str("/**/");
                }
            }
            if out.len() > MAX_EXPANSION {
                return None;
            }
        }
        Some(out)
    }
    pub(crate) fn resolve(&self, computed: &Computed) -> Option<String> {
        self.substitute(&mut |name| computed.get(name).cloned().flatten())
    }
}

/// Resolve each element before inheritance. Values in `parent` contain no var calls.
pub(crate) fn compute(parent: &Computed, declarations: &[(String, Value)]) -> Computed {
    let mut local = BTreeMap::new();
    let mut inherited = parent.clone();
    for (name, value) in declarations {
        match value.keyword().as_deref() {
            Some("inherit" | "unset") => {
                local.remove(name);
                inherited.insert(name.clone(), parent.get(name).cloned().flatten());
            }
            Some("initial") => {
                local.remove(name);
                inherited.insert(name.clone(), None);
            }
            _ => {
                local.insert(name.clone(), value.clone());
            }
        }
    }
    for name in local.keys() { inherited.remove(name); }
    for name in local.keys() {
        resolve_property(name, &local, &mut inherited);
    }
    inherited
}

// An explicit continuation stack bounds native/WASM call-stack use independently
// of the product of value nesting and custom-property dependency depth.
fn resolve_property<'a>(name: &'a str, local: &'a BTreeMap<String, Value>, computed: &mut Computed) {
    enum Task<'a> {
        Property(&'a str), FinishProperty(&'a str),
        Value(&'a Value), Part(&'a Part),
        Join(&'a Value, usize, Option<String>),
        Group(&'a str, char), AfterLookup(Option<&'a Value>), Wrap,
    }
    fn join(a: Option<String>, b: Option<String>) -> Option<String> {
        let mut a = a?; let b = b?;
        if a.len().checked_add(b.len())? > MAX_EXPANSION { return None; }
        a.push_str(&b); Some(a)
    }
    fn wrap(value: Option<String>) -> Option<String> {
        let value = value?;
        if value.len() + 8 > MAX_EXPANSION { return None; }
        Some(format!("/**/{value}/**/"))
    }
    let mut work = vec![Task::Property(name)];
    let mut results: Vec<Option<String>> = Vec::new();
    let mut active: Vec<&str> = Vec::new();
    let mut cyclic = BTreeSet::new();
    while let Some(task) = work.pop() {
        match task {
            Task::Property(name) => {
                if let Some(value) = computed.get(name) { results.push(value.clone()); }
                else if let Some(start) = active.iter().position(|n| *n == name) {
                    cyclic.extend(active[start..].iter().copied()); results.push(None);
                } else if let Some(value) = local.get(name) {
                    active.push(name); work.push(Task::FinishProperty(name)); work.push(Task::Value(value));
                } else { results.push(None); }
            }
            Task::FinishProperty(name) => {
                let value = results.pop().expect("property result");
                let finished = active.pop();
                debug_assert_eq!(finished, Some(name));
                let value = if cyclic.contains(name) { None } else { value };
                computed.insert(name.to_owned(), value.clone()); results.push(value);
            }
            Task::Value(value) => {
                if let Some(first) = value.0.first() {
                    work.push(Task::Join(value, 1, Some(String::new()))); work.push(Task::Part(first));
                } else { results.push(Some(String::new())); }
            }
            Task::Part(part) => match part {
                Part::Raw(raw) => results.push((raw.len() <= MAX_EXPANSION).then(|| raw.clone())),
                Part::Group(prefix, value, close) => { work.push(Task::Group(prefix, *close)); work.push(Task::Value(value)); }
                Part::Variable(name, fallback) => { work.push(Task::AfterLookup(fallback.as_ref())); work.push(Task::Property(name)); }
            },
            Task::Join(value, next, accumulated) => {
                let combined = join(accumulated, results.pop().expect("component result"));
                if let Some(part) = value.0.get(next) {
                    work.push(Task::Join(value, next + 1, combined)); work.push(Task::Part(part));
                } else { results.push(combined); }
            }
            Task::Group(prefix, close) => {
                let body = results.pop().expect("group result");
                let prefix = (prefix.len() <= MAX_EXPANSION).then(|| prefix.to_owned());
                results.push(join(join(prefix, body), Some(close.to_string())));
            }
            Task::AfterLookup(fallback) => {
                let value = results.pop().expect("lookup result");
                if value.is_some() { results.push(wrap(value)); }
                else if let Some(fallback) = fallback { work.push(Task::Wrap); work.push(Task::Value(fallback)); }
                else { results.push(None); }
            }
            Task::Wrap => { let value = results.pop().expect("fallback result"); results.push(wrap(value)); }
        }
    }
    debug_assert_eq!(results.len(), 1);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn declarations(values: &[(&str, &str)]) -> Vec<(String, Value)> {
        values
            .iter()
            .map(|(n, v)| (n.to_string(), Value::parse(v).unwrap()))
            .collect()
    }
    fn value(css: &str, map: &Computed) -> Option<String> {
        Value::parse(css).unwrap().resolve(map)
    }
    #[test]
    fn preserves_tokens_case_strings_and_empty_fallbacks() {
        let c = compute(
            &Computed::new(),
            &declarations(&[("--X", "1"), ("--x", "2"), ("--empty", "")]),
        );
        assert_eq!(value("var(--X)px", &c).unwrap(), "/**/1/**/px");
        assert_eq!(value("var(--x)", &c).unwrap(), "/**/2/**/");
        assert_eq!(value("'var(--X)'", &c).unwrap(), "'var(--X)'");
        assert_eq!(value("var(--absent,)", &c).unwrap(), "/**//**/");
        assert_eq!(value("var(--empty,red)", &c).unwrap(), "/**//**/");
        assert_eq!(value("var(--missing)", &c), None);
        assert_eq!(
            value("rgb(var(--X), 2, 3)", &c).unwrap(),
            "rgb(/**/1/**/, 2, 3)"
        );
    }
    #[test]
    fn fallback_edges_only_participate_when_evaluated() {
        let c = compute(
            &Computed::new(),
            &declarations(&[
                ("--safe", "red"),
                ("--a", "var(--safe,var(--b))"),
                ("--b", "var(--a)"),
                ("--outside", "var(--a,green)"),
            ]),
        );
        assert!(c["--a"].as_ref().unwrap().contains("red"));
        assert!(c["--b"].as_ref().unwrap().contains("red"));
        assert!(c["--outside"].as_ref().unwrap().contains("red"));
        let c = compute(&Computed::new(), &declarations(&[("--a", "var(--a,red)")]));
        assert_eq!(c["--a"], None);
    }
    #[test]
    fn resolves_before_inheritance_and_respects_cascade_and_wide_keywords() {
        let parent = compute(
            &Computed::new(),
            &declarations(&[("--x", "red"), ("--alias", "var(--x)")]),
        );
        let child = compute(
            &parent,
            &declarations(&[
                ("--x", "blue"),
                ("--alias", "inherit"),
                ("--gone", "initial"),
            ]),
        );
        assert!(child["--alias"].as_ref().unwrap().contains("red"));
        assert_eq!(child["--x"], Some("blue".into()));
        assert_eq!(child["--gone"], None);
        let c = compute(&parent, &declarations(&[("--x", "blue"), ("--x", "unset")]));
        assert_eq!(c["--x"], Some("red".into()));
    }
    #[test]
    fn rejects_bad_var_syntax_and_bounds_expansion() {
        for v in ["var(x)", "var(--)", "var(--a other)", "var(--a"] {
            assert!(Value::parse(v).is_err(), "{v}");
        }
        let mut d = vec![("--n0".into(), Value::parse("abc").unwrap())];
        for i in 1..20 {
            d.push((
                format!("--n{i}"),
                Value::parse(&format!("var(--n{})var(--n{})", i - 1, i - 1)).unwrap(),
            ));
        }
        let c = compute(&Computed::new(), &d);
        assert_eq!(c["--n19"], None);
    }
}
