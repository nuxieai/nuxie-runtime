//! Bounded, token-preserving custom-property computation. This module owns no
//! layout or runtime policy: the caller still admits the substituted property.
use crate::{Diagnostic, css::Declaration};
use cssparser::{Parser, ParserInput, ToCss, Token};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) type Variables = BTreeMap<String, Option<String>>;
const MAX_DEPTH: usize = 64;
const MAX_ENTRIES: usize = 256;
const MAX_VALUE: usize = 65_536;
const MAX_TOTAL: usize = 1_048_576;

#[derive(Clone, Debug)]
enum Node {
    Token(String),
    Group(String, Vec<Node>, char),
    Var(String, Option<Vec<Node>>),
}
fn syntax(source: &str, message: &str) -> Diagnostic {
    Diagnostic::new("css-variable-syntax", source, message)
}
fn limit(source: &str) -> Diagnostic {
    Diagnostic::new(
        "input-limit",
        source,
        "Custom properties exceed the 64-level, 256-entry, 64 KiB value or 1 MiB environment limit",
    )
}
fn parse(value: &str, source: &str) -> Result<Vec<Node>, Diagnostic> {
    if value.len() > MAX_VALUE {
        return Err(limit(source));
    }
    let mut input = ParserInput::new(value);
    parse_inner(&mut Parser::new(&mut input), source, 0)
}
fn parse_inner(
    p: &mut Parser<'_, '_>,
    source: &str,
    depth: usize,
) -> Result<Vec<Node>, Diagnostic> {
    if depth > MAX_DEPTH {
        return Err(limit(source));
    }
    let mut nodes = Vec::new();
    while !p.is_exhausted() {
        let token = p
            .next_including_whitespace_and_comments()
            .map_err(|_| syntax(source, "Invalid value token"))?
            .clone();
        if token.is_parse_error() {
            return Err(syntax(source, "Invalid value token"));
        }
        if matches!(&token, Token::Number {value, ..} | Token::Dimension {value, ..} if !value.is_finite())
            || matches!(&token, Token::Percentage {unit_value, ..} if !unit_value.is_finite())
        {
            return Err(syntax(source, "Non-finite value token"));
        }
        let group = match &token {
            Token::Function(name) => {
                Some((token.to_css_string(), ')', name.eq_ignore_ascii_case("var")))
            }
            Token::ParenthesisBlock => Some(("(".into(), ')', false)),
            Token::SquareBracketBlock => Some(("[".into(), ']', false)),
            Token::CurlyBracketBlock => Some(("{".into(), '}', false)),
            _ => None,
        };
        if let Some((open, close, variable)) = group {
            let start = p.position();
            let node = p
                .parse_nested_block(|p| {
                    let result = if variable {
                        (|| {
                            let name = p
                                .expect_ident()
                                .map_err(|_| {
                                    syntax(source, "var() requires a custom property name")
                                })?
                                .to_string();
                            if !name.starts_with("--") || name == "--" {
                                return Err(syntax(
                                    source,
                                    "var() requires a custom property name",
                                ));
                            }
                            let fallback = if p.is_exhausted() {
                                None
                            } else {
                                p.expect_comma().map_err(|_| {
                                    syntax(source, "Expected comma before var() fallback")
                                })?;
                                Some(parse_inner(p, source, depth + 1)?)
                            };
                            Ok(Node::Var(name, fallback))
                        })()
                    } else {
                        parse_inner(p, source, depth + 1).map(|body| Node::Group(open, body, close))
                    };
                    result.map_err(|e| p.new_custom_error::<_, Diagnostic>(e))
                })
                .map_err(|e| match e.kind {
                    cssparser::ParseErrorKind::Custom(e) => e,
                    _ => syntax(source, "Invalid nested value"),
                })?;
            if !p.slice_from(start).ends_with(close) {
                return Err(syntax(source, "Unclosed value block"));
            }
            nodes.push(node);
        } else {
            nodes.push(Node::Token(match token {
                Token::Comment(_) | Token::WhiteSpace(_) => " ".into(),
                _ => token.to_css_string(),
            }));
        }
    }
    Ok(nodes)
}

