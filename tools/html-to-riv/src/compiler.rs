//! Ordinary layout/fill lowering. Admission grows only with baseline evidence.
use crate::{color, css, variables, wire::{self, Record, Value}, CompileInput, CompileOutput, Diagnostic, SourceNode};
use scraper::{ElementRef, Html};
use std::collections::BTreeSet;
#[path = "baseline.rs"]
mod baseline;
#[path = "spacing.rs"]
mod spacing;
#[path = "margins.rs"]
mod margins;
#[allow(dead_code)]
#[path = "flex.rs"]
mod flex;
// Staged lowering, exercised against frozen ordinary-file experiments before
// public wrapping admission and its contextual guards are installed.
#[allow(dead_code)]
#[path = "wrapping.rs"]
mod wrapping;
#[allow(dead_code)]
#[path = "wrapping_paint.rs"]
mod wrapping_paint;

#[derive(Clone, Copy)]
enum Size { Auto, Pixels(f32), Percent(f32) }
#[derive(Clone, Copy)]
enum Direction { Column, ColumnReverse, Row, RowReverse }
impl Direction {
    // Baseline Rive paints siblings in reverse file order. Pinned Chrome
    // paints ordinary flex fragments in their visual line order, including
    // reversal for reverse directions. Reverse ordinary-direction siblings
    // and compensate layout with reversed flow/end alignment; reverse CSS
    // directions already have the required paint order. Keep these target
    // mappings separate from computed CSS inheritance and axis semantics.
    fn wire(self) -> u32 { if self.is_row() { 3 } else { 1 } }
    fn reverses_emission(self) -> bool { matches!(self, Self::Column | Self::Row) }
    fn alignment(self) -> u32 {
        if !self.reverses_emission() { 0 } else if self.is_row() { 2 } else { 6 }
    }
    fn is_row(self) -> bool { matches!(self, Self::Row | Self::RowReverse) }
}
fn computed_direction(text: &str, parent: Direction, source: &str) -> Result<Direction, Diagnostic> {
    match text.trim().to_ascii_lowercase().as_str() {
        "column" => Ok(Direction::Column), "column-reverse" => Ok(Direction::ColumnReverse),
        "row" | "initial" | "unset" => Ok(Direction::Row), "row-reverse" => Ok(Direction::RowReverse),
        "inherit" => Ok(parent),
        _ => Err(unsupported(source, "flex-direction admits column, column-reverse, row, row-reverse, inherit, initial and unset")),
    }
}
#[derive(Clone, Copy)]
enum AlignmentPosition { Auto, Normal, Stretch, FlexStart, Start, SelfStart, Center, FlexEnd, End, SelfEnd, Baseline, FirstBaseline, LastBaseline }
#[derive(Clone, Copy)]
enum OverflowAlignment { Default, Safe, Unsafe }
#[derive(Clone, Copy)]
struct SelfAlignment { position: AlignmentPosition, overflow: OverflowAlignment }
impl SelfAlignment {
    const AUTO: Self = Self { position: AlignmentPosition::Auto, overflow: OverflowAlignment::Default };
    fn is_baseline(self) -> bool { matches!(self.position, AlignmentPosition::Baseline | AlignmentPosition::FirstBaseline | AlignmentPosition::LastBaseline) }
    fn is_last_baseline(self) -> bool { matches!(self.position, AlignmentPosition::LastBaseline) }
    fn stretches(self) -> bool { matches!(self.position, AlignmentPosition::Auto | AlignmentPosition::Normal | AlignmentPosition::Stretch) }
    fn is_center(self) -> bool { matches!(self.position, AlignmentPosition::Center) }
    fn is_end(self) -> bool { matches!(self.position, AlignmentPosition::FlexEnd | AlignmentPosition::End | AlignmentPosition::SelfEnd) }
    fn is_safe(self) -> bool { matches!(self.overflow, OverflowAlignment::Safe) }
    fn wrapper_alignment(self, parent: Direction) -> Option<u32> {
        if !self.is_center() && !self.is_end() { return None; }
        // Perpendicular native flow is reversed. Safe alignment uses auto
        // margins for positive free space and physical-start justification
        // when those margins collapse on overflow.
        Some(if self.is_safe() { if parent.is_row() { 6 } else { 2 } }
            else if self.is_center() { if parent.is_row() { 3 } else { 1 } }
            else { 0 })
    }
    fn auto_margins(self, parent: Direction) -> [bool; 4] {
        let mut margins = [false; 4]; // physical left, top, right, bottom
        if self.is_safe() && (self.is_center() || self.is_end()) {
            let start = if parent.is_row() { 1 } else { 0 };
            margins[start] = true;
            if self.is_center() { margins[start + 2] = true; }
        }
        margins
    }
}
fn computed_alignment(text: &str, parent: SelfAlignment, source: &str) -> Result<SelfAlignment, Diagnostic> {
    let text = text.trim().to_ascii_lowercase();
    let tokens: Vec<_> = text.split_ascii_whitespace().collect();
    let (word, overflow) = match tokens.as_slice() {
        ["inherit"] => return Ok(parent),
        ["initial" | "unset"] => return Ok(SelfAlignment::AUTO),
        ["last", "baseline"] => return Ok(SelfAlignment { position: AlignmentPosition::LastBaseline, overflow: OverflowAlignment::Default }),
        ["first", "baseline"] => return Ok(SelfAlignment { position: AlignmentPosition::FirstBaseline, overflow: OverflowAlignment::Default }),
        [word] => (*word, OverflowAlignment::Default),
        ["safe", word] => (*word, OverflowAlignment::Safe),
        ["unsafe", word] => (*word, OverflowAlignment::Unsafe),
        _ => return Err(unsupported(source, "align-self requires a supported keyword, optionally prefixed by safe or unsafe for positional alignment")),
    };
    let position = match word {
        "baseline" => AlignmentPosition::Baseline,
        "auto" => AlignmentPosition::Auto, "normal" => AlignmentPosition::Normal,
        "stretch" => AlignmentPosition::Stretch, "flex-start" => AlignmentPosition::FlexStart,
        "start" => AlignmentPosition::Start, "self-start" => AlignmentPosition::SelfStart,
        "center" => AlignmentPosition::Center, "flex-end" => AlignmentPosition::FlexEnd,
        "end" => AlignmentPosition::End, "self-end" => AlignmentPosition::SelfEnd,
        _ => return Err(unsupported(source, "This align-self value needs ordinary-file validation; baseline alignment is unresolved")),
    };
    if !matches!(overflow, OverflowAlignment::Default)
        && matches!(position, AlignmentPosition::Auto | AlignmentPosition::Normal | AlignmentPosition::Stretch | AlignmentPosition::Baseline | AlignmentPosition::FirstBaseline | AlignmentPosition::LastBaseline) {
        return Err(unsupported(source, "safe and unsafe require positional alignment, not auto, normal or stretch"));
    }
    // Preserve specified logical/self-relative position and overflow preference
    // through inheritance. Their current lowering assumes horizontal LTR boxes.
    Ok(SelfAlignment { position, overflow })
}
fn computed_order(text: &str, parent: i32, source: &str) -> Result<i32, Diagnostic> {
    // Inspect the authored integer lexeme: cssparser's integer field saturates
    // out-of-range values, and ordinary number serialization can lose precision.
    let mut input = cssparser::ParserInput::new(text);
    let mut parser = cssparser::Parser::new(&mut input);
    parser.skip_whitespace();
    let start = parser.position();
    let token = parser.next().map_err(|_| unsupported(source, "order requires one signed 32-bit integer or CSS-wide keyword"))?.clone();
    let value = match token {
        cssparser::Token::Ident(keyword) if keyword.eq_ignore_ascii_case("inherit") => parent,
        cssparser::Token::Ident(keyword) if keyword.eq_ignore_ascii_case("initial") || keyword.eq_ignore_ascii_case("unset") => 0,
        cssparser::Token::Number { int_value: Some(_), .. } => parser.slice_from(start).parse::<i32>()
            .map_err(|_| unsupported(source, "order integer must be between -2147483648 and 2147483647"))?,
        _ => return Err(unsupported(source, "order requires one signed 32-bit integer or CSS-wide keyword")),
    };
    parser.expect_exhausted().map_err(|_| unsupported(source, "order requires exactly one integer or keyword"))?;
    Ok(value)
}
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
struct Style { margins: margins::Margins, spacing: spacing::Spacing, self_alignment: SelfAlignment, order: i32, direction: Direction, variables: variables::Variables, width: Size, height: Size, min_width: Size, min_height: Size, max_width: Size, max_height: Size, font_size: f32, foreground: u32, background: BackgroundColor }
impl Default for Style {
    fn default() -> Self { Self { margins: margins::Margins::default(), spacing: spacing::Spacing::Normal, self_alignment: SelfAlignment::AUTO, order: 0, direction: Direction::Column, variables: variables::Variables::default(), width: Size::Auto, height: Size::Auto, min_width: Size::Pixels(0.), min_height: Size::Pixels(0.), max_width: Size::Auto, max_height: Size::Auto, font_size: ROOT_FONT_SIZE, foreground: 0xff000000, background: BackgroundColor::Rgba(0) } }
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

// Auto represents an absent maximum or an automatic minimum, distinguished
// when emitting ordinary units. The authoring reset minimum stays explicit zero.
fn computed_bound(text: &str, inherited: Size, font_size: f32, minimum: bool, source: &str) -> Result<Size, Diagnostic> {
    match text.trim().to_ascii_lowercase().as_str() {
        "inherit" => Ok(inherited),
        "none" | "initial" | "unset" if !minimum => Ok(Size::Auto),
        "auto" | "initial" | "unset" if minimum => Ok(Size::Auto),
        "auto" | "none" => Err(unsupported(source, "min-size:none and max-size:auto are invalid")),
        _ => computed_size(text, inherited, font_size, source),
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
    if d.name.starts_with("--") {
        return variables::validate_value(&d.value, &d.source);
    }
    if variables::contains_var(&d.value) {
        if !["width", "height", "min-width", "min-height", "max-width", "max-height",
             "font-size", "background", "background-color", "color", "display", "flex-direction", "order", "align-self", "justify-content", "margin", "margin-left", "margin-top", "margin-right", "margin-bottom"].contains(&d.name.as_str()) {
            return Err(unsupported(&d.source, format!("{} has no admitted ordinary-Rive lowering yet", d.name)));
        }
        return variables::validate_value(&d.value, &d.source);
    }
    if d.name == "order" { computed_order(&d.value, 0, &d.source)?; return Ok(()); }
    if d.name == "background" {
        background_shorthand(&d.value, BackgroundColor::Rgba(0), &d.source)?;
        return Ok(());
    }
    let value = css::ordinary_value(&d.value)?.trim().to_ascii_lowercase();
    match d.name.as_str() {
        "width" | "height" => {
            if !["inherit", "initial", "unset"].contains(&value.as_str()) { size(&value, &d.source)?; }
        }
        "min-width" | "min-height" | "max-width" | "max-height" => {
            computed_bound(&value, Size::Pixels(0.), 0., d.name.starts_with("min-"), &d.source)?;
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
        // Display remains flex; direction is independent of the column authoring reset.
        "display" if matches!(value.as_str(), "flex" | "inherit") => {}
        "margin" | "margin-left" | "margin-top" | "margin-right" | "margin-bottom" => { margins::Margins::default().apply(&d.name, &value, margins::Margins::default(), &d.source)?; }
        "justify-content" => { spacing::computed(&value, spacing::Spacing::Normal, &d.source)?; }
        "align-self" => { computed_alignment(&value, SelfAlignment::AUTO, &d.source)?; }
        "flex-direction" => { computed_direction(&value, Direction::Column, &d.source)?; }
        _ => return Err(unsupported(&d.source, format!("{}: {} has no admitted ordinary-Rive lowering yet", d.name, d.value))),
    }
    Ok(())
}

// Sorting needs only one scalar per sibling. Do not retain full Styles here:
// each contains an inherited custom-property environment of up to 1 MiB.
// Full computation below still validates every declaration, including losers.
fn ordering_key(element: ElementRef<'_>, rules: &[css::Rule], parent: &Style) -> Result<i32, Diagnostic> {
    let declarations = css::cascade(rules, element)?;
    let Some(declaration) = declarations.iter().rev().find(|d| d.name == "order") else { return Ok(0); };
    if !variables::contains_var(&declaration.value) {
        return computed_order(&declaration.value, parent.order, &declaration.source);
    }
    let environment = variables::compute(&declarations, &parent.variables)?;
    let value = variables::substitute(&declaration.value, &environment, &declaration.source)?
        .ok_or_else(|| unsupported(&declaration.source, "Missing or cyclic custom property without a usable fallback; computed-value invalidation is not admitted"))?;
    computed_order(&value, parent.order, &declaration.source)
}

fn computed(element: ElementRef<'_>, rules: &[css::Rule], parent: &Style) -> Result<Style, Diagnostic> {
    let mut declarations = css::cascade(rules, element)?;
    for d in &declarations { validate(d)?; }
    let variable_values = variables::compute(&declarations, &parent.variables)?;
    declarations.retain(|d| !d.name.starts_with("--"));
    for d in &mut declarations {
        if variables::contains_var(&d.value) {
            d.value = variables::substitute(&d.value, &variable_values, &d.source)?
                .ok_or_else(|| unsupported(&d.source, "Missing or cyclic custom property without a usable fallback; computed-value invalidation is not admitted"))?;
            // Substitution cannot bypass property admission, even for a declaration
            // that loses the cascade. Keep the compiler's strict diagnostics.
            validate(d)?;
        }
        if d.name != "order" { d.value = css::ordinary_value(&d.value)?; }
    }
    let mut style = Style { variables: variable_values, foreground: parent.foreground, font_size: parent.font_size, ..Style::default() };
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
            "margin" | "margin-left" | "margin-top" | "margin-right" | "margin-bottom" => style.margins.apply(&d.name, &d.value, parent.margins, &d.source)?,
            "justify-content" => style.spacing = spacing::computed(&d.value, parent.spacing, &d.source)?,
            "align-self" => style.self_alignment = computed_alignment(&d.value, parent.self_alignment, &d.source)?,
            "order" => style.order = computed_order(&d.value, parent.order, &d.source)?,
            "flex-direction" => style.direction = computed_direction(&d.value, parent.direction, &d.source)?,
            "width" => style.width = computed_size(&d.value, parent.width, style.font_size, &d.source)?,
            "height" => style.height = computed_size(&d.value, parent.height, style.font_size, &d.source)?,
            "min-width" => style.min_width = computed_bound(&d.value, parent.min_width, style.font_size, true, &d.source)?,
            "min-height" => style.min_height = computed_bound(&d.value, parent.min_height, style.font_size, true, &d.source)?,
            "max-width" => style.max_width = computed_bound(&d.value, parent.max_width, style.font_size, false, &d.source)?,
            "max-height" => style.max_height = computed_bound(&d.value, parent.max_height, style.font_size, false, &d.source)?,
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
    root_style.set("flexDirectionValue", Value::Uint(Direction::Column.wire()))?;
    root_style.set("layoutAlignmentType", Value::Uint(Direction::Column.alignment()))?;records.push(root_style);
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
    // File object order serves native drawing; public identities stay in DOM
    // preorder, with numeric path components (so /2 precedes /10).
    output.map.sort_by_cached_key(|node| node.path.split('/').skip(1)
        .map(|part| part.parse::<usize>().expect("generated numeric DOM path")).collect::<Vec<_>>());
    if output.map.is_empty() { return Err(Diagnostic::new("empty-document", "html", "At least one box element is required")); }
    Ok(CompileOutput { riv: wire::encode(&output.records)?, source_map: output.map })
}

struct Emitter { records: Vec<Record>, map: Vec<SourceNode>, ids: BTreeSet<String> }
impl Emitter {
    fn layout_box(&mut self, name: &str, parent_id: u32, direction: Direction, alignment: u32,
        parent_direction: Direction, sizes: [Size; 2], bounds: [Size; 4], stretch: bool, auto_margins: [bool; 4]) -> Result<u32, Diagnostic> {
        let object_id = self.records.len() as u32 - 1;
            let mut layout = Record::new("LayoutComponent");
            layout.set("name", Value::String(name.into()))?;layout.set("parentId", Value::Uint(parent_id))?;
            layout.set("styleId", Value::Uint(object_id + 1))?;
            let mut layout_style = Record::new("LayoutComponentStyle");
            layout_style.set("flexDirectionValue", Value::Uint(direction.wire()))?;
            layout_style.set("layoutAlignmentType", Value::Uint(alignment))?;
            for (axis, size) in [("width", sizes[0]), ("height", sizes[1])] {
                let (value, units, scale) = match size {
                    Size::Pixels(v) => (v, 1, 0), Size::Percent(v) => (v, 2, 0),
                    // Baseline LayoutParticipant maps Fill on the cross axis to
                    // auto + align-self:stretch. Hug on the main axis gives
                    // auto + flex:0 0 auto. Using 100% here incorrectly resolves
                    // indefinite containing blocks and bypasses intrinsic sizing.
                    Size::Auto if stretch && (axis == "width") != parent_direction.is_row() => (0., 3, 1),
                    Size::Auto => (0., 3, 2),
                };
                layout.set(axis, Value::Float(value))?;
                layout_style.set(&format!("{axis}UnitsValue"), Value::Uint(units))?;
                layout_style.set(if axis == "width" { "layoutWidthScaleType" } else { "layoutHeightScaleType" }, Value::Uint(scale))?;
            }
            for (name, bound) in [("minWidth", bounds[0]), ("minHeight", bounds[1]), ("maxWidth", bounds[2]), ("maxHeight", bounds[3])] {
                let (value, units) = match bound {
                    // Omitted zero minima and absent maxima preserve the existing
                    // zero-content-minimum profile's byte output.
                    Size::Pixels(0.) if name.starts_with("min") => continue,
                    Size::Pixels(v) => (v, 1), Size::Percent(v) => (v, 2),
                    Size::Auto if name.starts_with("min") => (0., 3),
                    Size::Auto => continue,
                };
                layout_style.set(name, Value::Float(value))?;
                layout_style.set(&format!("{name}UnitsValue"), Value::Uint(units))?;
            }
            for (name, automatic) in ["marginLeft", "marginTop", "marginRight", "marginBottom"].into_iter().zip(auto_margins) {
                if automatic {
                    layout_style.set(name, Value::Float(0.))?;
                    layout_style.set(&format!("{name}UnitsValue"), Value::Uint(3))?;
                }
            }
            self.records.push(layout);self.records.push(layout_style);
        Ok(object_id)
    }
    fn children(&mut self, parent: ElementRef<'_>, parent_id: u32, parent_style: &Style, rules: &[css::Rule], path: &str, depth: usize) -> Result<Vec<baseline::Child>, Diagnostic> {
        if depth > 128 { return Err(Diagnostic::new("depth-limit", path, "HTML nesting exceeds 128")); }
        let mut elements = Vec::new();
        for node in parent.children() {
            if node.value().as_text().is_some_and(|t| !t.trim().is_empty()) {
                return Err(unsupported(path, "Text rendering is not yet requalified on the immutable runtime"));
            }
            let Some(element) = ElementRef::wrap(node) else { continue; };
            elements.push(element);
        }
        if self.ids.len().saturating_add(elements.len()) > 8192 {
            return Err(Diagnostic::new("object-limit", path, "Document exceeds 8192 authored elements"));
        }
        let mut ordered_elements = Vec::with_capacity(elements.len());
        for (index, element) in elements.into_iter().enumerate() {
            let path = format!("{path}/{index}");
            if !["div", "section", "article", "main", "header", "footer", "aside", "nav"].contains(&element.value().name()) {
                return Err(unsupported(&path, format!("Element {} is not admitted", element.value().name())));
            }
            for (name, _) in element.value().attrs() {
                if !["id", "class", "style"].contains(&name) { return Err(unsupported(&path, format!("Attribute {name} is not admitted"))); }
            }
            if self.map.len() >= 8192 { return Err(Diagnostic::new("object-limit", &path, "Document exceeds 8192 authored elements")); }
            let id = element.attr("id").map(str::to_owned).unwrap_or_else(|| format!("node{path}"));
            if id.is_empty() || !self.ids.insert(id.clone()) { return Err(Diagnostic::new("duplicate-id", &path, "Empty or duplicate element identity")); }
            let order = ordering_key(element, rules, parent_style)?;
            ordered_elements.push((index, element, path, id, order));
        }
        // CSS order modifies layout/paint order, never DOM selector positions.
        ordered_elements.sort_by_key(|(index, _, _, _, order)| (*order, *index));
        if parent_style.direction.reverses_emission() { ordered_elements.reverse(); }
        let count = ordered_elements.len();
        let mut children = Vec::with_capacity(count);
        if count != 0 { spacing::emit(self, parent_id, parent_style, true)?; }
        for (index, element, path, id, order) in ordered_elements {
            let style = computed(element, rules, parent_style)?;
            if [style.height, style.min_height, style.max_height].iter().any(|size| matches!(size, Size::Percent(_))) && matches!(parent_style.height, Size::Auto) && parent_id != 0 {
                return Err(unsupported(&path, "Percentage height or height bound inside an auto-height parent needs an immutable-target encoding proof"));
            }
            let cross_auto_margin = style.margins.cross(parent_style.direction);
            let effective_alignment = if cross_auto_margin {
                let cross_start = if parent_style.direction.is_row() { 1 } else { 0 };
                let start = style.margins.0[cross_start];
                let end = style.margins.0[cross_start + 2];
                // Native cross auto margins distribute negative free space.
                // A perpendicular safe-alignment wrapper uses main-axis auto
                // margins, which collapse on overflow as CSS requires.
                SelfAlignment { position: if start && end { AlignmentPosition::Center }
                    else if start { AlignmentPosition::FlexEnd } else { AlignmentPosition::FlexStart },
                    overflow: OverflowAlignment::Safe }
            } else { style.self_alignment };
            if style.margins.main(parent_style.direction) && parent_style.spacing.distributes() {
                return Err(unsupported(&path, "Main-axis automatic margins with space-around/evenly require line-aware distribution; flexible spacer helpers would consume margin free space"));
            }
            if effective_alignment.is_baseline() && style.margins.any() {
                return Err(unsupported(&path, "Baseline sharing with main-axis automatic margins requires separate native qualification"));
            }
            let mut authored_margins = style.margins.0;
            if cross_auto_margin {
                let cross = if parent_style.direction.is_row() { 1 } else { 0 };
                authored_margins[cross] = false;
                authored_margins[cross + 2] = false;
            }
            let main_axis = if parent_style.direction.is_row() { 0 } else { 1 };
            // Native main-axis auto margins leave their distributed space in
            // justification's free-space calculation. Ordinary flexible zero-
            // cross-size participants consume it first and collapse on overflow.
            // Native flow is reversed, so physical-end precedes the authored box
            // in file order and physical-start follows it.
            let margin_before = authored_margins[main_axis + 2];
            let margin_after = authored_margins[main_axis];
            authored_margins[main_axis] = false;
            authored_margins[main_axis + 2] = false;
            if margin_before { spacing::emit_weight(self, parent_id, parent_style.direction, 1.)?; }
            let mut sizes = [style.width, style.height];
            let mut bounds = [style.min_width, style.min_height, style.max_width, style.max_height];
            let mut authored_parent = parent_id;
            let mut native_parent_direction = parent_style.direction;
            let mut stretch = effective_alignment.stretches();
            if let Some(alignment) = effective_alignment.wrapper_alignment(parent_style.direction) {
                let main = if parent_style.direction.is_row() { 0 } else { 1 };
                let mut outer_sizes = [Size::Auto; 2];
                outer_sizes[main] = sizes[main];
                let mut outer_bounds = [Size::Pixels(0.), Size::Pixels(0.), Size::Auto, Size::Auto];
                outer_bounds[main] = bounds[main];
                outer_bounds[main + 2] = bounds[main + 2];
                // A perpendicular single-child wrapper aligns on its main
                // axis and stretches the authored box on its cross axis.
                // This transfers constrained automatic main sizes as well as
                // fixed sizes, without percentage substitution or flex growth.
                native_parent_direction = if parent_style.direction.is_row() { Direction::Column } else { Direction::Row };
                authored_parent = self.layout_box("", parent_id, native_parent_direction, alignment,
                    parent_style.direction, outer_sizes, outer_bounds, true, authored_margins)?;
                authored_margins = [false; 4];
                sizes[main] = Size::Auto;
                stretch = true;
                bounds[main] = Size::Pixels(0.);
                bounds[main + 2] = Size::Auto;
            }
            let alignment_margins = effective_alignment.auto_margins(parent_style.direction);
            for (margin, synthetic) in authored_margins.iter_mut().zip(alignment_margins) { *margin |= synthetic; }
            let object_id = self.layout_box(&id, authored_parent, style.direction, style.spacing.alignment(style.direction),
                native_parent_direction, sizes, bounds, stretch, authored_margins)?;
            let background = style.background.used(style.foreground);
            if background >> 24 != 0 {
                let fill_id = self.records.len() as u32 - 1;
                let mut fill = Record::new("Fill");fill.set("parentId", Value::Uint(object_id))?;self.records.push(fill);
                let mut paint = Record::new("SolidColor");paint.set("parentId", Value::Uint(fill_id))?;
                paint.set("colorValue", Value::Color(background))?;self.records.push(paint);
            }
            self.map.push(SourceNode { id, path: path.clone(), object_id });
            let descendants = self.children(element, object_id, &style, rules, &path, depth + 1)?;
            let metric = baseline::summarize(&style, &descendants);
            let used_height = baseline::used_height(&style, &descendants);
            let last_metric = baseline::summarize_last(&style, &descendants, used_height);
            let last = effective_alignment.is_last_baseline();
            if last && last_metric.is_none() {
                return Err(unsupported(&path, "Last baseline requires bounded fixed or intrinsic heights and a baseline within the used box; responsive metrics, distributed columns and non-column nested topology require further ordinary-file validation"));
            }
            if effective_alignment.is_baseline() && !last && metric.is_none() {
                return Err(unsupported(&path, "First baseline requires a bounded fixed or intrinsic box metric, or an empty unbounded percentage-height expression; nested columns need packed, undistributed children with fixed descendant metrics and a baseline within their used height"));
            }
            children.push(baseline::Child { object_id, index, order, metric, last_metric, used_height, last, vertical_auto_margin: style.margins.vertical(), participates: effective_alignment.is_baseline() });
            if margin_after { spacing::emit_weight(self, parent_id, parent_style.direction, 1.)?; }
            spacing::emit(self, parent_id, parent_style, children.len() == count)?;
        }
        baseline::emit(self, parent_id, parent_style, &children, path)?;
        Ok(children)
    }
}
