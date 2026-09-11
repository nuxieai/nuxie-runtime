//! Ordinary layout/fill lowering. Admission grows only with baseline evidence.
use crate::{color, css, wire::{self, Record, Value}, CompileInput, CompileOutput, Diagnostic, SourceNode};
use scraper::{ElementRef, Html};
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
enum Size { Auto, Pixels(f32), Percent(f32) }
#[derive(Clone, Copy)]
enum BackgroundColor { Rgba(u32), CurrentColor }
impl BackgroundColor {
    fn used(self, foreground: u32) -> u32 {
        match self { Self::Rgba(color) => color, Self::CurrentColor => foreground }
    }
}
#[derive(Clone)]
struct Style { width: Size, height: Size, foreground: u32, background: BackgroundColor }
impl Default for Style {
    fn default() -> Self { Self { width: Size::Auto, height: Size::Auto, foreground: 0xff000000, background: BackgroundColor::Rgba(0) } }
}

fn unsupported(source: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("unsupported-target-semantics", source, message)
}

fn size(text: &str, source: &str) -> Result<Size, Diagnostic> {
    let text = text.trim().to_ascii_lowercase();
    if text == "auto" { return Ok(Size::Auto); }
    let (number, percent) = if let Some(v) = text.strip_suffix('%') { (v, true) }
        else if let Some(v) = text.strip_suffix("px") { (v, false) }
        else if text == "0" { ("0", false) }
        else { return Err(unsupported(source, "This target currently admits auto, finite nonnegative px and percentage sizes")); };
    let value = number.parse::<f32>().ok().filter(|v| v.is_finite() && *v >= 0. && *v <= 1_000_000.)
        .ok_or_else(|| unsupported(source, "Size must be finite and between 0 and 1000000"))?;
    Ok(if percent { Size::Percent(value) } else { Size::Pixels(value) })
}

fn validate(d: &css::Declaration) -> Result<(), Diagnostic> {
    let value = css::ordinary_value(&d.value)?.trim().to_ascii_lowercase();
    match d.name.as_str() {
        "width" | "height" => { size(&value, &d.source)?; }
        "background-color" | "color" => {
            if !["currentcolor", "inherit", "initial", "unset"].contains(&value.as_str()) { color::parse(&value, &d.source)?; }
        }
        "display" if value == "flex" => {}
        "flex-direction" if value == "column" => {}
        _ => return Err(unsupported(&d.source, format!("{}: {} has no admitted ordinary-Rive lowering yet", d.name, d.value))),
    }
    Ok(())
}

fn computed(element: ElementRef<'_>, rules: &[css::Rule], parent: &Style) -> Result<Style, Diagnostic> {
    let mut declarations = css::cascade(rules, element)?;
    for d in &mut declarations { d.value = css::ordinary_value(&d.value)?; validate(d)?; }
    let mut style = Style { foreground: parent.foreground, ..Style::default() };
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
            "width" => style.width = size(&d.value, &d.source)?,
            "height" => style.height = size(&d.value, &d.source)?,
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
