//! Compiler-owned CSS sRGB literals, lowered to straight-alpha 0xAARRGGBB.
//! currentColor and inheritance are resolved by computed style, not this parser.
//! Restored from 20248ee6 without runtime or renderer dependencies.
use crate::Diagnostic;

pub(crate) fn parse(value: &str, source: &str) -> Result<u32, Diagnostic> {
    let value = value.trim().to_ascii_lowercase();
    if value == "transparent" {
        return Ok(0);
    }
    if let Some(body) = value
        .strip_prefix("rgb(")
        .or_else(|| value.strip_prefix("rgba("))
    {
        return rgb_color(body.strip_suffix(')').unwrap_or(""), source);
    }
    if let Some(body) = value
        .strip_prefix("hsl(")
        .or_else(|| value.strip_prefix("hsla("))
    {
        return hsl_color(body.strip_suffix(')').unwrap_or(""), source);
    }
    if let Ok((r, g, b)) = cssparser::color::parse_named_color(&value) {
        return Ok(0xff000000 | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b));
    }
    let hex = value.strip_prefix('#').unwrap_or("");
    let expanded = match hex.len() {
        3 | 4 => hex.chars().flat_map(|c| [c, c]).collect::<String>(),
        6 | 8 => hex.into(),
        _ => {
            return Err(Diagnostic::new(
                "unsupported-color",
                source,
                "Expected a named sRGB color, hex, rgb(), rgba(), hsl(), hsla() or transparent",
            ));
        }
    };
    let v = u32::from_str_radix(&expanded, 16)
        .map_err(|_| Diagnostic::new("unsupported-color", source, "Invalid hexadecimal color"))?;
    Ok(if expanded.len() == 6 {
        0xff000000 | v
    } else {
        v.rotate_right(8)
    })
}

// RGB is lowered to the format's 8-bit sRGB channels, including alpha.
fn rgb_color(body: &str, source: &str) -> Result<u32, Diagnostic> {
    use cssparser::{Parser, ParserInput, Token};
    let invalid = || {
        Diagnostic::new(
            "unsupported-color",
            source,
            "Expected numeric/percentage rgb() or rgba(); no mixed legacy units, missing channels or nested functions",
        )
    };
    let component = |text: &str| -> Result<(f32, bool), Diagnostic> {
        let mut input = ParserInput::new(text.trim());
        let mut parser = Parser::new(&mut input);
        let (value, percentage) = match parser.next().map_err(|_| invalid())? {
            Token::Number { value, .. } => (*value, false),
            Token::Percentage { unit_value, .. } => (*unit_value, true),
            _ => return Err(invalid()),
        };
        if !value.is_finite() || parser.expect_exhausted().is_err() {
            return Err(invalid());
        }
        Ok((value, percentage))
    };
    let legacy = body.contains(',');
    let (channels, alpha): (Vec<&str>, Option<&str>) = if legacy {
        let parts: Vec<_> = body.split(',').collect();
        match parts.as_slice() {
            [r, g, b] => (vec![r, g, b], None),
            [r, g, b, a] => (vec![r, g, b], Some(a)),
            _ => return Err(invalid()),
        }
    } else {
        let mut parts = body.split('/');
        let channels = parts
            .next()
            .unwrap_or("")
            .split_ascii_whitespace()
            .collect();
        let alpha = parts.next();
        if parts.next().is_some() {
            return Err(invalid());
        }
        (channels, alpha)
    };
    if channels.len() != 3 {
        return Err(invalid());
    }
    let channels = channels
        .into_iter()
        .map(component)
        .collect::<Result<Vec<_>, _>>()?;
    if legacy && channels.iter().any(|c| c.1 != channels[0].1) {
        return Err(invalid());
    }
    let byte = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u32;
    let alpha = alpha
        .map(component)
        .transpose()?
        .map_or(255, |(v, _)| byte(v));
    let mut result = alpha << 24;
    for ((value, percentage), shift) in channels.into_iter().zip([16, 8, 0]) {
        result |= byte(if percentage { value } else { value / 255.0 }) << shift;
    }
    Ok(result)
}

fn hsl_color(body: &str, source: &str) -> Result<u32, Diagnostic> {
    use cssparser::{Parser, ParserInput, Token};
    let invalid = || {
        Diagnostic::new(
            "unsupported-color",
            source,
            "Expected hsl()/hsla() with a hue, saturation, lightness and optional alpha; no missing components or nested functions",
        )
    };
    let legacy = body.contains(',');
    let (parts, alpha): (Vec<&str>, Option<&str>) = if legacy {
        let parts: Vec<_> = body.split(',').collect();
        match parts.as_slice() {
            [h, s, l] => (vec![h, s, l], None),
            [h, s, l, a] => (vec![h, s, l], Some(a)),
            _ => return Err(invalid()),
        }
    } else {
        let mut slash = body.split('/');
        let parts = slash
            .next()
            .unwrap_or("")
            .split_ascii_whitespace()
            .collect();
        let alpha = slash.next();
        if slash.next().is_some() {
            return Err(invalid());
        }
        (parts, alpha)
    };
    if parts.len() != 3 {
        return Err(invalid());
    }
    let component = |text: &str, index: usize| -> Result<f64, Diagnostic> {
        let mut input = ParserInput::new(text.trim());
        let mut parser = Parser::new(&mut input);
        let value = match parser.next().map_err(|_| invalid())? {
            Token::Number { value, .. } if index == 0 || index == 3 => f64::from(*value),
            Token::Number { value, .. } if !legacy => f64::from(*value) / 100.0,
            Token::Percentage { unit_value, .. } if index != 0 => f64::from(*unit_value),
            Token::Dimension { value, unit, .. } if index == 0 => {
                let value = f64::from(*value);
                match unit.to_ascii_lowercase().as_str() {
                    "deg" => value,
                    "grad" => value * 0.9,
                    "turn" => value * 360.0,
                    "rad" => value.to_degrees(),
                    _ => return Err(invalid()),
                }
            }
            _ => return Err(invalid()),
        };
        if !value.is_finite() || parser.expect_exhausted().is_err() {
            return Err(invalid());
        }
        Ok(value)
    };
    let hue = component(parts[0], 0)?.rem_euclid(360.0) / 30.0;
    let saturation = component(parts[1], 1)?.clamp(0.0, 1.0);
    let lightness = component(parts[2], 2)?.clamp(0.0, 1.0);
    let alpha = alpha.map(|a| component(a, 3)).transpose()?.unwrap_or(1.0);
    let amplitude = saturation * lightness.min(1.0 - lightness);
    let byte = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
    let channel = |n: f64| {
        let k = (n + hue) % 12.0;
        byte(lightness - amplitude * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0))
    };
    Ok((byte(alpha) << 24) | (channel(0.0) << 16) | (channel(8.0) << 8) | channel(4.0))
}