pub(crate) fn validate_value(value: &str, source: &str) -> Result<(), Diagnostic> {
    parse(value, source).map(|_| ())
}
pub(crate) fn contains_var(value: &str) -> bool {
    fn has(nodes: &[Node]) -> bool {
        nodes.iter().any(|n| match n {
            Node::Var(..) => true,
            Node::Group(_, n, _) => has(n),
            _ => false,
        })
    }
    parse(value, "css").is_ok_and(|n| has(&n))
}
fn refs(
    nodes: &[Node],
    result: &mut BTreeSet<String>,
    fallback_refs: &mut BTreeSet<String>,
    in_fallback: bool,
) {
    for node in nodes {
        match node {
            Node::Var(name, fallback) => {
                result.insert(name.clone());
                if in_fallback {
                    fallback_refs.insert(name.clone());
                }
                if let Some(f) = fallback {
                    refs(f, result, fallback_refs, true);
                }
            }
            Node::Group(_, body, _) => refs(body, result, fallback_refs, in_fallback),
            _ => {}
        }
    }
}
fn keyword(value: &str) -> Option<String> {
    let mut input = ParserInput::new(value);
    let mut p = Parser::new(&mut input);
    let name = p.expect_ident().ok()?.to_ascii_lowercase();
    p.expect_exhausted().ok()?;
    Some(name)
}

fn reaches(origin: &str, target: &str, graph: &BTreeMap<String, BTreeSet<String>>) -> bool {
    let mut pending = vec![origin];
    let mut seen = BTreeSet::new();
    while let Some(name) = pending.pop() {
        if name == target {
            return true;
        }
        if seen.insert(name) {
            if let Some(edges) = graph.get(name) {
                pending.extend(edges.iter().map(String::as_str));
            }
        }
    }
    false
}
fn cycle_members(graph: &BTreeMap<String, BTreeSet<String>>) -> BTreeSet<String> {
    graph
        .iter()
        .filter(|(origin, edges)| edges.iter().any(|edge| reaches(edge, origin, graph)))
        .map(|(origin, _)| origin.clone())
        .collect()
}
fn append(output: &mut String, piece: &str, source: &str) -> Result<(), Diagnostic> {
    // Comments preserve CSS token boundaries without manufacturing a dimension,
    // hash or identifier at a substitution boundary (var(--n)px is not 2px).
    let separator = if output.is_empty() || piece.is_empty() {
        ""
    } else {
        "/**/"
    };
    if output.len() + separator.len() + piece.len() > MAX_VALUE {
        return Err(limit(source));
    }
    output.push_str(separator);
    output.push_str(piece);
    Ok(())
}
fn expand(
    nodes: &[Node],
    lookup: &mut impl FnMut(&str) -> Result<Option<String>, Diagnostic>,
    source: &str,
    depth: usize,
) -> Result<Option<String>, Diagnostic> {
    if depth > MAX_DEPTH {
        return Err(limit(source));
    }
    let mut output = String::new();
    for node in nodes {
        let piece = match node {
            Node::Token(s) => s.clone(),
            Node::Group(open, body, close) => {
                let Some(body) = expand(body, lookup, source, depth + 1)? else {
                    return Ok(None);
                };
                format!("{open}{body}{close}")
            }
            Node::Var(name, fallback) => match lookup(name)? {
                Some(value) => value,
                None => match fallback {
                    Some(body) => {
                        let Some(value) = expand(body, lookup, source, depth + 1)? else {
                            return Ok(None);
                        };
                        value
                    }
                    None => return Ok(None),
                },
            },
        };
        append(&mut output, &piece, source)?;
    }
    Ok(Some(output))
}
fn resolve(
    name: &str,
    authored: &BTreeMap<String, Vec<Node>>,
    values: &mut Variables,
    depth: usize,
) -> Result<Option<String>, Diagnostic> {
    if let Some(value) = values.get(name) {
        return Ok(value.clone());
    }
    if depth >= MAX_DEPTH {
        return Err(limit(name));
    }
    let Some(nodes) = authored.get(name) else {
        return Ok(None);
    };
    let value = expand(
        nodes,
        &mut |dependency| resolve(dependency, authored, values, depth + 1),
        name,
        depth,
    )?;
    values.insert(name.into(), value.clone());
    check_environment(values, name)?;
    Ok(value)
}
fn check_environment(values: &Variables, source: &str) -> Result<(), Diagnostic> {
    if values.len() > MAX_ENTRIES
        || values
            .iter()
            .any(|(k, v)| k.len() > MAX_VALUE || v.as_ref().is_some_and(|v| v.len() > MAX_VALUE))
        || values
            .iter()
            .map(|(k, v)| k.len() + v.as_ref().map_or(0, String::len))
            .sum::<usize>()
            > MAX_TOTAL
    {
        return Err(limit(source));
    }
    Ok(())
}

