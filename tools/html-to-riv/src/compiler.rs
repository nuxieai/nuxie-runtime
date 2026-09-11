//! Ordinary layout/fill lowering. Admission grows only with baseline evidence.
use crate::{color, css, wire::{self, Record, Value}, CompileInput, CompileOutput, Diagnostic, SourceNode};
use scraper::{ElementRef, Html};
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
enum Size { Auto, Pixels(f32), Percent(f32) }
#[derive(Clone, Copy)]
enum SpecifiedSize { Auto, Pixels(f32), Percent(f32), Em(f32), Rem(f32) }
const ROOT_FONT_SIZE: f32 = 16.;
const MAX_SIZE: f32 = 1_000_000.;
#[derive(Clone, Copy)]
enum BackgroundColor { Rgba(u32), CurrentColor }
impl BackgroundColor {
    fn used(self, foreground: u32) -> u32 {
        match self { Self::Rgba(color) => color, Self::CurrentColor => foreground }
    }
}
#[derive(Clone)]
struct Style { width: Size, height: Size, font_size: f32, foreground: u32, background: BackgroundColor }
impl Default for Style {
    fn default() -> Self { Self { width: Size::Auto, height: Size::Auto, font_size: ROOT_FONT_SIZE, foreground: 0xff000000, background: BackgroundColor::Rgba(0) } }
}

fn unsupported(source: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("unsupported-target-semantics", source, message)
}

fn size(text: &str, source: &str) -> Result<SpecifiedSize, Diagnostic> {
    let text = text.trim().to_ascii_lowercase();
    if text == "auto" { return Ok(SpecifiedSize::Auto); }
    let (number, unit): (&str, fn(f32) -> SpecifiedSize) =
        if let Some(v) = text.strip_suffix("rem") { (v, SpecifiedSize::Rem) }
        else if let Some(v) = text.strip_suffix("em") { (v, SpecifiedSize::Em) }
        else if let Some(v) = text.strip_suffix('%') { (v, SpecifiedSize::Percent) }
        else if let Some(v) = text.strip_suffix("px") { (v, SpecifiedSize::Pixels) }
        else if text == "0" { ("0", SpecifiedSize::Pixels) }
        else { return Err(unsupported(source, "This target currently admits auto, finite nonnegative px, em, rem and percentage sizes")); };
    let value = number.parse::<f32>().ok().filter(|v| v.is_finite() && *v >= 0. && *v <= MAX_SIZE)
        .ok_or_else(|| unsupported(source, "Size must be finite and between 0 and 1000000"))?;
    Ok(unit(value))
}

fn resolved_length(value: f32, basis: f32, source: &str) -> Result<f32, Diagnostic> {
    let result = value * basis;
    if !result.is_finite() || !(0. ..=MAX_SIZE).contains(&result) {
        return Err(unsupported(source, "Computed font-relative size must be finite and between 0 and 1000000"));
    }
    Ok(result)
}

// Inheritance copies computed descriptors, not dimensions resolved at the
// parent's containing block. Percentages remain responsive at the receiving box;
// font-relative lengths have already become absolute at the parent.
fn computed_size(text: &str, inherited: Size, font_size: f32, source: &str) -> Result<Size, Diagnostic> {
    match text.trim().to_ascii_lowercase().as_str() {
        "inherit" => Ok(inherited),
        "initial" | "unset" => Ok(Size::Auto),
        _ => Ok(match size(text, source)? {
            SpecifiedSize::Auto => Size::Auto,
            SpecifiedSize::Pixels(v) => Size::Pixels(v),
            SpecifiedSize::Percent(v) => Size::Percent(v),
            SpecifiedSize::Em(v) => Size::Pixels(resolved_length(v, font_size, source)?),
            SpecifiedSize::Rem(v) => Size::Pixels(resolved_length(v, ROOT_FONT_SIZE, source)?),
        }),
    }
}

