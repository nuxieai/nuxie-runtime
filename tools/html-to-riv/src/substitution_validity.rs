//! Conservative computed-value invalidation. Valid CSS outside the rendering
//! profile must retain its diagnostic, so unclassified syntax is not invalidated.
use cssparser::{Parser, ParserInput, Token, ToCss};

fn wide_keyword(value: &str) -> bool {
    matches!(value, "initial" | "inherit" | "unset" | "revert" | "revert-layer")
}
// These CSS dimension types cannot occupy a length slot. Keep unclassified
// units with the profile validator rather than guessing future length syntax.
fn non_length_unit(unit: &str) -> bool {
    matches!(unit.to_ascii_lowercase().as_str(),
        "deg" | "grad" | "rad" | "turn" | "s" | "ms" | "hz" | "khz"
        | "dpi" | "dpcm" | "dppx" | "x" | "fr")
}
fn color_keyword(value: &str) -> bool {
    cssparser::color::parse_named_color(value).is_ok()
        || matches!(value, "transparent" | "currentcolor"
            | "accentcolor" | "accentcolortext" | "activetext" | "buttonborder"
            | "buttonface" | "buttontext" | "canvas" | "canvastext" | "field"
            | "fieldtext" | "graytext" | "highlight" | "highlighttext" | "linktext"
            | "mark" | "marktext" | "selecteditem" | "selecteditemtext" | "visitedtext"
            | "activeborder" | "activecaption" | "appworkspace" | "background"
            | "buttonhighlight" | "buttonshadow" | "captiontext" | "inactiveborder"
            | "inactivecaption" | "inactivecaptiontext" | "infobackground" | "infotext"
            | "menu" | "menutext" | "scrollbar" | "threeddarkshadow" | "threedface"
            | "threedhighlight" | "threedlightshadow" | "threedshadow"
            | "window" | "windowframe" | "windowtext")
}
fn length_keyword(property: &str, value: &str) -> bool {
    let intrinsic = matches!(value, "min-content" | "max-content" | "fit-content" | "stretch" | "contain");
    match property {
        "width" | "height" | "min-width" | "min-height" => value == "auto" || intrinsic,
        "max-width" | "max-height" => value == "none" || intrinsic,
        "flex-basis" => matches!(value, "auto" | "content") || intrinsic,
        "margin" | "margin-top" | "margin-right" | "margin-bottom" | "margin-left" | "inset" | "top" | "right" | "bottom" | "left" => value == "auto",
        "gap" | "row-gap" | "column-gap" | "letter-spacing" | "word-spacing" => value == "normal",
        _ => false,
    }
}

fn scalar_color_function_invalid(property: &str, value: &str) -> Option<bool> {
    if !matches!(property, "color" | "background-color" | "text-decoration-color" | "border-color" | "background") { return None; }
    let mut input = ParserInput::new(value);
    let mut p = Parser::new(&mut input);
    let Token::Function(name) = p.next().ok()? else { return None; };
    if !["rgb", "rgba", "hsl", "hsla"].iter().any(|n| name.eq_ignore_ascii_case(n)) { return None; }
    let scalar = p.parse_nested_block(|p| {
        while !p.is_exhausted() {
            match p.next()? {
                Token::Number { .. } | Token::Percentage { .. } | Token::Dimension { .. }
                | Token::Comma | Token::Delim('/') => {},
                // Relative colors, none and nested math are valid CSS forms
                // outside our profile; retain their diagnostics.
                _ => return Ok::<_, cssparser::ParseError<'_, ()>>(false),
            }
        }
        Ok(true)
    }).ok()?;
    if !scalar { return None; }
    if !p.is_exhausted() {
        return if matches!(property, "background" | "border-color") { None } else { Some(true) };
    }
    Some(crate::color(value, "css").is_err())
}

fn shorthand_color_functions_invalid(name: &str, value: &str) -> Option<bool> {
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    let mut flat = Vec::new();
    let mut has_function = false;
    while !parser.is_exhausted() {
        let start = parser.position();
        let token = parser.next().ok()?.clone();
        if matches!(token, Token::Function(_)) {
            parser.parse_nested_block(|p| {
                while !p.is_exhausted() { p.next()?; }
                Ok::<_, cssparser::ParseError<'_, ()>>(())
            }).ok()?;
            match scalar_color_function_invalid("text-decoration-color", parser.slice_from(start).trim()) {
                Some(true) => return Some(true),
                // A validated color occupies the same grammar slot as a named
                // color. Flatten only for validation, never for actual emission.
                Some(false) => flat.push("red".to_owned()),
                None => return None,
            }
            has_function = true;
        } else if matches!(token, Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock) {
            return None;
        } else {
            flat.push(token.to_css_string());
        }
    }
    has_function.then(|| definitely_invalid(name, &flat.join(" ")))
}