/// Declarations must arrive in ascending cascade order, with importance already
/// accounted for by the caller. Last declaration of a name wins.
pub(crate) fn compute(
    declarations: &[Declaration],
    parent: &Variables,
) -> Result<Variables, Diagnostic> {
    check_environment(parent, "css")?;
    let mut selected = BTreeMap::new();
    for declaration in declarations.iter().filter(|d| d.name.starts_with("--")) {
        if declaration.name == "--" {
            return Err(syntax(&declaration.source, "Invalid custom property name"));
        }
        validate_value(&declaration.value, &declaration.source)?;
        selected.insert(declaration.name.clone(), declaration);
        if selected.len() > MAX_ENTRIES {
            return Err(limit(&declaration.source));
        }
    }
    let sources: BTreeMap<_, _> = selected
        .iter()
        .map(|(name, declaration)| (name.clone(), declaration.source.clone()))
        .collect();
    let mut values = parent.clone();
    let mut authored = BTreeMap::new();
    for (name, declaration) in selected {
        match keyword(&declaration.value).as_deref() {
            Some("initial") => {
                values.insert(name, None);
            }
            Some("inherit" | "unset") => {
                values.insert(name.clone(), parent.get(&name).cloned().unwrap_or(None));
            }
            Some("revert" | "revert-layer") => {
                return Err(Diagnostic::new(
                    "unsupported-css-value",
                    &declaration.source,
                    "Custom property revert/revert-layer is not supported",
                ));
            }
            _ => {
                values.remove(&name);
                authored.insert(name, parse(&declaration.value, &declaration.source)?);
            }
        }
    }
    let mut graph = BTreeMap::new();
    let mut fallback_graph = BTreeMap::new();
    for (name, nodes) in &authored {
        let mut dependencies = BTreeSet::new();
        let mut fallback_dependencies = BTreeSet::new();
        refs(nodes, &mut dependencies, &mut fallback_dependencies, false);
        graph.insert(name.clone(), dependencies);
        fallback_graph.insert(name.clone(), fallback_dependencies);
    }
    // Chrome's observed treatment of unused fallback cycles differs from the
    // all-reference dependency graph. Until qualified, reject every cycle
    // containing a fallback reference instead of emitting mismatching values.
    for (name, edges) in fallback_graph {
        if edges.iter().any(|edge| reaches(edge, &name, &graph)) {
            return Err(Diagnostic::new(
                "unsupported-css-value",
                &sources[&name],
                "Custom property dependency cycles through var() fallbacks are not supported",
            ));
        }
    }
    let cyclic = cycle_members(&graph);
    for name in cyclic {
        values.insert(name, None);
    }
    for name in authored.keys() {
        resolve(name, &authored, &mut values, 0)?;
    }
    check_environment(&values, "css")?;
    Ok(values)
}
pub(crate) fn substitute(
    value: &str,
    vars: &Variables,
    source: &str,
) -> Result<Option<String>, Diagnostic> {
    check_environment(vars, source)?;
    expand(
        &parse(value, source)?,
        &mut |name| Ok(vars.get(name).cloned().unwrap_or(None)),
        source,
        0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn computed(css: &str, parent: &Variables) -> Variables {
        compute(&crate::css::declarations(css, "test").unwrap(), parent).unwrap()
    }
    fn ordinary(value: &str, vars: &Variables) -> Option<String> {
        substitute(value, vars, "test")
            .unwrap()
            .map(|v| crate::css::ordinary_value(&v).unwrap())
    }
    #[test]
    fn frozen_inheritance_and_names() {
        let parent = computed(
            "--a: 10px; --b: var(--a); --A: 20px; --\\63: 30px",
            &Variables::new(),
        );
        let child = computed("--a: 50px; --A: initial; --c: inherit", &parent);
        assert_eq!(ordinary("var(--b)", &child).unwrap().trim(), "10px");
        assert_eq!(ordinary("var(--a)", &child).unwrap().trim(), "50px");
        assert_eq!(ordinary("var(--c)", &child).unwrap().trim(), "30px");
        assert_eq!(ordinary("var(--A)", &child), None);
    }
    #[test]
    fn direct_cycles_are_invalid_but_dependents_can_recover() {
        let vars = computed(
            "--ok: red; --a: var(--b); --b: var(--a); --recover: var(--a,blue); --self: var(--self)",
            &Variables::new(),
        );
        let overlapping = computed(
            "--a: var(--b) var(--c); --b: var(--a); --c: var(--b)",
            &Variables::new(),
        );
        assert!(overlapping.values().all(Option::is_none));
        assert_eq!(vars["--a"], None);
        assert_eq!(vars["--b"], None);
        assert_eq!(vars["--self"], None);
        assert_eq!(ordinary("var(--recover)", &vars).unwrap().trim(), "blue");
        assert_eq!(
            ordinary("var(--missing, var(--ok))", &vars).unwrap().trim(),
            "red"
        );
        assert_eq!(ordinary("var(--missing,)", &vars), Some(String::new()));
    }
    #[test]
    fn fallback_cycle_edges_are_explicitly_unqualified() {
        for css in [
            "--good: navy; --x: var(--good,var(--x))",
            "--x: var(--missing,var(--x))",
            "--good: navy; --a: var(--good,var(--b)); --b: var(--a)",
            "--a: var(--b,var(--c)); --b: var(--a); --c: var(--a)",
            "--a: f(var(--missing,var(--a)))",
        ] {
            let declarations = crate::css::declarations(css, "fallback-cycle").unwrap();
            let error = compute(&declarations, &Variables::new()).unwrap_err();
            assert_eq!(error.code, "unsupported-css-value", "{css}");
            assert!(error.source.starts_with("fallback-cycle:"));
            assert!(error.message.contains("cycles through var() fallbacks"));
        }
        // A fallback edge between separate cycles is not itself a cycle edge.
        let independent = computed("--a: var(--a,var(--b)); --b: var(--b)", &Variables::new());
        assert!(independent.values().all(Option::is_none));
    }
    #[test]
    fn substitution_preserves_token_boundaries() {
        let vars = computed("--n: 2; --id: foo", &Variables::new());
        let value = substitute("var(--n)px", &vars, "test").unwrap().unwrap();
        let mut input = ParserInput::new(&value);
        let mut p = Parser::new(&mut input);
        assert!(matches!(p.next().unwrap(), Token::Number { .. }));
        assert!(matches!(p.next().unwrap(), Token::Ident(v) if *v == "px"));
        assert!(p.is_exhausted());
        assert_ne!(ordinary("var(--id)bar", &vars).unwrap(), "foobar");
        assert!(contains_var("rgb(var(--n), 0, 0)"));
        assert!(contains_var("v\\61r(--n)"));
        assert!(!contains_var("'var(--n)'"));
    }
    #[test]
    fn malformed_syntax_and_limits_are_errors() {
        for value in [
            "var(x)",
            "var(--)",
            "var(--x other)",
            "var(--x",
            "var()",
            "var(--x, var(bad))",
        ] {
            assert!(validate_value(value, "test").is_err(), "{value}");
        }
        assert!(validate_value(&"x".repeat(MAX_VALUE + 1), "test").is_err());
        assert!(
            validate_value(
                &format!(
                    "{}x{}",
                    "f(".repeat(MAX_DEPTH + 1),
                    ")".repeat(MAX_DEPTH + 1)
                ),
                "test"
            )
            .is_err()
        );
        let vars = BTreeMap::from([("--x".into(), Some("x".repeat(MAX_VALUE / 2)))]);
        assert!(substitute("var(--x)var(--x)", &vars, "test").is_err());
        let parent = (0..=MAX_ENTRIES)
            .map(|n| (format!("--x{n}"), None))
            .collect();
        assert!(compute(&[], &parent).is_err());
    }
}