fn computed_font_size(text: &str, parent: f32, source: &str) -> Result<f32, Diagnostic> {
    match text.trim().to_ascii_lowercase().as_str() {
        "inherit" | "unset" => Ok(parent),
        "initial" => Ok(ROOT_FONT_SIZE),
        _ => match size(text, source)? {
            SpecifiedSize::Pixels(v) => Ok(v),
            SpecifiedSize::Em(v) => resolved_length(v, parent, source),
            SpecifiedSize::Rem(v) => resolved_length(v, ROOT_FONT_SIZE, source),
            SpecifiedSize::Percent(v) => resolved_length(v / 100., parent, source),
            SpecifiedSize::Auto => Err(unsupported(source, "font-size:auto is invalid; use a nonnegative px, em, rem or percentage size")),
        },
    }
}

fn background_shorthand(value: &str, parent: BackgroundColor, source: &str) -> Result<BackgroundColor, Diagnostic> {
    let value = css::ordinary_value(value).map_err(|_| unsupported(source,
        "background supports one solid color, none, inherit, initial or unset; other image/layer/position/size/repeat/box/attachment values are not admitted"))?;
    match value.trim().to_ascii_lowercase().as_str() {
        "inherit" => Ok(parent),
        "none" | "initial" | "unset" => Ok(BackgroundColor::Rgba(0)),
        "currentcolor" => Ok(BackgroundColor::CurrentColor),
        _ => color::parse(&value, source).map(BackgroundColor::Rgba).map_err(|_| unsupported(source,
            "background supports one solid color, none, inherit, initial or unset; mixed values and background layers are not admitted")),
    }
}

fn apply_background_shorthand(style: &mut Style, parent: &Style, value: &str, source: &str) -> Result<(), Diagnostic> {
    // This is a shorthand operation, not a permanent background-color alias.
    // When other background longhands are admitted, color/none reset image:none,
    // position:0% 0%, size:auto auto, repeat:repeat, origin:padding-box,
    // clip:border-box and attachment:scroll at this declaration's cascade priority.
    // inherit copies every computed subproperty; initial/unset reset every one.
    // background-blend-mode is NOT part of this shorthand. The current profile
    // admits no other background constituents, so their initial state is invariant.
    style.background = background_shorthand(value, parent.background, source)?;
    Ok(())
}

fn validate(d: &css::Declaration) -> Result<(), Diagnostic> {
    if d.name == "background" {
        background_shorthand(&d.value, BackgroundColor::Rgba(0), &d.source)?;
        return Ok(());
    }
    let value = css::ordinary_value(&d.value)?.trim().to_ascii_lowercase();
    match d.name.as_str() {
        "width" | "height" => {
            if !["inherit", "initial", "unset"].contains(&value.as_str()) { size(&value, &d.source)?; }
        }
        "font-size" => {
            if !["inherit", "initial", "unset"].contains(&value.as_str())
                && matches!(size(&value, &d.source)?, SpecifiedSize::Auto) {
                return Err(unsupported(&d.source, "font-size:auto is invalid; use a nonnegative px, em, rem or percentage size"));
            }
        }
        "background-color" | "color" => {
            if !["currentcolor", "inherit", "initial", "unset"].contains(&value.as_str()) { color::parse(&value, &d.source)?; }
        }
        // All admitted elements and the explicit host compute to flex/column.
        // CSS initial/unset would mean inline/row, not the authoring reset.
        "display" if matches!(value.as_str(), "flex" | "inherit") => {}
        "flex-direction" if matches!(value.as_str(), "column" | "inherit") => {}
        _ => return Err(unsupported(&d.source, format!("{}: {} has no admitted ordinary-Rive lowering yet", d.name, d.value))),
    }
    Ok(())
}

