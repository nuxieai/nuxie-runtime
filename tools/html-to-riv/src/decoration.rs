//! CSS decoration values and propagated origins; distinct from inheritance.
use crate::{Diagnostic, SolidUnderline, UnderlineSkipInk, color};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Metric {
    Auto,
    FromFont,
    Px(f32),
    Percent(f32),
}
impl Metric {
    pub fn parse(
        value: &str,
        font_size: f32,
        offset: bool,
        source: &str,
    ) -> Result<Self, Diagnostic> {
        let fail = || {
            Diagnostic::new(
                "unsupported-value",
                source,
                "Expected auto, a bounded px/em/rem length or percentage; from-font is supported only for thickness",
            )
        };
        let mut input = cssparser::ParserInput::new(value);
        let mut p = cssparser::Parser::new(&mut input);
        let parsed = match p.next().map_err(|_| fail())? {
            cssparser::Token::Ident(v) if v.eq_ignore_ascii_case("auto") => Self::Auto,
            cssparser::Token::Ident(v) if !offset && v.eq_ignore_ascii_case("from-font") => {
                Self::FromFont
            }
            cssparser::Token::Number { value, .. } if *value == 0.0 => Self::Px(0.0),
            cssparser::Token::Percentage { unit_value, .. } => Self::Percent(*unit_value),
            cssparser::Token::Dimension { value, unit, .. } => {
                let basis = if unit.eq_ignore_ascii_case("px") {
                    1.0
                } else if unit.eq_ignore_ascii_case("em") {
                    font_size
                } else if unit.eq_ignore_ascii_case("rem") {
                    16.0
                } else {
                    return Err(fail());
                };
                Self::Px(*value * basis)
            }
            _ => return Err(fail()),
        };
        p.expect_exhausted().map_err(|_| fail())?;
        if let Self::Px(v) | Self::Percent(v) = parsed
            && (!v.is_finite() || v.abs() > 1_000_000.0 || (!offset && v < 0.0))
        {
            return Err(fail());
        }
        Ok(parsed)
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Decoration {
    pub underline: bool,
    pub strikethrough: bool,
    pub color: Option<u32>,
    pub thickness: Metric,
    pub offset: Metric,
    pub skip_ink: UnderlineSkipInk,
}
impl Default for Decoration {
    fn default() -> Self {
        Self {
            underline: false,
            strikethrough: false,
            color: None,
            thickness: Metric::Auto,
            offset: Metric::Auto,
            skip_ink: UnderlineSkipInk::Auto,
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Origin {
    pub underline: bool,
    pub strikethrough: bool,
    pub color: u32,
    pub thickness: Metric,
    pub offset: Metric,
}
impl Decoration {
    pub fn apply(
        &mut self,
        name: &str,
        value: &str,
        font_size: f32,
        source: &str,
    ) -> Result<bool, Diagnostic> {
        let bad = || {
            Diagnostic::new(
                "unsupported-value",
                source,
                "Only solid underline and line-through decorations are supported",
            )
        };
        match name {
            "text-decoration-line" => {
                let mut underline = false;
                let mut strike = false;
                if value != "none" {
                    for word in value.split_whitespace() {
                        match word {
                            "underline" if !underline => underline = true,
                            "line-through" if !strike => strike = true,
                            _ => return Err(bad()),
                        }
                    }
                    if !underline && !strike {
                        return Err(bad());
                    }
                }
                self.underline = underline;
                self.strikethrough = strike;
            }
            "text-decoration-color" => {
                self.color = if value == "currentcolor" {
                    None
                } else {
                    Some(color(value, source)?)
                }
            }
            "text-decoration-style" => {
                if value != "solid" {
                    return Err(bad());
                }
            }
            "text-decoration-thickness" => {
                self.thickness = Metric::parse(value, font_size, false, source)?
            }
            "text-underline-offset" => self.offset = Metric::parse(value, font_size, true, source)?,
            "text-decoration-skip-ink" => {
                self.skip_ink = match value {
                    "none" => UnderlineSkipInk::None,
                    "auto" => UnderlineSkipInk::Auto,
                    "all" => UnderlineSkipInk::All,
                    _ => return Err(bad()),
                }
            }
            "text-underline-position" => {
                if value != "auto" {
                    return Err(bad());
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
    pub fn origin(&self, color: u32) -> Option<Origin> {
        (self.underline || self.strikethrough).then_some(Origin {
            underline: self.underline,
            strikethrough: self.strikethrough,
            color: self.color.unwrap_or(color),
            thickness: self.thickness,
            offset: self.offset,
        })
    }
}
impl Origin {
    pub fn resolve_strikethrough(
        &self,
        size: f32,
        font_thickness: Option<f32>,
        ascent: f32,
        line_baseline: f32,
        source: &str,
    ) -> Result<crate::SolidStrikethrough, Diagnostic> {
        let mut metric_origin = self.clone();
        metric_origin.offset = Metric::Auto;
        let metric = metric_origin.resolve(size, font_thickness, UnderlineSkipInk::None, source)?;
        let offset = -(ascent * size).round() / 3.0 - metric.thickness / 2.0;
        Ok(crate::SolidStrikethrough {
            color: metric.color,
            thickness: metric.thickness,
            offset,
            line_baseline,
        })
    }

    pub fn resolve(
        &self,
        font_size: f32,
        font_thickness: Option<f32>,
        skip_ink: UnderlineSkipInk,
        source: &str,
    ) -> Result<SolidUnderline, Diagnostic> {
        let thickness = match self.thickness {
            Metric::Auto => font_size / 10.0,
            Metric::FromFont => font_thickness
                .map(|v| v * font_size)
                .unwrap_or(font_size / 10.0),
            Metric::Px(v) => v.round(),
            Metric::Percent(v) => (v * font_size).round(),
        }
        .max(1.0);
        let offset = match self.offset {
            Metric::Auto => (thickness / 2.0).ceil().max(1.0),
            Metric::Px(v) => v.round(),
            Metric::Percent(v) => (v * font_size).round(),
            Metric::FromFont => unreachable!("offset parser rejects from-font"),
        };
        if !thickness.is_finite()
            || thickness > 1_000_000.0
            || !offset.is_finite()
            || offset.abs() > 1_000_000.0
        {
            return Err(Diagnostic::new(
                "unsupported-value",
                source,
                "Resolved underline metric exceeds 1000000px",
            ));
        }
        Ok(SolidUnderline {
            color: self.color,
            thickness,
            offset,
            skip_ink,
        })
    }
}

pub(crate) fn expand(
    d: &crate::css::Declaration,
) -> Result<Vec<crate::css::Declaration>, Diagnostic> {
    if d.name != "text-decoration" {
        return Ok(vec![d.clone()]);
    }
    let mut normalized = d.clone();
    normalized.value.make_ascii_lowercase();
    let d = &normalized;
    let mut values = [
        "none".to_owned(),
        "solid".to_owned(),
        "currentcolor".to_owned(),
        "auto".to_owned(),
    ];
    if ["inherit", "initial", "unset"].contains(&d.value.as_str()) {
        values.fill(d.value.clone());
    } else {
        let bad = || {
            Diagnostic::new(
                "unsupported-text-decoration",
                &d.source,
                "Expected optional none/underline/line-through, solid, color and thickness; duplicate components and other line styles are unsupported",
            )
        };
        let mut input = cssparser::ParserInput::new(&d.value);
        let mut parser = cssparser::Parser::new(&mut input);
        let mut seen = [false; 4];
        while !parser.is_exhausted() {
            let start = parser.position();
            let token = parser.next().map_err(|_| bad())?.clone();
            if matches!(token, cssparser::Token::Function(_)) {
                parser
                    .parse_nested_block(|p| {
                        while !p.is_exhausted() {
                            p.next()?;
                        }
                        Ok::<_, cssparser::ParseError<'_, ()>>(())
                    })
                    .map_err(|_| bad())?;
            }
            let value = parser.slice_from(start).trim();
            let slot = if matches!(value, "none" | "underline" | "line-through") {
                0
            } else if value == "solid" {
                1
            } else if value == "currentcolor" || color(value, &d.source).is_ok() {
                2
            } else if Metric::parse(value, 1.0, false, &d.source).is_ok() {
                3
            } else {
                return Err(bad());
            };
            if seen[slot] {
                if slot == 0
                    && value != "none"
                    && values[0] != "none"
                    && !values[0].split_whitespace().any(|v| v == value)
                {
                    values[0].push(' ');
                    values[0].push_str(value);
                    continue;
                }
                return Err(bad());
            }
            seen[slot] = true;
            values[slot] = value.to_owned();
        }
        if !seen.iter().any(|v| *v) {
            return Err(bad());
        }
    }
    Ok([
        "text-decoration-line",
        "text-decoration-style",
        "text-decoration-color",
        "text-decoration-thickness",
    ]
    .into_iter()
    .zip(values)
    .map(|(name, value)| crate::css::Declaration {
        name: name.into(),
        value,
        ..d.clone()
    })
    .collect())
}
