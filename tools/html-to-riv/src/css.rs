use crate::Diagnostic;
use cssparser::{Delimiter, Parser, ParserInput, ToCss, Token};
use scraper::ElementRef;
use selectors::parser::{ParseRelative, SelectorList, SelectorParseErrorKind};

struct ProfileSelector(SelectorList<scraper::selector::Simple>);
struct ProfileSelectorParser;
impl<'i> selectors::parser::Parser<'i> for ProfileSelectorParser {
    type Impl = scraper::selector::Simple;
    type Error = SelectorParseErrorKind<'i>;
    fn parse_nth_child_of(&self) -> bool { true }
    fn parse_is_and_where(&self) -> bool { true }
}
impl ProfileSelector {
    fn parse(text: &str) -> Result<Self, String> {
        let mut input = ParserInput::new(text);
        SelectorList::parse(&ProfileSelectorParser, &mut Parser::new(&mut input), ParseRelative::No)
            .map(Self).map_err(|e| format!("{e:?}"))
    }
    fn matches(&self, element: &ElementRef<'_>) -> bool {
        use selectors::matching::{MatchingContext, MatchingMode, NeedsSelectorFlags, MatchingForInvalidation, QuirksMode};
        let mut caches = Default::default();
        let mut context = MatchingContext::new(MatchingMode::Normal, None, &mut caches,
            QuirksMode::NoQuirks, NeedsSelectorFlags::No, MatchingForInvalidation::No);
        self.0.slice().iter().any(|s| selectors::matching::matches_selector(s, 0, None, element, &mut context))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Declaration {
    pub name: String,
    pub value: String,
    pub important: bool,
    pub source: String,
}

pub(crate) struct Rule {
    selector: ProfileSelector,
    specificity: (u32, u32, u32),
    declarations: Vec<Declaration>,
}

fn error(source: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("css-syntax", source, message)
}

// CSS Syntax tokenization handles comments, strings and escaped values. Blocks
// are rejected; only the explicitly supported, non-nested RGB/HSL functions pass.
fn value_text(p: &mut Parser<'_, '_>) -> Result<String, Diagnostic> {
    value_text_inner(p, true)
}

fn value_text_inner(p: &mut Parser<'_, '_>, allow_color: bool) -> Result<String, Diagnostic> {
    let mut text = String::new();
    while !p.is_exhausted() {
        let token = p
            .next_including_whitespace_and_comments()
            .map_err(|e| error("css", format!("{e:?}")))?;
        match token {
            Token::Comment(_) => text.push(' '),
            Token::Function(name)
                if allow_color
                    && ["rgb", "rgba", "hsl", "hsla"]
                        .iter()
                        .any(|function| name.eq_ignore_ascii_case(function)) =>
            {
                text.push_str(&name.to_ascii_lowercase());
                text.push('(');
                let start = p.position();
                let body = p
                    .parse_nested_block(|p| {
                        value_text_inner(p, false)
                            .map_err(|e| p.new_custom_error::<_, Diagnostic>(e))
                    })
                    .map_err(|e| error("css", format!("{e:?}")))?;
                // cssparser repairs an EOF-closed function; this profile rejects repairs.
                if !p.slice_from(start).ends_with(')') {
                    return Err(error("css", "Unclosed color function"));
                }
                text.push_str(&body);
                text.push(')');
            }
            Token::Function(_)
            | Token::ParenthesisBlock
            | Token::CurlyBracketBlock
            | Token::SquareBracketBlock => {
                return Err(Diagnostic::new(
                    "unsupported-css-value",
                    "css",
                    "Only non-nested rgb()/rgba()/hsl()/hsla() functions are supported",
                ));
            }
            Token::Number { value, .. } | Token::Dimension { value, .. } if !value.is_finite() => {
                return Err(error("css", "Non-finite number"));
            }
            Token::Percentage { unit_value, .. } if !unit_value.is_finite() => {
                return Err(error("css", "Non-finite percentage"));
            }
            token if token.is_parse_error() => return Err(error("css", "Invalid CSS token")),
            token => text.push_str(&token.to_css_string()),
        }
    }
    Ok(text.trim().to_owned())
}

fn consume_values<'i>(p: &mut Parser<'i, '_>, depth: usize) -> Result<(), cssparser::ParseError<'i, Diagnostic>> {
    if depth > 64 { return Err(p.new_custom_error(error("css", "Value nesting exceeds 64 levels"))); }
    while !p.is_exhausted() {
        let token = p.next_including_whitespace_and_comments()?.clone();
        if matches!(token, Token::Function(_) | Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock) {
            p.parse_nested_block(|p| consume_values(p, depth + 1))?;
        }
    }
    Ok(())
}

pub(crate) fn property_value(name: &str, text: &str) -> Result<String, Diagnostic> {
    if matches!(name, "background" | "background-image") && crate::gradient::starts_linear(text) {
        // Keep nested color syntax for the gradient parser after substitution.
        return Ok(text.trim().to_owned());
    }
    if name == "opacity" {
        let mut input = ParserInput::new(text);
        let mut parser = Parser::new(&mut input);
        if matches!(parser.next(), Ok(Token::Number { .. } | Token::Percentage { .. })) {
            // Clamp the authored f64 before narrowing. Generic f32 token
            // serialization can overflow or round a percentage to opaque.
            // Validation follows computed-value substitution invalidation.
            return Ok(text.trim().to_owned());
        }
    }

    if name == "aspect-ratio" {
        let mut input = ParserInput::new(text);
        let mut parser = Parser::new(&mut input);
        let mut has_numeric_component = false;
        while let Ok(token) = parser.next() {
            if matches!(token, Token::Function(_) | Token::Number { .. }) { has_numeric_component = true; break; }
        }
        if has_numeric_component {
            // Preserve numeric precision and math whitespace/comments. Generic
            // value serialization can round a component across Chrome's clamp.
            crate::aspect_ratio::AspectRatio::parse(text, "css")?;
            return Ok(text.trim().to_owned());
        }
    }
    ordinary_value(text)
}

pub(crate) fn ordinary_value(text: &str) -> Result<String, Diagnostic> {
    let mut input = ParserInput::new(text);
    value_text(&mut Parser::new(&mut input))
}

pub(crate) fn declarations(text: &str, source: &str) -> Result<Vec<Declaration>, Diagnostic> {
    let mut input = ParserInput::new(text);
    parse_declarations(&mut Parser::new(&mut input), source)
}

fn parse_declarations(
    p: &mut Parser<'_, '_>,
    source: &str,
) -> Result<Vec<Declaration>, Diagnostic> {
    let mut result = Vec::new();
    while !p.is_exhausted() {
        if p.try_parse(|p| p.expect_semicolon()).is_ok() {
            continue;
        }
        let loc = p.current_source_location();
        let here = format!("{source}:{}:{}", loc.line + 1, loc.column);
        let name = p
            .expect_ident()
            .map_err(|e| error(&here, format!("Expected property: {e:?}")))?
            .to_string();
        let name = if name.starts_with("--") { name } else { name.to_ascii_lowercase() };
        if name == "--" { return Err(error(&here, "Invalid custom property name")); }
        p.expect_colon().map_err(|_| error(&here, "Expected ':'"))?;
        let value = p
            .parse_until_before(Delimiter::Semicolon, |p| {
                let start = p.position();
                consume_values(p, 0)?;
                Ok::<_, cssparser::ParseError<'_, Diagnostic>>(p.slice_from(start).to_owned())
            })
            .map_err(|e| error(&here, format!("{e:?}")))?;
        let mut value_input = ParserInput::new(&value);
        let mut vp = Parser::new(&mut value_input);
        let mut bang = None;
        while !vp.is_exhausted() {
            let start = vp.position().byte_index();
            match vp.next() {
                Ok(Token::Delim('!')) => {
                    bang = Some(start);
                    break;
                }
                Ok(Token::Function(_)) => {
                    vp.parse_nested_block(|p| {
                        while !p.is_exhausted() {
                            p.next()?;
                        }
                        Ok::<_, cssparser::ParseError<'_, ()>>(())
                    })
                    .map_err(|_| error(&here, "Invalid function"))?;
                }
                _ => {}
            }
        }
        let (value, important) = if let Some(start) = bang {
            if vp.expect_ident_matching("important").is_err() || vp.expect_exhausted().is_err() {
                return Err(error(&here, "Invalid !important suffix"));
            }
            (value[..start].trim().to_owned(), true)
        } else {
            (value.clone(), false)
        };
        let parsed = crate::custom_properties::Value::parse(&value)?;
        let value = if name.starts_with("--") || parsed.has_variables() { value }
            else { property_value(&name, &value)? };
        if value.is_empty() && !name.starts_with("--") {
            return Err(error(&here, "Empty declaration"));
        }
        result.push(Declaration {
            name,
            value,
            important,
            source: here,
        });
        if result.len() > 256 {
            return Err(Diagnostic::new(
                "input-limit",
                source,
                "At most 256 declarations per block are supported",
            ));
        }
    }
    Ok(result)
}

fn attribute_selector(p: &mut Parser<'_, '_>) -> Result<String, Diagnostic> {
    let start = p.position();
    let name = p.expect_ident().map_err(|_| error("css", "Expected attribute name"))?;
    if !name.is_ascii() {
        return Err(error("css", "Attribute names must decode to ASCII"));
    }
    if !p.is_exhausted() {
        match p.next().map_err(|_| error("css", "Expected attribute operator"))? {
            Token::Delim('=') | Token::IncludeMatch | Token::DashMatch
            | Token::PrefixMatch | Token::SuffixMatch | Token::SubstringMatch => {}
            _ => return Err(error("css", "Unsupported attribute operator or namespace")),
        }
        match p.next().map_err(|_| error("css", "Expected attribute value"))? {
            Token::Ident(_) | Token::QuotedString(_) => {}
            _ => return Err(error("css", "Attribute values must be identifiers or strings")),
        }
        if !p.is_exhausted() {
            let flag = p.expect_ident().map_err(|_| error("css", "Expected attribute case flag"))?;
            if !flag.eq_ignore_ascii_case("i") {
                return Err(error("css", "Only the i attribute case flag is supported; Chrome does not support s"));
            }
        }
    }
    p.expect_exhausted().map_err(|_| error("css", "Unexpected attribute selector token"))?;
    Ok(p.slice_from(start).to_owned())
}

fn selector_prelude(p: &mut Parser<'_, '_>) -> Result<Vec<(String, (u32, u32, u32))>, Diagnostic> {
    selector_prelude_depth(p, 0)
}
fn selector_prelude_depth(p: &mut Parser<'_, '_>, depth: usize) -> Result<Vec<(String, (u32, u32, u32))>, Diagnostic> {
    selector_prelude_mode(p, depth, false)
}
fn selector_prelude_mode(p: &mut Parser<'_, '_>, depth: usize, forgiving: bool) -> Result<Vec<(String, (u32, u32, u32))>, Diagnostic> {
    if depth > 32 { return Err(Diagnostic::new("selector-limit", "css", "Selector nesting exceeds 32 levels")); }

    let mut selectors = Vec::new();
    let mut branches = 0;
    loop {
        let selector = p.parse_until_before(Delimiter::Comma, |p| {
            let mut text = String::new();
            let mut specificity = (0, 0, 0);
            let mut class = false;
            while !p.is_exhausted() {
                let token = p.next_including_whitespace_and_comments()?.clone();
                match &token {
                    Token::WhiteSpace(_) => text.push(' '),
                    Token::Comment(_) => text.push_str("/**/"),
                    Token::IDHash(name) if name.is_ascii() => {
                        specificity.0 += 1;
                        text.push_str(&token.to_css_string());
                    }
                    Token::Ident(name) if name.is_ascii() => {
                        if class { specificity.1 += 1; class = false; }
                        else { specificity.2 += 1; }
                        text.push_str(&token.to_css_string());
                    }
                    Token::Delim('.') => { class = true; text.push('.'); }
                    Token::Delim('*' | '>' | '+' | '~') => text.push_str(&token.to_css_string()),
                    Token::Colon => {
                        let token = p.next_including_whitespace()?.clone();
                        match token {
                            Token::Ident(name) => {
                                let name = name.to_ascii_lowercase();
                                if !matches!(name.as_str(), "first-child" | "last-child" | "only-child") {
                                    return Err(p.new_custom_error(Diagnostic::new("unsupported-selector", "css", "Unsupported pseudo-class")));
                                }
                                specificity.1 += 1;
                                text.push(':'); text.push_str(&name);
                            }
                            Token::Function(name) if ["not", "is", "where"].iter().any(|n| name.eq_ignore_ascii_case(n)) => {
                                let start = p.position();
                                let list = p.parse_nested_block(|p| selector_prelude_mode(p, depth + 1, !name.eq_ignore_ascii_case("not"))
                                    .map_err(|e| p.new_custom_error::<_, Diagnostic>(e)))?;
                                if !p.slice_from(start).ends_with(')') {
                                    return Err(p.new_custom_error(error("css", "Unclosed selector-list function")));
                                }
                                let extra = if name.eq_ignore_ascii_case("where") { (0, 0, 0) }
                                    else { list.iter().map(|(_, s)| *s).max().unwrap_or_default() };
                                specificity.0 += extra.0;
                                specificity.1 += extra.1;
                                specificity.2 += extra.2;
                                text.push(':'); text.push_str(&name.to_ascii_lowercase()); text.push('(');
                                text.push_str(&list.into_iter().map(|(s, _)| s).collect::<Vec<_>>().join(","));
                                text.push(')');
                            }
                            Token::Function(name) if name.eq_ignore_ascii_case("nth-child") || name.eq_ignore_ascii_case("nth-last-child") => {
                                let start = p.position();
                                let (body, extra) = p.parse_nested_block(|p| {
                                    let (a, b) = cssparser::parse_nth(p)?;
                                    if a.unsigned_abs() > 1_000_000 || b.unsigned_abs() > 1_000_000 {
                                        return Err(p.new_custom_error(Diagnostic::new("selector-limit", "css", "Nth coefficients must be within -1000000..1000000")));
                                    }
                                    let mut body = format!("{a}n{b:+}");
                                    let mut extra = (0, 0, 0);
                                    if !p.is_exhausted() {
                                        p.expect_ident_matching("of")?;
                                        let list = selector_prelude_depth(p, depth + 1)
                                            .map_err(|e| p.new_custom_error(e))?;
                                        extra = list.iter().map(|(_, s)| *s).max().unwrap_or_default();
                                        body.push_str(" of ");
                                        body.push_str(&list.into_iter().map(|(s, _)| s).collect::<Vec<_>>().join(","));
                                    }
                                    Ok::<_, cssparser::ParseError<'_, Diagnostic>>((body, extra))
                                })?;
                                if !p.slice_from(start).ends_with(')') {
                                    return Err(p.new_custom_error(error("css", "Unclosed nth-child function")));
                                }
                                specificity.0 += extra.0;
                                specificity.1 += 1 + extra.1;
                                specificity.2 += extra.2;
                                text.push(':'); text.push_str(&name.to_ascii_lowercase());
                                text.push('('); text.push_str(&body); text.push(')');
                            }
                            _ => return Err(p.new_custom_error(Diagnostic::new("unsupported-selector", "css", "Unsupported pseudo-class syntax"))),
                        }
                    }
                    Token::SquareBracketBlock => {
                        let start = p.position();
                        let body = p.parse_nested_block(|p| attribute_selector(p)
                            .map_err(|e| p.new_custom_error::<_, Diagnostic>(Diagnostic::new("unsupported-selector", "css", e.message))))?;
                        if !p.slice_from(start).ends_with(']') {
                            return Err(p.new_custom_error(error("css", "Unclosed attribute selector")));
                        }
                        specificity.1 += 1;
                        text.push('['); text.push_str(&body); text.push(']');
                    }
                    _ => return Err(p.new_custom_error(Diagnostic::new(
                        "unsupported-selector", "css", "Unsupported selector token"))),
                }
            }
            if text.trim().is_empty() {
                return Err(p.new_custom_error(error("css", "Empty selector")));
            }
            Ok((text, specificity))
        }).map_err(|e| match e.kind {
            cssparser::ParseErrorKind::Custom(diagnostic) => diagnostic,
            cssparser::ParseErrorKind::Basic(kind) => error("css", format!("{kind:?}")),
        });
        branches += 1;
        if branches > 1024 { return Err(Diagnostic::new("selector-limit", "css", "At most 1024 selectors per list are supported")); }
        match selector {
            Ok((text, specificity)) => match ProfileSelector::parse(&text) {
                Ok(_) => selectors.push((text, specificity)),
                Err(_) if forgiving => {},
                Err(message) => return Err(error("css", message)),
            },
            Err(e) if forgiving && e.code == "css-syntax" => {},
            Err(e) => return Err(e),
        }
        if p.is_exhausted() { break; }
        p.expect_comma().map_err(|_| error("css", "Expected selector comma"))?;
    }
    Ok(selectors)
}

pub(crate) fn stylesheet(text: &str) -> Result<Vec<Rule>, Diagnostic> {
    let mut input = ParserInput::new(text);
    let mut p = Parser::new(&mut input);
    let mut rules = Vec::new();
    while !p.is_exhausted() {
        let selector_list = p
            .parse_until_before(Delimiter::CurlyBracketBlock, |p| {
                selector_prelude(p).map_err(|e| p.new_custom_error::<_, Diagnostic>(e))
            })
            .map_err(|e| error("css", format!("{e:?}")))?;
        p.expect_curly_bracket_block().map_err(|_| {
            error(
                "css",
                "Expected a selector and declaration block; at-rules are unsupported",
            )
        })?;
        let declarations = p
            .parse_nested_block(|p| {
                parse_declarations(p, "css").map_err(|e| p.new_custom_error::<_, Diagnostic>(e))
            })
            .map_err(|e| error("css", format!("{e:?}")))?;
        for (selector_text, specificity) in selector_list {
            let selector = ProfileSelector::parse(&selector_text)
                .map_err(|e| error("css", format!("{e:?}")))?;
            rules.push(Rule {
                selector,
                specificity,
                declarations: declarations.clone(),
            });
            if rules.len() > 1024 {
                return Err(Diagnostic::new(
                    "input-limit",
                    "css",
                    "At most 1024 selectors are supported",
                ));
            }
        }
    }
    Ok(rules)
}

pub(crate) fn all_declarations(rules: &[Rule]) -> impl Iterator<Item = &Declaration> {
    rules.iter().flat_map(|r| &r.declarations)
}

pub(crate) fn cascade(
    rules: &[Rule],
    element: ElementRef<'_>,
) -> Result<Vec<Declaration>, Diagnostic> {
    let mut candidates = Vec::new();
    for rule in rules {
        if rule.selector.matches(&element) {
            for d in &rule.declarations {
                candidates.push(((d.important, false, rule.specificity), d.clone()));
            }
        }
    }
    if let Some(text) = element.attr("style") {
        for d in declarations(
            text,
            &format!(
                "html#{}@style",
                element.attr("id").unwrap_or(element.value().name())
            ),
        )? {
            candidates.push(((d.important, true, (0, 0, 0)), d));
        }
    }
    // Stable sorting preserves source order for equal specificity. Return all
    // declarations so shorthands participate at the same order as longhands.
    candidates.sort_by_key(|(priority, _)| *priority);
    Ok(candidates.into_iter().map(|(_, d)| d).collect())
}