fn computed(element: ElementRef<'_>, rules: &[css::Rule], parent: &Style) -> Result<Style, Diagnostic> {
    let mut declarations = css::cascade(rules, element)?;
    for d in &mut declarations {
        validate(d)?;
        d.value = css::ordinary_value(&d.value)?;
    }
    let mut style = Style { foreground: parent.foreground, font_size: parent.font_size, ..Style::default() };
    // Font-size-relative units use the parent for font-size itself, but the final
    // computed element font size for other lengths, regardless of source order.
    for d in declarations.iter().filter(|d| d.name == "font-size") {
        style.font_size = computed_font_size(&d.value, parent.font_size, &d.source)?;
    }
    // Resolve currentColor against the final computed color, irrespective of
    // declaration order. A background currentColor stays a computed keyword
    // through inheritance and resolves against the receiving element at emission.
    for d in declarations.iter().filter(|d| d.name == "color") {
        let value = d.value.trim().to_ascii_lowercase();
        style.foreground = match value.as_str() {
            "inherit" | "unset" | "currentcolor" => parent.foreground,
            "initial" => 0xff000000,
            _ => color::parse(&value, &d.source)?,
        };
    }
    for d in &declarations {
        match d.name.as_str() {
            "width" => style.width = computed_size(&d.value, parent.width, style.font_size, &d.source)?,
            "height" => style.height = computed_size(&d.value, parent.height, style.font_size, &d.source)?,
            "background" => apply_background_shorthand(&mut style, parent, &d.value, &d.source)?,
            "background-color" => {
                style.background = match d.value.trim().to_ascii_lowercase().as_str() {
                    "currentcolor" => BackgroundColor::CurrentColor,
                    "inherit" => parent.background,
                    "initial" | "unset" => BackgroundColor::Rgba(0),
                    _ => BackgroundColor::Rgba(color::parse(&d.value, &d.source)?),
                };
            }
            _ => {}
        }
    }
    Ok(style)
}

pub(super) fn compile(input: &CompileInput) -> Result<CompileOutput, Diagnostic> {
    if ![input.width, input.height].into_iter().all(|v| v.is_finite() && v > 0. && v <= 16384.) {
        return Err(Diagnostic::new("invalid-viewport", "viewport", "Dimensions must be finite and in (0,16384]"));
    }
    if input.html.len().saturating_add(input.css.len()) > 1_048_576 {
        return Err(Diagnostic::new("input-limit", "document", "HTML and CSS exceed 1 MiB"));
    }
    let fragment = Html::parse_fragment(&input.html);
    if !fragment.errors.is_empty() { return Err(Diagnostic::new("html-syntax", "html", "Malformed HTML fragment")); }
    let document = Html::parse_document(&format!("<!doctype html><html><head></head><body>{}</body></html>", input.html));
    let body = document.root_element().child_elements().find(|e| e.value().name() == "body")
        .ok_or_else(|| Diagnostic::new("html-syntax", "html", "Missing document body"))?;
    let rules = css::stylesheet(&input.css)?;
    css::validate_rules(&rules, validate)?;
    for host in [document.root_element(), body] {
        if !css::cascade(&rules, host)?.is_empty() {
            return Err(unsupported("document", "Rules matching host html/body are not admitted yet; style authored box elements"));
        }
    }
    let mut records = vec![Record::new("Backboard")];
    let mut artboard = Record::new("Artboard");
    artboard.set("name", Value::String("HTML".into()))?;
    artboard.set("width", Value::Float(input.width))?; artboard.set("height", Value::Float(input.height))?;
    artboard.set("styleId", Value::Uint(1))?; records.push(artboard);
    let mut root_style = Record::new("LayoutComponentStyle");
    root_style.set("flexDirectionValue", Value::Uint(0))?;records.push(root_style);
    // The authoring reset has an opaque white host. Encode it in the file;
    // relying on the caller's canvas clear color would change the design.
    let mut host_fill = Record::new("Fill");
    host_fill.set("parentId", Value::Uint(0))?;
    let host_fill_id = records.len() as u32 - 1;
    records.push(host_fill);
    let mut host_paint = Record::new("SolidColor");
    host_paint.set("parentId", Value::Uint(host_fill_id))?;
    host_paint.set("colorValue", Value::Color(0xffffffff))?;
    records.push(host_paint);
    let mut output = Emitter { records, map: Vec::new(), ids: BTreeSet::new() };
    // Match the fixed host body in reset.css for inherited computed values.
    let host_style = Style { width: Size::Percent(100.), height: Size::Percent(100.),
        background: BackgroundColor::Rgba(0xffffffff), ..Style::default() };
    output.children(body, 0, &host_style, &rules, "", 0)?;
    if output.map.is_empty() { return Err(Diagnostic::new("empty-document", "html", "At least one box element is required")); }
    Ok(CompileOutput { riv: wire::encode(&output.records)?, source_map: output.map })
}

