//! HTML/CSS authoring compiler. See SUPPORT.md for the versioned language contract.
use scraper::{ElementRef, Html};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
mod assets;
mod border;
mod corner_radii;
mod aspect_ratio;
mod number_math;
mod css;
mod custom_properties;
mod decoration;
mod requirements;
mod style;
mod overflow;
mod opacity;
mod gradient;
mod overflow_clip_margin;
mod position;
mod substitution_validity;
mod text_content;
pub use requirements::{
    LayoutLinearGradientRequirement, LinearGradientDirection, LinearGradientPosition, LinearGradientStop,
    CornerRadiusValue, LayoutCornerRadiiRequirement, LayoutGroupOpacityRequirement,
    LayoutBorderRequirement, LayoutBorderSidesRequirement,
    LayoutAspectRatioRequirement, LayoutStackingRequirement, LayoutAxisOverflowRequirement, OverflowAxis, LayoutOverflowClipMarginRequirement, OverflowClipBox,
    AlignSelf, LayoutAlignSelfRequirement, AlignContent, LayoutAlignContentRequirement,
    JustifyDistribution, LayoutJustifyContentRequirement, LayoutFlexFactorsRequirement,
    RuntimeCapability, RuntimeRequirements, SolidStrikethrough, SolidUnderline, TextPolicy,
    TextPolicyRequirement, TextStrikethroughRequirement, TextUnderlineRequirement,
    UnderlineSkipInk,
};
#[cfg(target_arch = "wasm32")]
mod wasm;
mod wire;
use style::Style;
use wire::{Record, Value};

const MAX_SOURCE_IDENTITIES: usize = 8192;

