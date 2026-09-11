//! Specified linear-gradient values; responsive emission retains authored stop positions.
use crate::Diagnostic;
use cssparser::{ParseError, Parser, ParserInput, ToCss, Token};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Direction {
    Degrees(f32),
    Corner { right: bool, bottom: bool },
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Position {
    Pixels(f32),
    Percent(f32),
    Em(f32),
    Rem(f32),
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Color {
    Rgba(u32),
    CurrentColor,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Stop {
    pub color: Color,
    pub position: Option<Position>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LinearGradient {
    pub direction: Direction,
    pub stops: Vec<Stop>,
}

fn invalid(source: &str) -> Diagnostic {
    Diagnostic::new(
        "unsupported-linear-gradient",
        source,
        "Expected one linear-gradient with a direction and at least two color stops; hints, interpolation spaces and repetition are not yet supported",
    )
}

fn components<'i>(p: &mut Parser<'i, '_>) -> Result<Vec<String>, ParseError<'i, ()>> {
    let mut result = Vec::new();
    while !p.is_exhausted() {
        let start = p.position();
        let token = p.next()?.clone();
        result.push(match token {
            Token::Number { value, .. } | Token::Dimension { value, .. } if !value.is_finite() => {
                return Err(p.new_custom_error(()));
            }
            Token::Percentage { unit_value, .. } if !unit_value.is_finite() => {
                return Err(p.new_custom_error(()));
            }
            Token::Percentage { .. } => p.slice_from(start).trim().to_owned(),
            Token::Function(name)
                if ["rgb", "rgba", "hsl", "hsla"]
                    .iter()
                    .any(|v| name.eq_ignore_ascii_case(v)) =>
            {
                let body = p.parse_nested_block(|p| {
                    let mut body = String::new();
                    while let Ok(token) = p.next_including_whitespace_and_comments() {
                        match token {
                            Token::Function(_)
                            | Token::ParenthesisBlock
                            | Token::SquareBracketBlock
                            | Token::CurlyBracketBlock => return Err(p.new_custom_error(())),
                            Token::Comment(_) => body.push(' '),
                            _ => body.push_str(&token.to_css_string()),
                        }
                    }
                    Ok(body)
                })?;
                format!("{}({body})", name.to_ascii_lowercase())
            }
            Token::Ident(name) => name.to_string(),
            Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock => return Err(p.new_custom_error(())),
            other => other.to_css_string(),
        });
    }
    Ok(result)
}

fn position(value: &str) -> Option<Position> {
    let mut input = ParserInput::new(value);
    let mut p = Parser::new(&mut input);
    let value = match p.next().ok()? {
        Token::Percentage { unit_value, .. }
            if unit_value.is_finite() && (unit_value * 100.).is_finite() =>
        {
            // Preserve percentage points without the tokenizer's fraction round-trip.
            let points = value.trim().strip_suffix('%')?.parse::<f32>().ok()?;
            if !points.is_finite() { return None; }
            Position::Percent(points)
        }
        Token::Number { value: 0., .. } => Position::Pixels(0.),
        Token::Dimension { value, unit, .. } if value.is_finite() => {
            match unit.to_ascii_lowercase().as_str() {
                "px" => Position::Pixels(*value),
                "em" => Position::Em(*value),
                "rem" => Position::Rem(*value),
                _ => return None,
            }
        }
        _ => return None,
    };
    p.is_exhausted().then_some(value)
}

fn direction(values: &[String]) -> Option<Direction> {
    if values.first()?.eq_ignore_ascii_case("to") {
        let (mut x, mut y) = (None, None);
        for word in &values[1..] {
            match word.to_ascii_lowercase().as_str() {
                "left" if x.is_none() => x = Some(false),
                "right" if x.is_none() => x = Some(true),
                "top" if y.is_none() => y = Some(false),
                "bottom" if y.is_none() => y = Some(true),
                _ => return None,
            }
        }
        return match (x, y) {
            (Some(right), Some(bottom)) => Some(Direction::Corner { right, bottom }),
            (Some(right), None) => Some(Direction::Degrees(if right { 90. } else { 270. })),
            (None, Some(bottom)) => Some(Direction::Degrees(if bottom { 180. } else { 0. })),
            _ => None,
        };
    }
    if values.len() != 1 {
        return None;
    }
    let mut input = ParserInput::new(&values[0]);
    let mut p = Parser::new(&mut input);
    let degrees = match p.next().ok()? {
        Token::Number { value: 0., .. } => 0.,
        Token::Dimension { value, unit, .. } if value.is_finite() => {
            let factor = match unit.to_ascii_lowercase().as_str() {
                "deg" => 1.,
                "grad" => 0.9,
                "turn" => 360.,
                "rad" => 180. / std::f64::consts::PI,
                _ => return None,
            };
            (f64::from(*value) * factor).rem_euclid(360.) as f32
        }
        _ => return None,
    };
    Some(Direction::Degrees(degrees))
}

pub(crate) fn parse(value: &str, source: &str) -> Result<LinearGradient, Diagnostic> {
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    parser
        .expect_function_matching("linear-gradient")
        .map_err(|_| invalid(source))?;
    let groups = parser
        .parse_nested_block(|p| p.parse_comma_separated(components))
        .map_err(|_| invalid(source))?;
    if !parser.is_exhausted() || groups.is_empty() {
        return Err(invalid(source));
    }
    let specified_direction = direction(&groups[0]);
    let first_stop = usize::from(specified_direction.is_some());
    let mut stops = Vec::new();
    for group in &groups[first_stop..] {
        if group.is_empty() || group.len() > 3 {
            return Err(invalid(source));
        }
        let color = if group[0].eq_ignore_ascii_case("currentcolor") {
            Color::CurrentColor
        } else {
            Color::Rgba(crate::color(&group[0], source).map_err(|_| invalid(source))?)
        };
        if group.len() == 1 {
            stops.push(Stop {
                color,
                position: None,
            });
        } else {
            for value in &group[1..] {
                stops.push(Stop {
                    color: color.clone(),
                    position: Some(position(value).ok_or_else(|| invalid(source))?),
                });
            }
        }
        if stops.len() > 256 {
            return Err(Diagnostic::new(
                "gradient-stop-limit",
                source,
                "At most 256 expanded gradient stops are supported",
            ));
        }
    }
    if stops.len() < 2 {
        return Err(invalid(source));
    }
    Ok(LinearGradient {
        direction: specified_direction.unwrap_or(Direction::Degrees(180.)),
        stops,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_initial_browser_corpus() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../validation/linear-gradient-initial-cases.json"
        ))
        .unwrap();
        for case in cases.as_array().unwrap() {
            let css = case["css"].as_str().unwrap();
            let value = css
                .split("background:linear-gradient")
                .nth(1)
                .unwrap()
                .split(';')
                .next()
                .unwrap();
            assert!(
                parse(&format!("linear-gradient{value}"), "gradient").is_ok(),
                "{}",
                case["name"]
            );
        }
    }
    #[test]
    fn directions_colors_and_expanded_stops() {
        let result = parse(
            "linear-gradient(to bottom right, currentColor -1em 20%, rgba(0, 0, 255, .5) 2rem)",
            "g",
        )
        .unwrap();
        assert_eq!(
            result.direction,
            Direction::Corner {
                right: true,
                bottom: true
            }
        );
        assert_eq!(result.stops.len(), 3);
        assert_eq!(
            result.stops[0],
            Stop {
                color: Color::CurrentColor,
                position: Some(Position::Em(-1.))
            }
        );
        assert_eq!(result.stops[2].color, Color::Rgba(0x800000ff));
        assert_eq!(
            parse("LINEAR-GRADIENT(/*a*/.25turn, red, hsl(240 100% 50%))", "g")
                .unwrap()
                .direction,
            Direction::Degrees(90.)
        );
    }
    #[test]
    fn rejects_unimplemented_or_invalid_components() {
        for value in [
            "linear-gradient(red)",
            "linear-gradient(red,)",
            "linear-gradient(to right left,red,blue)",
            "linear-gradient(red 2,blue)",
            "linear-gradient(red calc(2px),blue)",
            "linear-gradient(red,40%,blue)",
            "linear-gradient(in oklab,red,blue)",
            "repeating-linear-gradient(red,blue)",
            "linear-gradient(red,blue),linear-gradient(red,blue)",
            "linear-gradient(1e999deg,red,blue)",
        ] {
            assert!(parse(value, "g").is_err(), "{value}");
        }
        let value = format!("linear-gradient({})", vec!["red"; 257].join(","));
        assert_eq!(parse(&value, "g").unwrap_err().code, "gradient-stop-limit");
    }
}

pub(crate) fn starts_linear(value: &str) -> bool {
    let mut input = ParserInput::new(value);
    Parser::new(&mut input).expect_function_matching("linear-gradient").is_ok()
}
