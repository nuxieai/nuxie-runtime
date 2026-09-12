//! Receiving-value grammar for variable recovery, independent of Rive admission.
//! Only definite invalidity becomes unset. Opaque functions and grammar outside
//! these primitive productions continue through the normal target diagnostics.
use cssparser::{Parser, ParserInput, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Validity {
    Valid,
    Invalid,
    Unclassified,
}
use Validity::{Invalid, Unclassified, Valid};

struct Component<'i> {
    token: Token<'i>,
    // Match the pinned browser's numeric parsing precision before the existing
    // target narrows values to binary32. Tiny nonzero numbers must not become
    // unitless zeros or lose a prohibited negative sign through f32 underflow.
    number: Option<f64>,
}

const PROPERTIES: &[&str] = &[
    "width",
    "height",
    "min-width",
    "min-height",
    "max-width",
    "max-height",
    "box-sizing",
    "font-size",
    "background",
    "background-color",
    "color",
    "display",
    "flex-direction",
    "flex",
    "flex-grow",
    "flex-shrink",
    "flex-basis",
    "order",
    "align-self",
    "justify-content",
    "margin",
    "margin-left",
    "margin-top",
    "margin-right",
    "margin-bottom",
    "padding",
    "padding-left",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "gap",
    "row-gap",
    "column-gap",
];

pub(super) fn classify(property: &str, text: &str) -> Validity {
    if !PROPERTIES.contains(&property) {
        return Unclassified;
    }
    let mut input = ParserInput::new(text);
    let mut parser = Parser::new(&mut input);
    let mut tokens = Vec::new();
    while !parser.is_exhausted() {
        parser.skip_whitespace();
        let start = parser.position();
        let Ok(token) = parser.next().cloned() else {
            return Unclassified;
        };
        // Complete variable expansion already checked syntax and resource bounds.
        // A function's arguments must be parsed against its own grammar; neither
        // its name nor failure in a limited target parser proves invalidity.
        if matches!(&token, Token::Function(_)) {
            return Unclassified;
        }
        if matches!(&token, Token::Number { value, .. } | Token::Dimension { value, .. } if !value.is_finite())
            || matches!(&token, Token::Percentage { unit_value, .. } if !unit_value.is_finite())
        {
            return Unclassified;
        }
        let number = if matches!(
            token,
            Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. }
        ) {
            let raw = crate::numeric_tokens::number_prefix(parser.slice_from(start));
            let Ok(number) = raw.parse::<f64>() else {
                return Unclassified;
            };
            if !number.is_finite() {
                return Unclassified;
            }
            Some(number)
        } else {
            None
        };
        tokens.push(Component { token, number });
    }
    if tokens.is_empty() {
        return Invalid;
    }
    if tokens.iter().any(css_wide) {
        return if tokens.len() == 1 { Valid } else { Invalid };
    }
    // None of these properties accepts free-standing strings or blocks, or
    // custom identifiers containing whitespace/control characters. Do not apply
    // this rule to custom-property names, strings/URLs inside functions, or HTML.
    if tokens.iter().any(|t| match &t.token {
        Token::QuotedString(_)
        | Token::ParenthesisBlock
        | Token::SquareBracketBlock
        | Token::CurlyBracketBlock
        | Token::BadString(_)
        | Token::BadUrl(_) => true,
        Token::Ident(s) | Token::Dimension { unit: s, .. } => {
            s.chars().any(|c| c.is_whitespace() || c.is_control())
        }
        Token::Delim(c) => c.is_whitespace() || c.is_control(),
        _ => false,
    }) {
        return Invalid;
    }
    let one = |f: fn(&Component<'_>) -> Validity| {
        if tokens.len() == 1 {
            f(&tokens[0])
        } else {
            Invalid
        }
    };
    match property {
        "width" | "height" | "min-width" | "min-height" | "max-width" | "max-height"
        | "font-size" | "flex-basis" => {
            if tokens.len() != 1 {
                Invalid
            } else {
                size(property, &tokens[0])
            }
        }
        "order" => one(|t| {
            if matches!(
                &t.token,
                Token::Number {
                    int_value: Some(_),
                    ..
                }
            ) {
                Valid
            } else {
                Invalid
            }
        }),
        "flex-grow" | "flex-shrink" => one(factor),
        "box-sizing" => one(|t| keyword(t, &["border-box", "content-box"])),
        "flex-direction" => {
            one(|t| keyword(t, &["row", "row-reverse", "column", "column-reverse"]))
        }
        "color" | "background-color" => one(color),
        "background" => background(&tokens),
        "margin" | "margin-left" | "margin-top" | "margin-right" | "margin-bottom" => {
            components(&tokens, if property == "margin" { 4 } else { 1 }, |t| {
                if ident(t, "auto") {
                    Valid
                } else {
                    length(t, false)
                }
            })
        }
        "padding" | "padding-left" | "padding-top" | "padding-right" | "padding-bottom" => {
            components(&tokens, if property == "padding" { 4 } else { 1 }, |t| {
                length(t, true)
            })
        }
        "gap" | "row-gap" | "column-gap" => {
            components(&tokens, if property == "gap" { 2 } else { 1 }, |t| {
                if ident(t, "normal") {
                    Valid
                } else {
                    length(t, true)
                }
            })
        }
        "flex" => flex(&tokens),
        "display" => display(&tokens),
        "align-self" | "justify-content" => alignment(property, &tokens),
        _ => Unclassified,
    }
}