pub const LANGUAGE_VERSION: &str = "nuxie-html-v1";
pub const BROWSER_RESET_CSS: &str = include_str!("reset.css");

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompileInput {
    pub html: String,
    pub css: String,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub assets: BTreeMap<String, Asset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Asset {
    Font {
        family: String,
        weight: u16,
        bytes: Vec<u8>,
    },
    Image {
        bytes: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceBreak {
    pub id: String,
    pub path: String,
    /// Unicode scalar offset of the newline in rendered text after normalization/casing.
    pub text_offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceTextTransform {
    /// Normalized text before presentational casing (not the raw HTML source).
    pub source: String,
    pub rendered: String,
    /// Rendered scalar offset at every source scalar boundary, including end.
    pub scalar_offsets: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceNode {
    pub id: String,
    pub path: String,
    pub object_id: u32,
    pub text_run_id: Option<u32>,
    /// All text runs in logical source order. Singular id is set only for one run.
    #[serde(default)]
    pub text_run_ids: Vec<u32>,
    #[serde(default)]
    pub text_breaks: Vec<SourceBreak>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_transform: Option<SourceTextTransform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileOutput {
    pub runtime_requirements: RuntimeRequirements,
    pub riv: Vec<u8>,
    pub source_map: Vec<SourceNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub source: String,
    pub message: String,
}

impl Diagnostic {
    pub(crate) fn new(code: &str, source: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            source: source.into(),
            message: message.into(),
        }
    }
}

pub fn compile(input: &CompileInput) -> Result<CompileOutput, Vec<Diagnostic>> {
    compile_inner(input).map_err(|e| vec![e])
}

fn compile_inner(input: &CompileInput) -> Result<CompileOutput, Diagnostic> {
    if !input.width.is_finite()
        || !input.height.is_finite()
        || input.width <= 0.0
        || input.height <= 0.0
        || input.width > 16384.0
        || input.height > 16384.0
    {
        return Err(Diagnostic::new(
            "invalid-viewport",
            "viewport",
            "Width and height must be finite and in (0, 16384]",
        ));
    }
    if input.html.len() + input.css.len() > 1_048_576 {
        return Err(Diagnostic::new(
            "input-limit",
            "document",
            "HTML and CSS exceed the 1 MiB limit",
        ));
    }
    let html = Html::parse_fragment(&input.html);
    if !html.errors.is_empty() {
        return Err(Diagnostic::new(
            "html-syntax",
            "html",
            html.errors
                .iter()
                .map(|e| e.as_ref())
                .collect::<Vec<_>>()
                .join("; "),
        ));
    }
    if html
        .root_element()
        .children()
        .any(|n| n.value().as_text().is_some_and(|t| !t.trim().is_empty()))
    {
        return Err(Diagnostic::new(
            "unsupported-root-text",
            "html",
            "Top-level text must be wrapped in a supported element",
        ));
    }
    let mut html_ids = BTreeSet::new();
    for node in html.tree.nodes() {
        if let Some(element) = ElementRef::wrap(node)
            && let Some(id) = element.attr("id")
            && (id.is_empty() || !html_ids.insert(id.to_owned()))
        {
            return Err(Diagnostic::new(
                "duplicate-id",
                "html",
                format!("Empty or duplicate HTML id: {id}"),
            ));
        }
    }
    let rules = css::stylesheet(&input.css)?;
    for d in css::all_declarations(&rules) {
        validate(d)?;
    }
    let mut records = vec![Record::new("Backboard")];
    assets::write(&input.assets, &mut records)?;
    let root_start = records.len() as u32;
    let mut artboard = Record::new("Artboard");
    artboard.set("name", Value::String("HTML".into()))?;
    artboard.set("width", Value::Float(input.width))?;
    artboard.set("height", Value::Float(input.height))?;
    artboard.set("styleId", Value::Uint(1))?;
    records.push(artboard);
    let mut root_style = Record::new("LayoutComponentStyle");
    root_style.set("flexDirectionValue", Value::Uint(0))?;
    records.push(root_style);
    let mut source_map = Vec::new();
    let mut ids = BTreeSet::new();
    let root_style = Style {
        width: style::Size::Percent(100.0),
        height: style::Size::Percent(100.0),
        definite_width: true,
        definite_height: true,
        ..Style::default()
    };
    // Match selectors in the same ancestry as a browser fragment in <body>.
    // Only authored elements are lowered; host html/body styles stay fixed.
    let document = Html::parse_document(&format!(
        "<!doctype html><html><head></head><body>{}</body></html>",
        input.html
    ));
    let body = document
        .root_element()
        .child_elements()
        .find(|e| e.value().name() == "body")
        .expect("HTML parser creates a body");
    let mut runtime_requirements = RuntimeRequirements::default();
    for (index, element, style) in ordered_children(body, &root_style, &rules, false)? {
        emit(
            element,
            0,
            &root_style,
            root_start,
            &input.assets,
            &format!("/{index}"),
            &rules,
            &mut records,
            &mut source_map,
            &mut ids,
            &mut runtime_requirements,
            false,
            0,
            style,
        )?;
    }
    // Object indices address this artifact; authored identities and paths retain
    // document order even when CSS changes the visual emission order.
    source_map.sort_by_cached_key(|node| node.path.split('/').skip(1)
        .map(|part| part.parse::<usize>().expect("generated numeric source path"))
        .collect::<Vec<_>>());
    if source_map.is_empty() {
        return Err(Diagnostic::new(
            "empty-document",
            "html",
            "At least one supported element is required",
        ));
    }
    // Any sibling layout pair needs CSS paint order, even when `order` is zero.
    // Source paths retain DOM ancestry independently of sorted object emission.
    let mut seen_parents = std::collections::HashSet::new();
    if source_map.iter().any(|node| !seen_parents.insert(node.path.rsplit_once('/').unwrap().0)) {
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssPaintOrderV1);
    }
    if !runtime_requirements.layout_positioned.is_empty() {
        // Finalize after walking descendants so older per-feature assignments
        // cannot downgrade the positioned paint contract.
        runtime_requirements.version = 17;
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssPositionedPaintV1);
    }
    if !runtime_requirements.layout_absolute.is_empty() {
        runtime_requirements.version = 18;
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssAbsolutePositionV1);
    }
    if !runtime_requirements.layout_stacking.is_empty() {
        runtime_requirements.version = 19;
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssStackingV1);
    }
    if !runtime_requirements.layout_axis_overflow.is_empty() {
        runtime_requirements.version = 20;
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssAxisOverflowV1);
    }
    if !runtime_requirements.layout_overflow_clip_margins.is_empty() {
        runtime_requirements.version = 21;
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssOverflowClipMarginV1);
    }
    if !runtime_requirements.layout_borders.is_empty() {
        runtime_requirements.version = 22;
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssSolidBordersV1);
    }
    if !runtime_requirements.layout_border_sides.is_empty() {
        runtime_requirements.version = 23;
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssBorderSidesV1);
    }
    if !runtime_requirements.layout_corner_radii.is_empty() {
        runtime_requirements.version = 24;
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssCornerRadiiV1);
    }
    if !runtime_requirements.layout_group_opacity.is_empty() {
        runtime_requirements.version = 25;
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssGroupOpacityV1);
    }
    if !runtime_requirements.layout_linear_gradients.is_empty() {
        runtime_requirements.version = 26;
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssLinearGradientV1);
    }
    Ok(CompileOutput {
        runtime_requirements,
        riv: wire::encode(&records)?,
        source_map,
    })
}

fn validate(d: &css::Declaration) -> Result<(), Diagnostic> {
    Style::validate_declaration(d)
}

fn length(value: &str, source: &str) -> Result<f32, Diagnostic> {
    let normalized = value.to_ascii_lowercase();
    let value = normalized.as_str();
    let value = value
        .strip_suffix("px")
        .or_else(|| (value == "0").then_some("0"));
    value
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0 && *v <= 1_000_000.0)
        .ok_or_else(|| {
            Diagnostic::new(
                "unsupported-value",
                source,
                "Expected a finite nonnegative px/em/rem length or zero",
            )
        })
}