fn background_numeric(token: &Token<'_>) -> bool {
    matches!(token, Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. })
}
fn background_axis(token: &Token<'_>) -> u8 {
    match token {
        Token::Ident(word) => match word.to_ascii_lowercase().as_str() {
            "left" | "right" => 1,
            "top" | "bottom" => 2,
            "center" => 3,
            _ => 0,
        },
        _ => 0,
    }
}
fn background_length_invalid(token: &Token<'_>, negative: bool) -> bool {
    match token {
        Token::Number { value, .. } => *value != 0.0,
        Token::Dimension { value, unit, .. } => non_length_unit(unit) || (!negative && *value < 0.0),
        Token::Percentage { unit_value, .. } => !negative && *unit_value < 0.0,
        _ => false,
    }
}
fn background_position_invalid(tokens: &[Token<'_>]) -> bool {
    if tokens.is_empty() || tokens.len() > 4 || tokens.iter().any(|t| background_length_invalid(t, true)) { return true; }
    let axes_fit = |a: u8, b: u8| (a & 1 != 0 && b & 2 != 0) || (a & 2 != 0 && b & 1 != 0);
    if tokens.len() == 1 { return false; }
    if tokens.len() == 2 {
        let a = background_axis(&tokens[0]);
        let b = background_axis(&tokens[1]);
        // Keyword pairs may reverse axes; any numeric component fixes x/y order.
        return if a != 0 && b != 0 { !axes_fit(a, b) }
               else { !(a == 0 || a & 1 != 0) || !(b == 0 || b & 2 != 0) };
    }
    let mut groups = Vec::new();
    let mut at = 0;
    while at < tokens.len() {
        let axis = background_axis(&tokens[at]);
        if axis == 0 { return true; }
        at += 1;
        if tokens.get(at).is_some_and(background_numeric) {
            // Offsets attach to edges, never to center.
            if axis == 3 { return true; }
            at += 1;
        }
        groups.push(axis);
    }
    groups.len() != 2 || !axes_fit(groups[0], groups[1])
}
fn background_size_token(token: &Token<'_>) -> bool {
    background_numeric(token) || matches!(token, Token::Ident(word) if matches!(word.to_ascii_lowercase().as_str(), "auto" | "cover" | "contain"))
}
fn background_size_invalid(tokens: &[Token<'_>]) -> bool {
    tokens.is_empty() || tokens.len() > 2 || tokens.iter().any(|token| {
        background_length_invalid(token, false)
            || (tokens.len() > 1 && matches!(token, Token::Ident(word) if word.eq_ignore_ascii_case("cover") || word.eq_ignore_ascii_case("contain")))
    })
}

// Inspect structural/type mismatches before normalization rejects otherwise
// unimplemented syntax. Parser::next skips nested function contents, so a color
// inside a gradient and math inside a color are classified by their outer type.
// Only classify the bounded scalar grammar understood by the ratio parser.
// Unknown functions and typed arithmetic must keep unsupported diagnostics.
fn invalid_scalar_ratio(value: &str) -> bool {
    let mut keyword_input = ParserInput::new(value);
    let mut keyword_parser = Parser::new(&mut keyword_input);
    if let Ok(keyword) = keyword_parser.expect_ident_cloned() {
        if keyword_parser.is_exhausted() && wide_keyword(&keyword.to_ascii_lowercase()) { return false; }
    }
    if crate::aspect_ratio::AspectRatio::parse(value, "css").is_ok() { return false; }
    fn known(p: &mut Parser<'_, '_>, depth: usize, budget: &mut usize) -> bool {
        if depth >= 16 { return false; }
        while !p.is_exhausted() {
            if *budget == 0 { return false; }
            *budget -= 1;
            let Ok(token) = p.next().cloned() else { return false; };
            match token {
                Token::Number { .. } | Token::Percentage { .. } | Token::Delim(_) | Token::Comma => {},
                Token::Dimension { unit, .. } if crate::number_math::absolute_unit(&unit).is_some()
                    || crate::number_math::font_unit(&unit, 16.0).is_some() => {},
                Token::Ident(name) if depth == 0 || matches!(name.to_ascii_lowercase().as_str(),
                    "pi" | "e" | "infinity" | "-infinity" | "nan") => {},
                Token::Function(name) if matches!(name.to_ascii_lowercase().as_str(), "calc" | "min" | "max" | "clamp") => {
                    if !p.parse_nested_block(|p| Ok::<_, cssparser::ParseError<'_, ()>>(known(p, depth + 1, budget))).unwrap_or(false) { return false; }
                },
                Token::ParenthesisBlock => {
                    if !p.parse_nested_block(|p| Ok::<_, cssparser::ParseError<'_, ()>>(known(p, depth + 1, budget))).unwrap_or(false) { return false; }
                },
                _ => return false,
            }
        }
        true
    }
    let mut input = ParserInput::new(value);
    known(&mut Parser::new(&mut input), 0, &mut 128)
}

fn invalid_side_component_count(name: &str, value: &str) -> bool {
    if crate::border::side_property(name).is_some_and(|(_, component)| component != "border") {
        // Unlike the four-edge longhands, a physical side component takes
        // exactly one value. Count top-level tokens even for valid functions
        // outside the profile; nested function arguments are not extra values.
        let mut input = ParserInput::new(value);
        let mut parser = Parser::new(&mut input);
        let mut count = 0;
        while !parser.is_exhausted() {
            if parser.next().is_err() { return true; }
            count += 1;
            if count > 1 { return true; }
        }
        if count == 0 { return true; }
    }
    false
}

pub(crate) fn invalid_top_level_component(property: &str, value: &str) -> bool {
    if invalid_side_component_count(property, value) { return true; }
    let property = crate::border::side_property(property).map(|(_, component)| component).unwrap_or(property);
    if property == "aspect-ratio" && invalid_scalar_ratio(value) { return true; }
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    while let Ok(token) = parser.next() {
        let compatible = match token {
            Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock => false,
            Token::UnquotedUrl(_) => matches!(property, "background" | "background-image"),
            Token::Function(name) => match name.to_ascii_lowercase().as_str() {
                "rgb" | "rgba" | "hsl" | "hsla" | "hwb" | "lab" | "lch" | "oklab" | "oklch"
                | "color" | "color-mix" | "light-dark" | "contrast-color" | "device-cmyk" =>
                    matches!(property, "color" | "background-color" | "text-decoration-color" | "background" | "text-decoration" | "border" | "border-color"),
                "url" | "image" | "image-set" | "cross-fade" | "element" | "paint"
                | "linear-gradient" | "radial-gradient" | "conic-gradient"
                | "repeating-linear-gradient" | "repeating-radial-gradient" | "repeating-conic-gradient" => matches!(property, "background" | "background-image"),
                "calc" | "min" | "max" | "clamp" => matches!(property,
                    "width" | "height" | "min-width" | "min-height" | "max-width" | "max-height"
                    | "padding" | "padding-top" | "padding-right" | "padding-bottom" | "padding-left"
                    | "margin" | "margin-top" | "margin-right" | "margin-bottom" | "margin-left"
                    | "gap" | "row-gap" | "column-gap" | "border-radius" | "border-top-left-radius" | "border-top-right-radius" | "border-bottom-right-radius" | "border-bottom-left-radius" | "border" | "border-width" | "background"
                    | "aspect-ratio" | "opacity" | "order" | "flex" | "flex-basis" | "flex-grow" | "flex-shrink" | "font" | "font-size"
                    | "font-weight" | "line-height" | "letter-spacing" | "word-spacing"
                    | "text-decoration" | "text-decoration-thickness" | "text-underline-offset"),
                // New/unknown functions are not assumed to be invalid CSS.
                _ => true,
            },
            _ => true,
        };
        if !compatible { return true; }
    }
    false
}

pub(crate) fn definitely_invalid(name: &str, value: &str) -> bool {
    if invalid_side_component_count(name, value) { return true; }
    let name = crate::border::side_property(name).map(|(_, component)| component).unwrap_or(name);
    if matches!(name, "text-decoration" | "background" | "border") {
        if let Some(invalid) = shorthand_color_functions_invalid(name, value) { return invalid; }
    }
    if let Some(invalid) = scalar_color_function_invalid(name, value) { return invalid; }
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    let mut tokens = Vec::new();
    while !parser.is_exhausted() {
        let Ok(token) = parser.next() else { return true; };
        if matches!(token, Token::Function(_) | Token::ParenthesisBlock | Token::SquareBracketBlock | Token::CurlyBracketBlock) {
            return false;
        }
        tokens.push(token.clone());
    }
    if tokens.is_empty() { return true; }
    if tokens.len() == 1 {
        if let Token::Ident(value) = &tokens[0] {
            if wide_keyword(&value.to_ascii_lowercase()) { return false; }
        }
    }
    if name == "background-image" {
        return !(tokens.len() == 1 && matches!(&tokens[0],
            Token::Ident(word) if word.eq_ignore_ascii_case("none")))
            && !tokens.iter().any(|token| matches!(token, Token::UnquotedUrl(_)));
    }
    if matches!(name, "border" | "border-width" | "border-style" | "border-color") {
        let mut seen = 0u8;
        if name != "border" && tokens.len() > 4 { return true; }
        for token in &tokens {
            let bit = match token {
                Token::Number { value, .. } if *value == 0.0 => 1,
                Token::Dimension { value, unit, .. } => {
                    if !value.is_finite() || *value < 0.0 || non_length_unit(unit) { return true; }
                    1
                }
                Token::Ident(word) => {
                    let word = word.to_ascii_lowercase();
                    if word.starts_with('-') { return false; }
                    if matches!(word.as_str(), "thin" | "medium" | "thick") { 1 }
                    else if matches!(word.as_str(), "none" | "hidden" | "solid" | "dotted" |
                        "dashed" | "double" | "groove" | "ridge" | "inset" | "outset") { 2 }
                    else if color_keyword(&word) { 4 }
                    else { return true; }
                }
                Token::Hash(_) | Token::IDHash(_) => {
                    if crate::color(&token.to_css_string(), "css").is_err() { return true; }
                    4
                }
                _ => return true,
            };
            if name == "border" {
                if seen & bit != 0 { return true; }
                seen |= bit;
            } else if bit != match name { "border-width" => 1, "border-style" => 2, _ => 4 } {
                return true;
            }
        }
        return false;
    }
    if name == "overflow-clip-margin" {
        let mut origin = false;
        let mut length = false;
        for token in &tokens {
            match token {
                Token::Ident(value) if !origin && matches!(value.to_ascii_lowercase().as_str(),
                    "content-box" | "padding-box" | "border-box") => origin = true,
                Token::Dimension { unit, .. } if !length && !non_length_unit(unit) => length = true,
                Token::Number { value, .. } if !length && *value == 0. => length = true,
                _ => return true,
            }
        }
        return false;
    }
    if name == "order" {
        return tokens.len() != 1 || !matches!(tokens[0], Token::Number { int_value: Some(_), .. });
    }
    if name == "background" {
        let mut color = false;
        let mut image = false;
        let mut repeats = 0;
        let mut repeat_axis = false;
        let mut last_repeat = None;
        let mut attachment = false;
        let mut boxes = 0;
        let mut layer = false;
        let mut position = false;
        let mut skip_until = 0;
        for (index, token) in tokens.iter().enumerate() {
            if index < skip_until { continue; }
            if background_numeric(token) || background_axis(token) != 0 {
                if position { return true; }
                position = true;
                let mut end = index + 1;
                while end < tokens.len() && (background_numeric(&tokens[end]) || background_axis(&tokens[end]) != 0) { end += 1; }
                if background_position_invalid(&tokens[index..end]) { return true; }
                if matches!(tokens.get(end), Some(Token::Delim('/'))) {
                    let start = end + 1;
                    end = start;
                    while end < tokens.len() && background_size_token(&tokens[end]) { end += 1; }
                    if background_size_invalid(&tokens[start..end]) { return true; }
                }
                skip_until = end;
                layer = true;
                continue;
            }
            match token {
                Token::Comma => {
                    // A color is permitted only in the final layer.
                    if color || !layer { return true; }
                    image = false;
                    repeats = 0;
                    repeat_axis = false;
                    last_repeat = None;
                    attachment = false;
                    boxes = 0;
                    layer = false;
                    position = false;
                    continue;
                }
                Token::Ident(word) => {
                    let word = word.to_ascii_lowercase();
                    if word == "none" {
                        if image { return true; }
                        image = true;
                    } else if color_keyword(&word) {
                        if color { return true; }
                        color = true;
                    } else if matches!(word.as_str(), "repeat" | "no-repeat" | "space" | "round" | "repeat-x" | "repeat-y") {
                        let axis = matches!(word.as_str(), "repeat-x" | "repeat-y");
                        // The two-axis repeat form is one contiguous grammar
                        // component; repeat-x/y already occupy both axes.
                        if repeats > 0 && (repeat_axis || axis || repeats == 2 || last_repeat != Some(index - 1)) { return true; }
                        repeats += 1;
                        repeat_axis = axis;
                        last_repeat = Some(index);
                    } else if matches!(word.as_str(), "scroll" | "fixed" | "local") {
                        if attachment { return true; }
                        attachment = true;
                    } else if matches!(word.as_str(), "border-box" | "padding-box" | "content-box") {
                        boxes += 1;
                        if boxes > 2 { return true; }
                    } else if word.starts_with('-') || matches!(word.as_str(),
                        "start" | "end"
                        | "x-start" | "x-end" | "y-start" | "y-end" | "inline-start" | "inline-end"
                        | "block-start" | "block-end" | "text" | "border-area") {
                        return false;
                    } else { return true; }
                }
                Token::Hash(_) | Token::IDHash(_) => {
                    if color || definitely_invalid("background-color", &token.to_css_string()) { return true; }
                    color = true;
                }
                // Image grammar stays with the profile validator.
                Token::UnquotedUrl(_) => return false,
                _ => return true,
            }
            layer = true;
        }
        return !layer;
    }
    if name == "text-decoration" {
        let mut lines = Vec::new();
        let mut seen = 0u8;
        for token in &tokens {
            let value = token.to_css_string();
            let slot = match token {
                Token::Ident(word) => {
                    let word = word.to_ascii_lowercase();
                    if word.starts_with('-') { return false; }
                    match word.as_str() {
                        "none" | "underline" | "overline" | "line-through" | "blink"
                        | "spelling-error" | "grammar-error" => { lines.push(value); continue; }
                        "solid" | "double" | "dotted" | "dashed" | "wavy" => 1,
                        "auto" | "from-font" => 2,
                        _ if color_keyword(&word) => 4,
                        _ => return true,
                    }
                }
                Token::Number { value, .. } if *value == 0.0 => 2,
                Token::Dimension { unit, .. } if non_length_unit(unit) => return true,
                Token::Dimension { .. } | Token::Percentage { .. } => 2,
                Token::Hash(_) | Token::IDHash(_) => {
                    if definitely_invalid("text-decoration-color", &value) { return true; }
                    4
                }
                _ => return true,
            };
            if seen & slot != 0 { return true; }
            seen |= slot;
        }
        return !lines.is_empty() && definitely_invalid("text-decoration-line", &lines.join(" "));
    }
    if name == "text-underline-position" {
        let mut seen = 0u8;
        for token in &tokens {
            let Token::Ident(word) = token else { return true; };
            let word = word.to_ascii_lowercase();
            if word.starts_with('-') { return false; }
            let bit = match word.as_str() {
                "auto" => return tokens.len() != 1,
                "from-font" | "under" => 1,
                "left" | "right" => 2,
                _ => return true,
            };
            if seen & bit != 0 { return true; }
            seen |= bit;
        }
        return false;
    }
    if matches!(name, "text-decoration-thickness" | "text-underline-offset") {
        if tokens.len() != 1 { return true; }
        return match &tokens[0] {
            Token::Number { value, .. } => *value != 0.0,
            // Negative thickness is valid CSS. The renderer's narrower range
            // remains a diagnostic, rather than becoming an invalid-value reset.
            Token::Dimension { unit, .. } => non_length_unit(unit),
            Token::Percentage { .. } => false,
            Token::Ident(word) => {
                let word = word.to_ascii_lowercase();
                word != "auto" && !(name == "text-decoration-thickness" && word == "from-font")
                    && !word.starts_with('-')
            }
            _ => true,
        };
    }
    if name == "text-decoration-line" {
        let mut seen = 0u8;
        for token in &tokens {
            let Token::Ident(word) = token else { return true; };
            let word = word.to_ascii_lowercase();
            if word.starts_with('-') { return false; }
            let bit = match word.as_str() {
                "none" | "spelling-error" | "grammar-error" => return tokens.len() != 1,
                "underline" => 1,
                "overline" => 2,
                "line-through" => 4,
                "blink" => 8,
                _ => return true,
            };
            if seen & bit != 0 { return true; }
            seen |= bit;
        }
        return false;
    }
    if name == "white-space" {
        let mut seen = 0u8;
        for token in &tokens {
            let Token::Ident(word) = token else { return true; };
            let word = word.to_ascii_lowercase();
            if word.starts_with('-') { return false; }
            let bit = match word.as_str() {
                "normal" | "pre" | "pre-wrap" | "pre-line" => return tokens.len() != 1,
                "collapse" | "discard" | "preserve" | "preserve-breaks" | "preserve-spaces" | "break-spaces" => 1,
                "wrap" | "nowrap" => 2,
                "none" => 4,
                "discard-before" => 8,
                "discard-after" => 16,
                "discard-inner" => 32,
                _ => return true,
            };
            if seen & bit != 0 || (bit == 4 && seen & 56 != 0) || (bit & 56 != 0 && seen & 4 != 0) { return true; }
            seen |= bit;
        }
        return false;
    }
    if name == "text-transform" {
        let mut seen = 0u8;
        for token in &tokens {
            let Token::Ident(word) = token else { return true; };
            let word = word.to_ascii_lowercase();
            if word.starts_with('-') { return false; }
            let bit = match word.as_str() {
                "none" | "math-auto" => return tokens.len() != 1,
                "uppercase" | "lowercase" | "capitalize" => 1,
                "full-width" => 2,
                "full-size-kana" => 4,
                _ => return true,
            };
            if seen & bit != 0 { return true; }
            seen |= bit;
        }
        return false;
    }
    if name == "font" {
        let mut at = 0;
        let mut normals = 0;
        let mut weight = false;
        let mut style = false;
        let mut variant = false;
        let mut width = false;
        while let Some(token) = tokens.get(at) {
            match token {
                Token::Dimension { .. } | Token::Percentage { .. } => break,
                Token::Number { value, .. } if *value == 0.0 => break,
                Token::Number { value, .. } => {
                    if weight || !(1.0..=1000.0).contains(value) { return true; }
                    weight = true;
                },
                Token::Ident(value) if value.eq_ignore_ascii_case("normal") => normals += 1,
                Token::Ident(value) if ["bold", "bolder", "lighter"].iter().any(|v| value.eq_ignore_ascii_case(v)) => {
                    if weight { return true; }
                    weight = true;
                },
                Token::Ident(value) => {
                    let value = value.to_ascii_lowercase();
                    if matches!(value.as_str(), "caption" | "icon" | "menu" | "message-box" | "small-caption" | "status-bar") {
                        return tokens.len() != 1;
                    }
                    match value.as_str() {
                        "italic" | "oblique" => {
                            if style { return true; }
                            style = true;
                            if value == "oblique" {
                                if let Some(Token::Dimension { value, unit, .. }) = tokens.get(at + 1) {
                                    let degrees = match unit.to_ascii_lowercase().as_str() {
                                        "deg" => Some(f64::from(*value)),
                                        "grad" => Some(f64::from(*value) * 0.9),
                                        "turn" => Some(f64::from(*value) * 360.0),
                                        "rad" => Some(f64::from(*value).to_degrees()),
                                        _ => None,
                                    };
                                    if let Some(degrees) = degrees {
                                        if !(-90.0..=90.0).contains(&degrees) { return true; }
                                        at += 1;
                                    }
                                }
                            }
                        }
                        "small-caps" => {
                            if variant { return true; }
                            variant = true;
                        }
                        "ultra-condensed" | "extra-condensed" | "condensed" | "semi-condensed"
                        | "semi-expanded" | "expanded" | "extra-expanded" | "ultra-expanded" => {
                            if width { return true; }
                            width = true;
                        }
                        "xx-small" | "x-small" | "small" | "medium" | "large" | "x-large"
                        | "xx-large" | "xxx-large" | "larger" | "smaller" | "math" => break,
                        _ => return !value.starts_with('-'),
                    }
                },
                _ => return true,
            }
            if normals + usize::from(weight) + usize::from(style) + usize::from(variant) + usize::from(width) > 4 { return true; }
            at += 1;
        }
        let Some(size) = tokens.get(at) else { return true; };
        if definitely_invalid("font-size", &size.to_css_string()) { return true; }
        at += 1;
        if matches!(tokens.get(at), Some(Token::Delim('/'))) {
            at += 1;
            let Some(height) = tokens.get(at) else { return true; };
            if definitely_invalid("line-height", &height.to_css_string()) { return true; }
            at += 1;
        }
        if let Some(Token::Ident(first)) = tokens.get(at) {
            if wide_keyword(&first.to_ascii_lowercase()) { return true; }
        }
        let family = tokens[at..].iter().map(ToCss::to_css_string).collect::<Vec<_>>().join(" ");
        return definitely_invalid("font-family", &family);
    }
    if name == "font-family" {
        // After substitution Chromium treats a leading CSS-wide word plus
        // trailing tokens as invalid, even where direct family syntax accepts
        // the words as a name. Sole CSS-wide values were handled above.
        if let Token::Ident(first) = &tokens[0] {
            if wide_keyword(&first.to_ascii_lowercase()) { return true; }
        }
        for family in tokens.split(|token| matches!(token, Token::Comma)) {
            if matches!(family, [Token::QuotedString(_)]) { continue; }
            if family.is_empty() || family.iter().any(|t| !matches!(t, Token::Ident(_))) { return true; }
            let Token::Ident(first) = &family[0] else { unreachable!() };
            let first = first.to_ascii_lowercase();
            // Chromium accepts keyword-looking words inside multi-word family
            // names, but a leading generic consumes a complete list entry.
            if family.len() == 1 {
                if first == "default" || wide_keyword(&first) { return true; }
            } else if matches!(first.as_str(), "serif" | "sans-serif" | "monospace" | "cursive" | "fantasy"
                | "system-ui" | "ui-serif" | "ui-sans-serif" | "ui-monospace" | "ui-rounded" | "emoji" | "math" | "fangsong") {
                return true;
            }
        }
        return false;
    }
    if matches!(name, "font-size" | "font-weight" | "line-height") {
        if tokens.len() != 1 { return true; }
        return match &tokens[0] {
            Token::Number { value, .. } => match name {
                "font-weight" => !(1.0..=1000.0).contains(value),
                "font-size" => *value != 0.0,
                _ => *value < 0.0,
            },
            Token::Dimension { value, unit, .. } => name == "font-weight" || *value < 0.0 || non_length_unit(unit),
            Token::Percentage { unit_value, .. } => name == "font-weight" || *unit_value < 0.0,
            Token::Ident(value) => {
                let value = value.to_ascii_lowercase();
                if value.starts_with('-') { return false; }
                !match name {
                    "font-weight" => matches!(value.as_str(), "normal" | "bold" | "bolder" | "lighter"),
                    "font-size" => matches!(value.as_str(), "xx-small" | "x-small" | "small" | "medium" | "large" | "x-large" | "xx-large" | "xxx-large" | "larger" | "smaller" | "math"),
                    _ => value == "normal",
                }
            },
            _ => true,
        };
    }
    if matches!(name, "border-top-left-radius" | "border-top-right-radius" |
        "border-bottom-right-radius" | "border-bottom-left-radius") {
        // Two lengths are valid elliptical CSS, although outside the circular
        // profile. Keep valid excluded syntax for an explicit compiler error.
        if tokens.is_empty() || tokens.len() > 2 { return true; }
        return tokens.iter().any(|token| !matches!(token,
            Token::Number { value, .. } if *value == 0.0) && !matches!(token,
            Token::Dimension { value, unit, .. } if *value >= 0.0 && !non_length_unit(unit)) && !matches!(token,
            Token::Percentage { unit_value, .. } if *unit_value >= 0.0));
    }
    if name == "border-radius" {
        let mut count = 0;
        let mut slash = false;
        for token in &tokens {
            match token {
                Token::Delim('/') => {
                    if slash || count == 0 { return true; }
                    slash = true;
                    count = 0;
                    continue;
                },
                Token::Number { value, .. } if *value == 0.0 => {},
                Token::Dimension { value, unit, .. } if *value >= 0.0 && !non_length_unit(unit) => {},
                Token::Percentage { unit_value, .. } if *unit_value >= 0.0 => {},
                Token::Ident(value) if value.starts_with('-') => return false,
                _ => return true,
            }
            count += 1;
            if count > 4 { return true; }
        }
        return count == 0;
    }
    if name == "opacity" {
        return tokens.len() != 1 || !matches!(&tokens[0], Token::Number { .. } | Token::Percentage { .. });
    }
    if matches!(name, "flex-grow" | "flex-shrink") {
        return tokens.len() != 1 || !matches!(&tokens[0], Token::Number { value, .. } if *value >= 0.0);
    }
    if name == "flex" {
        // The two factors form one ordered group; basis may precede or follow
        // that group. Preserve possible valid-but-excluded basis units/keywords.
        let factor = |t: &Token<'_>| matches!(t, Token::Number { value, .. } if *value >= 0.0);
        let basis = |t: &Token<'_>| match t {
            Token::Number { value, .. } => *value == 0.0,
            Token::Dimension { value, unit, .. } => *value >= 0.0 && !non_length_unit(unit),
            Token::Percentage { unit_value, .. } => *unit_value >= 0.0,
            Token::Ident(value) => {
                let value = value.to_ascii_lowercase();
                value.starts_with('-') || length_keyword("flex-basis", &value)
            },
            _ => false,
        };
        return !match tokens.as_slice() {
            [Token::Ident(value)] if value.eq_ignore_ascii_case("none") => true,
            [a] => factor(a) || basis(a),
            [a,b] => (factor(a) && (factor(b) || basis(b))) || (basis(a) && factor(b)),
            [a,b,c] => (factor(a) && factor(b) && basis(c)) || (basis(a) && factor(b) && factor(c)),
            _ => false,
        };
    }
    if matches!(name, "align-items" | "align-self" | "align-content" | "justify-content" | "text-align") {
        let mut words = Vec::new();
        for token in &tokens {
            match token {
                Token::Ident(value) => words.push(value.to_ascii_lowercase()),
                // Preserve string alignment syntax as outside the profile.
                Token::QuotedString(_) if name == "text-align" => return false,
                _ => return true,
            }
        }
        if words.iter().any(|word| word.starts_with('-')) { return false; }
        let positions: &[&str] = if matches!(name, "align-items" | "align-self") {
            &["center", "start", "end", "self-start", "self-end", "flex-start", "flex-end"]
        } else {
            &["center", "start", "end", "flex-start", "flex-end", "left", "right"]
        };
        return !match words.as_slice() {
            [word] if name == "text-align" => matches!(word.as_str(), "start" | "end" | "left" | "right" | "center" | "justify" | "match-parent" | "justify-all"),
            [word] => (name == "align-self" && word == "auto") || positions.contains(&word.as_str()) || matches!(word.as_str(), "normal" | "stretch")
                || (name == "align-content" && word == "baseline")
                || if matches!(name, "align-items" | "align-self") { matches!(word.as_str(), "baseline" | "anchor-center" | "dialog") }
                   else { matches!(word.as_str(), "space-between" | "space-around" | "space-evenly") },
            [modifier, position] if name != "text-align" =>
                (matches!(modifier.as_str(), "safe" | "unsafe") && positions.contains(&position.as_str()))
                || (matches!(name, "align-items" | "align-self" | "align-content") && matches!(modifier.as_str(), "first" | "last") && position == "baseline"),
            _ => false,
        };
    }
    let enumeration: Option<(&[&str], usize, bool)> = match name {
        "text-decoration-style" => Some((&["solid", "double", "dotted", "dashed", "wavy"], 1, false)),
        "text-decoration-skip-ink" => Some((&["auto", "none", "all"], 1, false)),
        "position" => Some((&["static", "relative", "absolute", "fixed", "sticky"], 1, false)),
        "flex-direction" => Some((&["row", "row-reverse", "column", "column-reverse"], 1, false)),
        "flex-wrap" => Some((&["nowrap", "wrap", "wrap-reverse"], 1, false)),
        "overflow-x" | "overflow-y" => Some((&["visible", "hidden", "clip", "scroll", "auto", "overlay"], 1, false)),
        "overflow" => Some((&["visible", "hidden", "clip", "scroll", "auto", "overlay"], 2, false)),
        "text-overflow" => Some((&["clip", "ellipsis", "fade"], 2, true)),
        _ => None,
    };
    if let Some((keywords, maximum, strings)) = enumeration {
        return tokens.len() > maximum || tokens.iter().any(|token| match token {
            Token::Ident(value) => {
                let value = value.to_ascii_lowercase();
                !value.starts_with('-') && !keywords.contains(&value.as_str())
            }
            Token::QuotedString(_) if strings => false,
            _ => true,
        });
    }
    if matches!(name, "color" | "background-color" | "text-decoration-color") {
        if tokens.len() != 1 { return true; }
        return match &tokens[0] {
            // Preserve valid system/vendor colors as profile diagnostics.
            Token::Ident(value) => {
                let value = value.to_ascii_lowercase();
                !value.starts_with('-') && !color_keyword(&value)
            },
            Token::Hash(hex) | Token::IDHash(hex) =>
                !matches!(hex.len(), 3 | 4 | 6 | 8) || !hex.bytes().all(|b| b.is_ascii_hexdigit()),
            _ => true,
        };
    }
    let (max_components, negative_allowed) = match name {
        "width" | "height" | "min-width" | "max-width" | "min-height" | "max-height" | "flex-basis" => (1, false),
        "padding" => (4, false),
        "padding-top" | "padding-right" | "padding-bottom" | "padding-left" => (1, false),
        "gap" => (2, false),
        "row-gap" | "column-gap" => (1, false),
        "margin" | "inset" => (4, true),
        "margin-top" | "margin-right" | "margin-bottom" | "margin-left" | "letter-spacing" | "word-spacing" | "top" | "right" | "bottom" | "left" => (1, true),
        _ => return false,
    };
    if tokens.len() > max_components { return true; }
    tokens.iter().any(|token| match token {
        Token::Number { value, .. } => *value != 0.0,
        Token::Dimension { value, unit, .. } => non_length_unit(unit) || (!negative_allowed && *value < 0.0),
        Token::Percentage { unit_value, .. } => !negative_allowed && *unit_value < 0.0,
        Token::Ident(value) => {
            let value = value.to_ascii_lowercase();
            !value.starts_with('-') && !length_keyword(name, &value)
        },
        // Unit validation remains the property's responsibility: recognized but
        // unimplemented units must not silently change authored layout.
        _ => true,
    })
}