fn ident(t: &Component<'_>, expected: &str) -> bool {
    matches!(&t.token, Token::Ident(s) if s.eq_ignore_ascii_case(expected))
}
fn css_wide(t: &Component<'_>) -> bool {
    [
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
        "revert-rule",
    ]
    .iter()
    .any(|s| ident(t, s))
}
fn keyword(t: &Component<'_>, allowed: &[&str]) -> Validity {
    match &t.token {
        Token::Ident(s) if allowed.iter().any(|a| s.eq_ignore_ascii_case(a)) => Valid,
        // Vendor extensions are target diagnostics, not recovery opportunities.
        Token::Ident(s) if s.starts_with('-') => Unclassified,
        _ => Invalid,
    }
}
fn components(
    tokens: &[Component<'_>],
    max: usize,
    f: impl Fn(&Component<'_>) -> Validity,
) -> Validity {
    if tokens.is_empty() || tokens.len() > max {
        return Invalid;
    }
    let mut result = Valid;
    for t in tokens {
        match f(t) {
            Invalid => return Invalid,
            Unclassified => result = Unclassified,
            Valid => (),
        }
    }
    result
}
fn length(t: &Component<'_>, nonnegative: bool) -> Validity {
    match &t.token {
        Token::Number { .. } => {
            if t.number == Some(0.) {
                Valid
            } else {
                Invalid
            }
        }
        Token::Percentage { .. } => {
            if nonnegative && t.number.is_some_and(|v| v < 0.) {
                Invalid
            } else {
                Valid
            }
        }
        Token::Dimension { unit, .. } => {
            let unit = unit.to_ascii_lowercase();
            if [
                "px", "em", "rem", "ex", "rex", "ch", "rch", "ic", "ric", "cap", "rcap", "lh",
                "rlh", "vw", "vh", "vi", "vb", "vmin", "vmax", "svw", "svh", "svi", "svb", "svmin",
                "svmax", "lvw", "lvh", "lvi", "lvb", "lvmin", "lvmax", "dvw", "dvh", "dvi", "dvb",
                "dvmin", "dvmax", "cqw", "cqh", "cqi", "cqb", "cqmin", "cqmax", "cm", "mm", "q",
                "in", "pt", "pc",
            ]
            .contains(&unit.as_str())
            {
                if nonnegative && t.number.is_some_and(|v| v < 0.) {
                    Invalid
                } else {
                    Valid
                }
            } else if [
                "deg", "grad", "rad", "turn", "s", "ms", "hz", "khz", "dpi", "dpcm", "dppx", "x",
                "fr",
            ]
            .contains(&unit.as_str())
            {
                Invalid
            } else {
                Unclassified
            }
        }
        _ => Invalid,
    }
}
fn size(property: &str, t: &Component<'_>) -> Validity {
    if matches!(&t.token, Token::Ident(_)) {
        if property == "font-size" {
            return keyword(
                t,
                &[
                    "xx-small",
                    "x-small",
                    "small",
                    "medium",
                    "large",
                    "x-large",
                    "xx-large",
                    "xxx-large",
                    "larger",
                    "smaller",
                    "math",
                ],
            );
        }
        if ident(t, "auto") {
            return if property.starts_with("max-") {
                Invalid
            } else {
                Valid
            };
        }
        if ident(t, "none") {
            return if property.starts_with("max-") {
                Valid
            } else {
                Invalid
            };
        }
        if property == "flex-basis" && ident(t, "content") {
            return Valid;
        }
        return keyword(
            t,
            &[
                "min-content",
                "max-content",
                "fit-content",
                "stretch",
                "contain",
            ],
        );
    }
    length(t, true)
}
fn factor(t: &Component<'_>) -> Validity {
    if matches!(&t.token, Token::Number { .. } if t.number.is_some_and(|v| v >= 0.)) {
        Valid
    } else {
        Invalid
    }
}
fn flex(tokens: &[Component<'_>]) -> Validity {
    if tokens.len() == 1 && ident(&tokens[0], "none") {
        return Valid;
    }
    let basis = |t: &Component<'_>| size("flex-basis", t);
    let alternatives: Vec<Vec<Validity>> = match tokens {
        [a] => vec![vec![factor(a)], vec![basis(a)]],
        [a, b] => vec![
            vec![factor(a), factor(b)],
            vec![factor(a), basis(b)],
            vec![basis(a), factor(b)],
        ],
        [a, b, c] => vec![
            vec![factor(a), factor(b), basis(c)],
            vec![basis(a), factor(b), factor(c)],
        ],
        _ => return Invalid,
    };
    if alternatives.iter().any(|a| a.iter().all(|v| *v == Valid)) {
        Valid
    } else if alternatives.iter().any(|a| !a.contains(&Invalid)) {
        Unclassified
    } else {
        Invalid
    }
}
fn color(t: &Component<'_>) -> Validity {
    match &t.token {
        Token::Hash(s) | Token::IDHash(s) => {
            if [3, 4, 6, 8].contains(&s.len()) && s.bytes().all(|c| c.is_ascii_hexdigit()) {
                Valid
            } else {
                Invalid
            }
        }
        Token::Ident(s) if cssparser::color::parse_named_color(&s.to_ascii_lowercase()).is_ok() => {
            Valid
        }
        Token::Ident(_) => keyword(
            t,
            &[
                "transparent",
                "currentcolor",
                "accentcolor",
                "accentcolortext",
                "activetext",
                "buttonborder",
                "buttonface",
                "buttontext",
                "canvas",
                "canvastext",
                "field",
                "fieldtext",
                "graytext",
                "highlight",
                "highlighttext",
                "linktext",
                "mark",
                "marktext",
                "selecteditem",
                "selecteditemtext",
                "visitedtext",
                "activeborder",
                "activecaption",
                "appworkspace",
                "background",
                "buttonhighlight",
                "buttonshadow",
                "captiontext",
                "inactiveborder",
                "inactivecaption",
                "inactivecaptiontext",
                "infobackground",
                "infotext",
                "menu",
                "menutext",
                "scrollbar",
                "threeddarkshadow",
                "threedface",
                "threedhighlight",
                "threedlightshadow",
                "threedshadow",
                "window",
                "windowframe",
                "windowtext",
            ],
        ),
        _ => Invalid,
    }
}
fn background(tokens: &[Component<'_>]) -> Validity {
    // Multi-layer shorthand grammar is deliberately not inferred from the
    // current solid-color lowering. A position alone is valid CSS too.
    if tokens.len() != 1 {
        return Unclassified;
    }
    let t = &tokens[0];
    match &t.token {
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
            length(t, false)
        }
        Token::UnquotedUrl(_) => Valid,
        Token::Ident(_)
            if keyword(
                t,
                &[
                    "none",
                    "left",
                    "right",
                    "top",
                    "bottom",
                    "center",
                    "repeat",
                    "repeat-x",
                    "repeat-y",
                    "no-repeat",
                    "space",
                    "round",
                    "scroll",
                    "fixed",
                    "local",
                    "border-box",
                    "padding-box",
                    "content-box",
                    "text",
                    "border-area",
                ],
            ) == Valid =>
        {
            Valid
        }
        _ => color(t),
    }
}
fn display(tokens: &[Component<'_>]) -> Validity {
    if tokens.len() == 1 {
        return keyword(
            &tokens[0],
            &[
                "none",
                "contents",
                "block",
                "inline",
                "run-in",
                "flow",
                "flow-root",
                "table",
                "flex",
                "grid",
                "ruby",
                "math",
                "list-item",
                "inline-block",
                "inline-table",
                "inline-flex",
                "inline-grid",
                "table-row-group",
                "table-header-group",
                "table-footer-group",
                "table-row",
                "table-cell",
                "table-column-group",
                "table-column",
                "table-caption",
                "ruby-base",
                "ruby-text",
                "ruby-base-container",
                "ruby-text-container",
            ],
        );
    }
    if tokens.len() > 3 || tokens.iter().any(|t| !matches!(&t.token, Token::Ident(_))) {
        Invalid
    } else {
        Unclassified
    }
}
fn alignment(property: &str, tokens: &[Component<'_>]) -> Validity {
    let positional = if property == "align-self" {
        &[
            "center",
            "start",
            "end",
            "self-start",
            "self-end",
            "flex-start",
            "flex-end",
        ][..]
    } else {
        &[
            "center",
            "start",
            "end",
            "flex-start",
            "flex-end",
            "left",
            "right",
        ][..]
    };
    match tokens {
        [a] => {
            let special = if property == "align-self" {
                &[
                    "auto",
                    "normal",
                    "stretch",
                    "baseline",
                    "anchor-center",
                    "dialog",
                ][..]
            } else {
                &[
                    "normal",
                    "stretch",
                    "space-between",
                    "space-around",
                    "space-evenly",
                ][..]
            };
            let p = keyword(a, positional);
            if p == Valid {
                Valid
            } else {
                keyword(a, special)
            }
        }
        [a, b] if ident(a, "safe") || ident(a, "unsafe") => {
            // Current Alignment draft permits overflow-position with normal,
            // while pinned Chrome rejects it. Retain a diagnostic, not recovery.
            if property == "align-self" && ident(b, "normal") {
                Unclassified
            } else {
                keyword(b, positional)
            }
        }
        [a, b]
            if property == "align-self"
                && (ident(a, "first") || ident(a, "last"))
                && ident(b, "baseline") =>
        {
            Valid
        }
        _ => Invalid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grammar_validity_does_not_depend_on_target_or_resource_admission() {
        for (p, v) in [
            ("width", "1000001px"),
            ("order", "2147483648"),
            ("width", "1vw"),
            ("width", "1cm"),
            ("margin", "-1px"),
            ("background", "20px"),
            ("color", "CanvasText"),
            ("font-size", "larger"),
            ("display", "grid"),
            ("flex", "1 1 0px"),
            ("width", "revert-layer"),
            ("width", "revert-rule"),
        ] {
            assert_eq!(classify(p, v), Valid, "{p}:{v}");
        }
        for (p, v) in [
            ("width", "calc(1px + 2px)"),
            ("color", "lab(50% 0 0)"),
            ("width", "mystery(1px)"),
            ("background", "left top red"),
            ("width", "1e999px"),
            ("width", "1newunit"),
            ("--custom", ""),
        ] {
            assert_eq!(classify(p, v), Unclassified, "{p}:{v}");
        }
    }
}