fn color(value: &str, source: &str) -> Result<u32, Diagnostic> {
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
                "Expected a named sRGB color, hex, rgb(), rgba() or transparent",
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

// Resolve against the original DOM, then stably order complete sibling subtrees.
// This changes both imported layout children and paint order without rewriting
// selectors or authored paths. Compute each element's style only once.
fn ordered_children<'a>(
    parent: ElementRef<'a>,
    parent_style: &Style,
    rules: &[css::Rule],
    skip_breaks: bool,
) -> Result<Vec<(usize, ElementRef<'a>, Box<Style>)>, Diagnostic> {
    let mut children = Vec::new();
    for (index, element) in parent.child_elements().enumerate() {
        if skip_breaks && element.value().name() == "br" { continue; }
        if children.len() >= 2048 {
            return Err(Diagnostic::new("input-limit", "html", "Document exceeds 2048 elements"));
        }
        let mut declarations = css::cascade(rules, element)?;
        // Chrome's replaced-image UA rule precedes authored declarations.
        // Explicit initial/unset therefore still resolve to padding-box zero.
        if element.value().name() == "img" {
            declarations.insert(0, css::Declaration {
                name:"overflow-clip-margin".into(), value:"content-box".into(),
                important:false, source:"profile.img".into(),
            });
        }
        let style = Style::compute(&declarations, parent_style)?;
        children.push((index, element, Box::new(style)));
    }
    children.sort_by_key(|(_, _, style)| style.order);
    Ok(children)
}

// Keep per-side computation out of the recursive element frame so documents
// reach the declared depth limit on the default test-thread stack.
fn emit_gradient_requirement(computed: &Style, local: u32, path: &str,
    runtime_requirements: &mut RuntimeRequirements) -> Result<(), Diagnostic> {
    if let Some(gradient) = &computed.gradient {
        let direction = match gradient.direction {
            gradient::Direction::Degrees(v) => LinearGradientDirection::Degrees(v),
            gradient::Direction::Corner { right, bottom } => LinearGradientDirection::Corner { right, bottom },
        };
        let stops = gradient.stops.iter().map(|stop| {
            let color = match stop.color { gradient::Color::Rgba(v) => v, gradient::Color::CurrentColor => computed.color };
            let position = match stop.position {
                None => None,
                Some(gradient::Position::Pixels(v)) => Some(LinearGradientPosition::Pixels(v)),
                Some(gradient::Position::Percent(v)) => Some(LinearGradientPosition::Percent(v)),
                Some(gradient::Position::Em(_) | gradient::Position::Rem(_)) => {
                    return Err(Diagnostic::new("invalid-linear-gradient", path, "Gradient font-relative positions must be computed before emission"));
                }
            };
            Ok(LinearGradientStop { color, position })
        }).collect::<Result<Vec<_>, Diagnostic>>()?;
        runtime_requirements.layout_linear_gradients.push(LayoutLinearGradientRequirement { object_id: local, direction, stops });
    }
    Ok(())
}

fn emit_corner_requirement(computed: &Style, local: u32, path: &str,
    runtime_requirements: &mut RuntimeRequirements) -> Result<(), Diagnostic> {
    if !computed.radii.iter().all(|pair| matches!(pair,
        [corner_radii::RadiusValue::Pixels(x),corner_radii::RadiusValue::Pixels(y)] if x == y)) {
        let mut radii = [[CornerRadiusValue::Pixels(0.);2];4];
        for (output, input) in radii.iter_mut().flatten().zip(computed.radii.iter().flatten()) {
            *output = match *input {
                corner_radii::RadiusValue::Pixels(value) => CornerRadiusValue::Pixels(value),
                corner_radii::RadiusValue::Percent(value) => CornerRadiusValue::Percent(value),
                _ => return Err(Diagnostic::new("unsupported-corner-radius", path,
                    "Corner font-relative lengths must be resolved before emission")),
            };
        }
        runtime_requirements.layout_corner_radii.push(LayoutCornerRadiiRequirement { object_id:local, radii });
    }
    Ok(())
}

fn emit_border_requirement(computed: &Style, local: u32, path: &str,
    runtime_requirements: &mut RuntimeRequirements) -> Result<bool, Diagnostic> {
    let mut widths = [0.0; 4];
    for (width, edge) in widths.iter_mut().zip(computed.borders) {
        *width = edge.used_width(computed.font_size, path)?;
    }
    let colors = computed.borders.map(|edge| edge.color.resolve(computed.color));
    let has_border = widths.iter().any(|width| *width > 0.0);
    if has_border {
        if widths.iter().all(|width| *width == widths[0])
            && colors.iter().all(|color| *color == colors[0]) {
            runtime_requirements.layout_borders.push(LayoutBorderRequirement {
                object_id: local, color: colors[0],
            });
        } else {
            runtime_requirements.layout_border_sides.push(LayoutBorderSidesRequirement {
                object_id: local, colors,
            });
        }
    }
    Ok(has_border)
}

// Keep the large per-element emission frame out of recursive traversal. Each
// node finishes emission before descending, preserving preorder and depth limits.
fn emit(
    element: ElementRef<'_>,
    parent: u32,
    parent_style: &Style,
    root_start: u32,
    assets: &BTreeMap<String, Asset>,
    path: &str,
    rules: &[css::Rule],
    records: &mut Vec<Record>,
    sources: &mut Vec<SourceNode>,
    ids: &mut BTreeSet<String>,
    runtime_requirements: &mut RuntimeRequirements,
    ancestor_hidden: bool,
    depth: usize,
    mut computed: Box<Style>,
) -> Result<(), Diagnostic> {
    if depth > 64 || sources.len() >= 2048 {
        return Err(Diagnostic::new(
            "input-limit",
            path,
            "Document exceeds 64 levels or 2048 elements",
        ));
    }
    let (local, paint_hidden) = emit_element(
        element, parent, parent_style, root_start, assets, path, rules,
        records, sources, ids, runtime_requirements, ancestor_hidden, &mut computed,
    )?;
    for (index, child, style) in ordered_children(element, &computed, rules, true)? {
        emit(
            child,
            local,
            &computed,
            root_start,
            assets,
            &format!("{path}/{index}"),
            rules,
            records,
            sources,
            ids,
            runtime_requirements,
            paint_hidden,
            depth + 1,
            style,
        )?;
    }
    Ok(())
}