struct Emitter { records: Vec<Record>, map: Vec<SourceNode>, ids: BTreeSet<String> }
impl Emitter {
    fn children(&mut self, parent: ElementRef<'_>, parent_id: u32, parent_style: &Style, rules: &[css::Rule], path: &str, depth: usize) -> Result<(), Diagnostic> {
        if depth > 128 { return Err(Diagnostic::new("depth-limit", path, "HTML nesting exceeds 128")); }
        let mut index = 0;
        for node in parent.children() {
            if node.value().as_text().is_some_and(|t| !t.trim().is_empty()) {
                return Err(unsupported(path, "Text rendering is not yet requalified on the immutable runtime"));
            }
            let Some(element) = ElementRef::wrap(node) else { continue; };
            let path = format!("{path}/{index}");index += 1;
            if !["div", "section", "article", "main", "header", "footer", "aside", "nav"].contains(&element.value().name()) {
                return Err(unsupported(&path, format!("Element {} is not admitted", element.value().name())));
            }
            for (name, _) in element.value().attrs() {
                if !["id", "class", "style"].contains(&name) { return Err(unsupported(&path, format!("Attribute {name} is not admitted"))); }
            }
            if self.map.len() >= 8192 { return Err(Diagnostic::new("object-limit", &path, "Document exceeds 8192 authored elements")); }
            let id = element.attr("id").map(str::to_owned).unwrap_or_else(|| format!("node{path}"));
            if id.is_empty() || !self.ids.insert(id.clone()) { return Err(Diagnostic::new("duplicate-id", &path, "Empty or duplicate element identity")); }
            let style = computed(element, rules, parent_style)?;
            if matches!(style.height, Size::Percent(_)) && matches!(parent_style.height, Size::Auto) && parent_id != 0 {
                return Err(unsupported(&path, "Percentage height inside an auto-height parent needs an immutable-target encoding proof"));
            }
            let object_id = self.records.len() as u32 - 1;
            let mut layout = Record::new("LayoutComponent");
            layout.set("name", Value::String(id.clone()))?;layout.set("parentId", Value::Uint(parent_id))?;
            layout.set("styleId", Value::Uint(object_id + 1))?;
            let mut layout_style = Record::new("LayoutComponentStyle");
            layout_style.set("flexDirectionValue", Value::Uint(0))?;
            for (axis, size) in [("width", style.width), ("height", style.height)] {
                let (value, units, scale) = match size {
                    Size::Pixels(v) => (v, 1, 0), Size::Percent(v) => (v, 2, 0),
                    Size::Auto if axis == "width" => (100., 2, 0), Size::Auto => (0., 3, 2),
                };
                layout.set(axis, Value::Float(value))?;
                layout_style.set(&format!("{axis}UnitsValue"), Value::Uint(units))?;
                layout_style.set(if axis == "width" { "layoutWidthScaleType" } else { "layoutHeightScaleType" }, Value::Uint(scale))?;
            }
            self.records.push(layout);self.records.push(layout_style);
            let background = style.background.used(style.foreground);
            if background >> 24 != 0 {
                let fill_id = self.records.len() as u32 - 1;
                let mut fill = Record::new("Fill");fill.set("parentId", Value::Uint(object_id))?;self.records.push(fill);
                let mut paint = Record::new("SolidColor");paint.set("parentId", Value::Uint(fill_id))?;
                paint.set("colorValue", Value::Color(background))?;self.records.push(paint);
            }
            self.map.push(SourceNode { id, path: path.clone(), object_id });
            self.children(element, object_id, &style, rules, &path, depth + 1)?;
        }
        Ok(())
    }
}