fn emit_element(
    element: ElementRef<'_>,
    parent: u32,
    parent_style: &Style,
    root_start: u32,
    assets: &BTreeMap<String, Asset>,
    path: &str,
    rules: &[css::Rule],
    records: &mut Vec<Record>,
    sources: &mut Vec<SourceNode>,
    ids: &mut BTreeSet<String>,
    runtime_requirements: &mut RuntimeRequirements,
    ancestor_hidden: bool,
    computed: &mut Style,
) -> Result<(u32, bool), Diagnostic> {
    let tag = element.value().name();
    if !matches!(
        tag,
        "div"
            | "section"
            | "main"
            | "article"
            | "header"
            | "footer"
            | "p"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "span"
            | "img"
    ) {
        return Err(Diagnostic::new(
            "unsupported-element",
            path,
            format!("Unsupported HTML element: {tag}"),
        ));
    }
    for (name, _) in element.value().attrs() {
        if !(matches!(name, "id" | "class" | "style" | "data-nuxie-id" | "lang")
            || name.starts_with("data-") && name.len() > 5 && name.is_ascii()
            || tag == "img" && name == "src")
        {
            return Err(Diagnostic::new(
                "unsupported-attribute",
                path,
                format!("Unsupported HTML attribute: {name}"),
            ));
        }
    }
    let id = element
        .attr("data-nuxie-id")
        .or(element.attr("id"))
        .unwrap_or(path)
        .to_owned();
    if ids.len() >= MAX_SOURCE_IDENTITIES {
        return Err(Diagnostic::new(
            "input-limit",
            path,
            "Document exceeds 8192 element and break identities",
        ));
    }
    if id.is_empty() || !ids.insert(id.clone()) {
        return Err(Diagnostic::new(
            "duplicate-id",
            path,
            format!("Empty or duplicate authored identity: {id}"),
        ));
    }
    if let Some(language) = element.attr("lang") {
        computed.language = language.to_owned();
    }
    if computed.block_text {
        // These properties describe the inner flex formatting context; they do
        // not affect a block text container. Its outer flex-item factors remain.
        computed.row = false;
        computed.reverse = false;
        computed.wrap = style::WrapMode::NoWrap;
        computed.gap = [0.0; 2];
        computed.align = 3;
        computed.justify = 0;
    }
    computed.resolve_flex(parent_style, path)?;
    let image = if tag == "img" {
        let name = element
            .attr("src")
            .and_then(|s| s.strip_prefix("asset:"))
            .ok_or_else(|| {
                Diagnostic::new("missing-image", path, "Image src must be asset:<name>")
            })?;
        let index = assets
            .iter()
            .enumerate()
            .find_map(|(i, (key, asset))| {
                (key == name && matches!(asset, Asset::Image { .. })).then_some(i as u32)
            })
            .ok_or_else(|| {
                Diagnostic::new("missing-image", path, format!("Missing image asset {name}"))
            })?;
        let auto_width = matches!(computed.width, style::Size::Auto);
        let auto_height = matches!(computed.height, style::Size::Auto);
        let intrinsic = auto_width || auto_height;
        if intrinsic && (computed.aspect_ratio.auto || computed.aspect_ratio.ratio.is_none()) {
            // Natural dimensions come from the validated embedded asset. Keep
            // automatic axes in the scene so flex and constraints resolve at
            // runtime. A loaded replaced element prefers its natural ratio for
            // auto+ratio; a bare nondegenerate authored ratio retains priority.
            let Asset::Image { bytes } = &assets[name] else { unreachable!() };
            let [width, height] = assets::image_dimensions(bytes, path)?;
            let (mut a, mut b) = (width, height);
            while b != 0 { (a, b) = (b, a % b); }
            let pair = [width / a, height / a];
            computed.aspect_ratio = aspect_ratio::AspectRatio {
                auto: true, ratio: Some(pair[0] as f32 / pair[1] as f32), pair: Some(pair),
            };
        }
        Some((index, intrinsic))
    } else {
        None
    };
    let (text, mut text_breaks) = text_content::collect(element, &computed, path, rules, ids)?;
    let text_transform =
        text_content::transform(&text, computed.text_transform, &computed.language);
    let text = if let Some(mapping) = &text_transform {
        for entry in &mut text_breaks {
            entry.text_offset = mapping.scalar_offsets[entry.text_offset as usize];
        }
        mapping.rendered.clone()
    } else {
        text
    };
    if computed.word_spacing != 0.0
        && text.chars().any(|ch| {
            matches!(
                ch,
                '\u{1361}' | '\u{10100}' | '\u{10101}' | '\u{1039f}' | '\u{1091f}'
            )
        })
    {
        return Err(Diagnostic::new(
            "unsupported-text",
            path,
            "Word spacing currently supports U+0020 and U+00A0 separators; visible-script separators need qualification",
        ));
    }
    if computed.text_ellipsis && !text.is_empty()
        && (!computed.block_text || !computed.overflow.clips_both()
            || computed.white_space != style::WhiteSpace::NoWrap
            || !text_breaks.is_empty()
            || text.chars().any(|ch| matches!(ch, '\n' | '\r' | '\t' | '\u{85}' | '\u{2028}' | '\u{2029}'))) {
        return Err(Diagnostic::new("unsupported-text-overflow-combination", path,
            "Ellipsis currently requires text-only display:block, white-space:nowrap and overflow:hidden or clip, without explicit breaks"));
    }
    let font = if !text.is_empty() {
        if element.child_elements().any(|e| e.value().name() != "br") {
            return Err(Diagnostic::new(
                "mixed-inline-content",
                path,
                "Mixed text and child elements require an inline formatting context, which is not supported yet",
            ));
        }
        Some(assets::font_for(
            assets,
            &computed.font_family,
            computed.font_weight,
            &text,
            path,
        )?)
    } else {
        None
    };
    let normal_line_height = matches!(computed.line_height, style::LineHeight::Normal);
    let normal_height = font.as_ref().map_or(0.0, |font| {
        (font.ascent * computed.font_size).round()
            + (font.descent * computed.font_size).round()
            + (font.line_gap * computed.font_size).round()
    });
    let line_height = computed.used_line_height(normal_height);
    let mut text_line_padding = [0.0_f32; 2];
    let mut text_baseline_offset = 0.0;
    if let Some(font) = &font {
        // Preserve natural line-box sizing, then offset the glyph transform to
        // the pinned Chromium baseline (rounded metrics, floored half-leading).
        // A transform admits negative corrections without negative padding or
        // changing the runtime's responsive text measurement.
        let natural = (font.ascent + font.descent) * computed.font_size;
        let top = if normal_line_height {
            0.0
        } else {
            (line_height - natural) / 2.0
        };
        let browser_ascent = (font.ascent * computed.font_size).round();
        let browser_descent = (font.descent * computed.font_size).round();
        let browser_baseline =
            browser_ascent + ((line_height - browser_ascent - browser_descent) / 2.0).floor();
        text_baseline_offset = browser_baseline - font.ascent * computed.font_size - top;
        // Normal's rounded line box may be shorter than fractional natural
        // metrics. Trim the measured final descent, then restore the exact
        // trailing space; the runtime still measures every wrapped line.
        let bottom = if normal_line_height {
            line_height - font.ascent * computed.font_size
        } else {
            line_height * font.ascent / (font.ascent + font.descent)
                - font.ascent * computed.font_size
                - top
        };
        if !top.is_finite()
            || !bottom.is_finite()
            || top < 0.0
            || bottom < 0.0
            || line_height > 1_000_000.0
        {
            return Err(Diagnostic::new(
                "unsupported-line-height",
                path,
                "Used line height must be at least the font's natural ascent plus descent and at most 1000000px",
            ));
        }
        text_line_padding = [top, bottom];
    }
    let paint_hidden = ancestor_hidden || computed.hidden;
    let local = records.len() as u32 - root_start;
    let mut layout = Record::new("LayoutComponent");
    layout.set("name", Value::String(id.clone()))?;
    if paint_hidden {
        layout.set("drawableFlags", Value::Uint(1))?;
    }
    layout.set("parentId", Value::Uint(parent))?;
    layout.set("styleId", Value::Uint(local + 1))?;
    if computed.overflow.clips_both() {
        layout.set("clip", Value::Bool(true))?;
    }
    let mut layout_style = Record::new("LayoutComponentStyle");
    computed.write(&mut layout, &mut layout_style, parent_style)?;
    if image.is_some_and(|(_, intrinsic)| intrinsic) {
        layout_style.set("intrinsicallySizedValue", Value::Bool(true))?;
    }
    records.push(layout);
    records.push(layout_style);
    let source_index = sources.len();
    sources.push(SourceNode {
        id,
        path: path.into(),
        object_id: local,
        text_run_id: None,
        text_run_ids: Vec::new(),
        text_breaks,
        text_transform,
    });
    if computed.aspect_ratio.ratio.is_some() {
        runtime_requirements.version = runtime_requirements.version.max(15);
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssAspectRatioV1);
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssAspectRatioPairV1);
        runtime_requirements.layout_aspect_ratios.push(LayoutAspectRatioRequirement {
            object_id: local, content_box: computed.aspect_ratio.auto, pair: computed.aspect_ratio.pair,
        });
    }
    if computed.content_box {
        runtime_requirements.version = runtime_requirements.version.max(13);
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssContentBoxV1);
        runtime_requirements.layout_content_box.push(local);
    }
    if computed.requires_indefinite_basis(parent_style) {
        runtime_requirements.version = runtime_requirements.version.max(12);
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssIndefiniteBasisV1);
    }
    if computed.content_auto_basis(parent_style)
        || (!text.is_empty() && computed.width == style::Size::Auto && matches!(computed.basis, style::Size::Auto | style::Size::Percent(_)))
    {
        runtime_requirements.version = runtime_requirements.version.max(11);
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssIntrinsicSizingV1);
        runtime_requirements.layout_intrinsic_sizing.push(local);
    }
    let partial_factors = [computed.grow, computed.shrink].iter().any(|v| *v > 0.0 && *v < 1.0);
    if partial_factors {
        runtime_requirements.version = runtime_requirements.version.max(10);
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssPartialFlexFactorsV1);
    }
    if computed.grow != computed.shrink || partial_factors || computed.content_auto_basis(parent_style)
        || matches!(computed.basis, style::Size::Percent(_)) {
        runtime_requirements.version = runtime_requirements.version.max(9);
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssFlexFactorsV1);
        runtime_requirements.layout_flex_factors.push(LayoutFlexFactorsRequirement {
            object_id: local, grow: computed.grow, shrink: computed.shrink,
        });
    }
    // Every wrapped container needs independent line alignment, including the
    // authoring reset's flex-start, because Rive couples it to align-items.
    if !computed.block_text && (computed.justify >= 4 || computed.align_content == AlignContent::SpaceEvenly) {
        runtime_requirements.version = runtime_requirements.version.max(8);
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssDistributedSpacingV1);
        if computed.justify >= 4 {
            runtime_requirements.layout_justify_content.push(LayoutJustifyContentRequirement {
                object_id: local,
                alignment: if computed.justify == 4 { JustifyDistribution::SpaceAround } else { JustifyDistribution::SpaceEvenly },
            });
        }
    }
    if !computed.block_text && (computed.wrap.wraps() || computed.align_content != AlignContent::FlexStart) {
        runtime_requirements.version = runtime_requirements.version.max(7);
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssAlignContentV1);
        runtime_requirements.layout_align_content.push(LayoutAlignContentRequirement {object_id:local,alignment:computed.align_content});
    }
    if computed.align_self != AlignSelf::Auto {
        runtime_requirements.version = runtime_requirements.version.max(6);
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssAlignSelfV1);
        runtime_requirements.layout_align_self.push(LayoutAlignSelfRequirement {object_id:local,alignment:computed.align_self});
    }
    let overflow = computed.overflow.computed();
    if ((overflow.x == crate::overflow::Value::Clip && overflow.y == crate::overflow::Value::Clip)
        || (image.is_some() && computed.overflow.clips_both()))
        && computed.overflow_clip_margin != crate::overflow_clip_margin::ClipMargin::default() {
        use crate::overflow_clip_margin::ClipBox;
        runtime_requirements.layout_overflow_clip_margins.push(LayoutOverflowClipMarginRequirement {
            object_id: local,
            origin: match computed.overflow_clip_margin.origin {
                ClipBox::Content => OverflowClipBox::ContentBox,
                ClipBox::Padding => OverflowClipBox::PaddingBox,
                ClipBox::Border => OverflowClipBox::BorderBox,
            },
            pixels: computed.overflow_clip_margin.pixels,
        });
    }
    if let Some(axis) = computed.overflow.axis_clip() {
        runtime_requirements.layout_axis_overflow.push(LayoutAxisOverflowRequirement { object_id:local, axis });
    }
    emit_gradient_requirement(&computed, local, path, runtime_requirements)?;
    if computed.opacity < 1.0 {
        runtime_requirements.layout_group_opacity.push(LayoutGroupOpacityRequirement {
            object_id: local, opacity: computed.opacity,
        });
    }
    if let Some(level) = computed.z_index {
        runtime_requirements.layout_stacking.push(LayoutStackingRequirement { object_id: local, level });
    }
    if computed.position.is_positioned() {
        runtime_requirements.layout_positioned.push(local);
        if computed.position == crate::position::PositionMode::Absolute {
            runtime_requirements.layout_absolute.push(local);
        }
    }
    if computed.padding.iter().any(|v| matches!(v, style::Spacing::Percent(_)))
        || computed.margin.iter().any(|v| matches!(v, style::Margin::Percent(_))) {
        runtime_requirements.version = 16;
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssPercentageSpacingV1);
        runtime_requirements.layout_percentage_spacing.push(local);
    }
    emit_corner_requirement(&computed, local, path, runtime_requirements)?;
    let has_border = emit_border_requirement(&computed, local, path, runtime_requirements)?;
    if computed.background() != 0 || computed.gradient.is_some() || computed.overflow.clips_both() || has_border {
        runtime_requirements.version = runtime_requirements.version.max(5);
        runtime_requirements.capabilities.insert(RuntimeCapability::LayoutCssPixelBoundsV1);
        runtime_requirements.layout_pixel_bounds.push(local);
    }
    if computed.background() != 0 {
        let fill_id = records.len() as u32 - root_start;
        let mut fill = Record::new("Fill");
        fill.set("parentId", Value::Uint(local))?;
        records.push(fill);
        let mut paint = Record::new("SolidColor");
        paint.set("parentId", Value::Uint(fill_id))?;
        paint.set("colorValue", Value::Color(computed.background()))?;
        records.push(paint);
    }
    if let Some((asset, _)) = image {
        let image_id = records.len() as u32 - root_start;
        let mut object = Record::new("Image");
        if paint_hidden {
            object.set("drawableFlags", Value::Uint(1))?;
        }
        object.set("parentId", Value::Uint(local))?;
        object.set("assetId", Value::Uint(asset))?;
        object.set("originX", Value::Float(0.0))?;
        object.set("originY", Value::Float(0.0))?;
        object.set("fit", Value::Uint(7))?;
        records.push(object);
        let mut participant = Record::new("LayoutParticipant");
        participant.set("parentId", Value::Uint(image_id))?;
        participant.set("layoutWidthScaleType", Value::Uint(1))?;
        participant.set("layoutHeightScaleType", Value::Uint(1))?;
        records.push(participant);
    }
    if let Some(font) = font {
        runtime_requirements
            .capabilities
            .insert(RuntimeCapability::TextCssShapingPrecisionV1);
        if text.contains('\t') {
            runtime_requirements
                .capabilities
                .insert(RuntimeCapability::TextCssTabsV1);
            if computed.white_space == style::WhiteSpace::PreWrap {
                runtime_requirements
                    .capabilities
                    .insert(RuntimeCapability::TextCssWrappedTabsV1);
            }
        }
        if text.chars().any(|ch| {
            matches!(ch as u32,
            0x1680 | 0x2000..=0x2006 | 0x2008..=0x200a | 0x205f | 0x3000)
        }) {
            runtime_requirements
                .capabilities
                .insert(RuntimeCapability::TextPreservedSpaceBreaksV1);
        }
        let intrinsic_text = computed.width == style::Size::Auto
            && matches!(computed.basis, style::Size::Auto | style::Size::Percent(_));
        // Intrinsic measurement comes from TextSizing::AutoWidth. The internal
        // containers still stretch to the resolved content width, so centered
        // and right-aligned lines retain the element's available inline space.
        // Line leading belongs to the text's intrinsic line box, not the
        // authored element's padding. Keeping it on an internal container
        // prevents short explicit heights from growing to the leading sum.
        let line_box_id = records.len() as u32 - root_start;
        let mut line_box = Record::new("LayoutComponent");
        line_box.set("parentId", Value::Uint(local))?;
        line_box.set("styleId", Value::Uint(line_box_id + 1))?;
        let mut line_style = Record::new("LayoutComponentStyle");
        line_style.set("widthUnitsValue", Value::Uint(3))?;
        line_style.set("heightUnitsValue", Value::Uint(3))?;
        line_style.set("layoutWidthScaleType", Value::Uint(1))?;
        line_style.set("layoutHeightScaleType", Value::Uint(2))?;
        line_style.set("flexDirectionValue", Value::Uint(0))?;
        line_style.set("layoutAlignmentType", Value::Uint(0))?;
        for (side, value) in ["Top", "Bottom"].into_iter().zip(text_line_padding) {
            line_style.set(&format!("padding{side}"), Value::Float(value))?;
            line_style.set(&format!("padding{side}UnitsValue"), Value::Uint(1))?;
        }
        records.push(line_box);
        records.push(line_style);
        let text_id = records.len() as u32 - root_start;
        let mut object = Record::new("Text");
        if paint_hidden {
            object.set("drawableFlags", Value::Uint(1))?;
        }
        object.set("parentId", Value::Uint(line_box_id))?;
        object.set(
            "name",
            Value::String(format!("{}:text", sources.last().unwrap().id)),
        )?;
        object.set(
            "sizingValue",
            Value::Uint(if intrinsic_text { 0 } else { 1 }),
        )?;
        object.set("alignValue", Value::Uint(computed.text_align))?;
        if computed.white_space.nowrap() {
            runtime_requirements.version = runtime_requirements.version.max(2);
            runtime_requirements
                .text_policies
                .push(TextPolicyRequirement {
                    object_id: text_id,
                    policy: if computed.text_ellipsis { TextPolicy::CssSingleLineEllipsisV1 } else { TextPolicy::CssNowrapAlignmentV1 },
                });
            runtime_requirements
                .capabilities
                .insert(if computed.text_ellipsis { RuntimeCapability::TextCssSingleLineEllipsisV1 } else { RuntimeCapability::TextCssNowrapAlignmentV1 });
            object.set("wrapValue", Value::Uint(1))?;
        }
        if computed.white_space == style::WhiteSpace::PreWrap {
            runtime_requirements.version = runtime_requirements.version.max(2);
            runtime_requirements
                .capabilities
                .insert(RuntimeCapability::TextCssPreWrapV1);
            runtime_requirements
                .text_policies
                .push(TextPolicyRequirement {
                    object_id: text_id,
                    policy: TextPolicy::CssPreWrapV1,
                });
        }
        if computed.white_space == style::WhiteSpace::PreLine {
            runtime_requirements.version = runtime_requirements.version.max(2);
            runtime_requirements
                .capabilities
                .insert(RuntimeCapability::TextCssPreLineV1);
            runtime_requirements
                .text_policies
                .push(TextPolicyRequirement {
                    object_id: text_id,
                    policy: TextPolicy::CssPreLineV1,
                });
        }
        if computed.white_space == style::WhiteSpace::Normal {
            runtime_requirements.version = runtime_requirements.version.max(2);
            runtime_requirements
                .capabilities
                .insert(RuntimeCapability::TextCssNormalWrapV1);
            runtime_requirements
                .text_policies
                .push(TextPolicyRequirement {
                    object_id: text_id,
                    policy: TextPolicy::CssNormalWrapV1,
                });
        }
        if computed
            .decoration_origins
            .iter()
            .any(|origin| origin.underline)
        {
            let lines = computed
                .decoration_origins
                .iter()
                .filter(|origin| origin.underline)
                .map(|origin| {
                    origin.resolve(
                        computed.font_size,
                        font.underline_thickness,
                        computed.decoration.skip_ink,
                        path,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            runtime_requirements.version = runtime_requirements.version.max(3);
            runtime_requirements
                .capabilities
                .insert(RuntimeCapability::TextSolidUnderlinesV1);
            runtime_requirements
                .text_underlines
                .push(TextUnderlineRequirement {
                    object_id: text_id,
                    lines,
                });
        }
        if computed
            .decoration_origins
            .iter()
            .any(|origin| origin.strikethrough)
        {
            let ascent = (font.ascent * computed.font_size).round();
            let descent = (font.descent * computed.font_size).round();
            let line_baseline = ascent + ((line_height - ascent - descent) / 2.0).floor();
            let lines = computed
                .decoration_origins
                .iter()
                .filter(|origin| origin.strikethrough)
                .map(|origin| {
                    origin.resolve_strikethrough(
                        computed.font_size,
                        font.underline_thickness,
                        font.ascent,
                        line_baseline,
                        path,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            runtime_requirements.version = runtime_requirements.version.max(4);
            runtime_requirements
                .capabilities
                .insert(RuntimeCapability::TextSolidStrikethroughsV1);
            runtime_requirements
                .text_strikethroughs
                .push(TextStrikethroughRequirement {
                    object_id: text_id,
                    lines,
                });
        }
        object.set("fitFromBaseline", Value::Bool(false))?;
        object.set("y", Value::Float(text_baseline_offset))?;
        if normal_line_height {
            object.set("verticalTrimValue", Value::Uint(1 << 8))?;
        }
        records.push(object);
        let mut participant = Record::new("LayoutParticipant");
        participant.set("parentId", Value::Uint(text_id))?;
        participant.set(
            "layoutWidthScaleType",
            Value::Uint(1),
        )?;
        participant.set("layoutHeightScaleType", Value::Uint(2))?;
        records.push(participant);
        // Keep native line breaking over one Text object. Only separator runs
        // receive additional advance; normal spacing preserves the original run.
        let mut parts: Vec<(String, bool)> = Vec::new();
        if computed.word_spacing == 0.0 {
            parts.push((text, false));
        } else {
            for ch in text.chars() {
                let separator = matches!(ch, ' ' | '\u{a0}' | '\t');
                if let Some((part, _)) = parts
                    .last_mut()
                    .filter(|(_, previous)| *previous == separator)
                {
                    part.push(ch);
                } else {
                    if parts.len() >= 4096 {
                        return Err(Diagnostic::new(
                            "input-limit",
                            path,
                            "Word spacing exceeds 4096 text runs in one element",
                        ));
                    }
                    parts.push((ch.to_string(), separator));
                }
            }
        }
        let mut style_ids = [None, None];
        for (_, separator) in &parts {
            let index = usize::from(*separator);
            if style_ids[index].is_some() {
                continue;
            }
            let style_id = records.len() as u32 - root_start;
            style_ids[index] = Some(style_id);
            let mut text_style = Record::new("TextStylePaint");
            text_style.set("parentId", Value::Uint(text_id))?;
            text_style.set("fontAssetId", Value::Uint(font.id))?;
            text_style.set("fontSize", Value::Float(computed.font_size))?;
            let spacing = computed.letter_spacing
                + if *separator {
                    computed.word_spacing
                } else {
                    0.0
                };
            if spacing != 0.0 {
                runtime_requirements
                    .capabilities
                    .insert(RuntimeCapability::TextCssLetterSpacingV1);
                text_style.set("letterSpacing", Value::Float(spacing))?;
            }
            text_style.set("lineHeight", Value::Float(line_height))?;
            records.push(text_style);
            let fill_id = records.len() as u32 - root_start;
            let mut fill = Record::new("Fill");
            fill.set("parentId", Value::Uint(style_id))?;
            records.push(fill);
            let mut paint = Record::new("SolidColor");
            paint.set("parentId", Value::Uint(fill_id))?;
            paint.set("colorValue", Value::Color(computed.color))?;
            records.push(paint);
        }
        for (text, separator) in parts {
            sources[source_index]
                .text_run_ids
                .push(records.len() as u32 - root_start);
            let mut run = Record::new("TextValueRun");
            run.set("parentId", Value::Uint(text_id))?;
            run.set(
                "styleId",
                Value::Uint(style_ids[usize::from(separator)].unwrap()),
            )?;
            run.set("text", Value::String(text))?;
            records.push(run);
        }
        if sources[source_index].text_run_ids.len() == 1 {
            sources[source_index].text_run_id = sources[source_index].text_run_ids.first().copied();
        }
    }
    Ok((local, paint_hidden))
}
