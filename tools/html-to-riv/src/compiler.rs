//! Ordinary layout/fill lowering. Admission grows only with baseline evidence.
use crate::{css_whitespace, color, css, variables, wire::{self, Record, Value}, CompileInput, CompileOutput, Diagnostic, SourceNode};
use scraper::{ElementRef, Html};
use std::collections::BTreeSet;
#[path = "value_grammar.rs"]
mod value_grammar;
#[path = "baseline.rs"]
mod baseline;
#[path = "spacing.rs"]
mod spacing;
#[path = "margins.rs"]
mod margins;
#[path = "numeric.rs"]
mod numeric;
#[allow(dead_code)] // Numeric proof carrier; typed field integration follows.
#[path = "scalar_provenance.rs"]
mod scalar_provenance;
#[allow(dead_code)] // Private analyzer; descriptor premises must be established before admission.
#[path = "flex_numeric.rs"]
pub(super) mod flex_numeric;
#[path = "computed_provenance.rs"]
mod computed_provenance;
#[path = "gap.rs"]
mod gap;
#[path = "padding.rs"]
mod padding;
#[path = "box_sizing.rs"]
mod box_sizing;
#[allow(dead_code)]
#[path = "flex.rs"]
mod flex;
#[allow(dead_code)] // Conditional parent-size propagation, world proof remains separate.
#[path = "flex_sizes.rs"]
pub(super) mod flex_sizes;
#[allow(dead_code)] // Conditional world propagation; native admission validation follows.
#[path = "flex_world.rs"]
pub(super) mod flex_world;
#[allow(dead_code)] // Actual-group analyzer bridge; public admission remains separate.
#[path = "flex_proof.rs"]
pub(super) mod flex_proof;
#[path = "flex_structure.rs"]
mod flex_structure;
#[path = "flex_scene.rs"]
mod flex_scene;
#[path = "flex_descriptor.rs"]
pub(super) mod flex_descriptor;
// Staged lowering, exercised against frozen ordinary-file experiments before
// public wrapping admission and its contextual guards are installed.
#[allow(dead_code)]
#[path = "wrapping.rs"]
mod wrapping;
#[allow(dead_code)]
#[path = "wrapping_paint.rs"]
mod wrapping_paint;
#[allow(dead_code)] // Machine size bounds; full wrapping admission remains separate.
#[path = "wrapping_sizes.rs"]
mod wrapping_sizes;
#[allow(dead_code)] // Base scene binding; augmented wrapping graph proof follows.
#[path = "wrapping_slots.rs"]
mod wrapping_slots;
#[allow(dead_code)] // Authored domains bound to base slots; full graph proof follows.
#[path = "wrapping_domains.rs"]
mod wrapping_domains;
#[allow(dead_code)] // Conditional scalar normalizer proof; graph premises remain separate.
#[path = "wrapping_normalizer.rs"]
mod wrapping_normalizer;
#[allow(dead_code)] // Closed private composition; no public wrapping admission.
#[path = "wrapping_composition.rs"]
mod wrapping_composition;
#[path = "images.rs"]
mod images;
#[path = "image_constraints.rs"]
mod image_constraints;
#[path = "image_responsive_constraints.rs"]
mod image_responsive_constraints;
#[path = "image_preferred_constraints.rs"]
mod image_preferred_constraints;

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
    match css_whitespace::trim(text).to_ascii_lowercase().as_str() {
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
    let text = css_whitespace::trim(text).to_ascii_lowercase();
    let tokens: Vec<_> = css_whitespace::words(&text).collect();
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
struct Style { image_paint: images::Paint, box_sizing: box_sizing::BoxSizing, gap: gap::Gap, numeric: Box<computed_provenance::NumericStyle>, padding: padding::Padding, flex: flex::Flex, margins: margins::Margins, spacing: spacing::Spacing, self_alignment: SelfAlignment, order: i32, direction: Direction, variables: variables::Variables, width: Size, height: Size, min_width: Size, min_height: Size, max_width: Size, max_height: Size, font_size: f32, foreground: u32, background: BackgroundColor }
impl Default for Style {
    fn default() -> Self { Self { image_paint: images::Paint::default(), box_sizing: box_sizing::BoxSizing::default(), gap: gap::Gap::default(), numeric: Box::default(), padding: padding::Padding::default(), flex: flex::Flex::default(), margins: margins::Margins::default(), spacing: spacing::Spacing::Normal, self_alignment: SelfAlignment::AUTO, order: 0, direction: Direction::Column, variables: variables::Variables::default(), width: Size::Auto, height: Size::Auto, min_width: Size::Pixels(0.), min_height: Size::Pixels(0.), max_width: Size::Auto, max_height: Size::Auto, font_size: ROOT_FONT_SIZE, foreground: 0xff000000, background: BackgroundColor::Rgba(0) } }
}

fn unsupported(source: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("unsupported-target-semantics", source, message)
}

fn size(text: &str, source: &str) -> Result<SpecifiedSize, Diagnostic> {
    let text = css_whitespace::trim(text).to_ascii_lowercase();
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
    match css_whitespace::trim(text).to_ascii_lowercase().as_str() {
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
    match css_whitespace::trim(text).to_ascii_lowercase().as_str() {
        "inherit" => Ok(inherited),
        "none" | "initial" | "unset" if !minimum => Ok(Size::Auto),
        "auto" | "initial" | "unset" if minimum => Ok(Size::Auto),
        "auto" | "none" => Err(unsupported(source, "min-size:none and max-size:auto are invalid")),
        _ => computed_size(text, inherited, font_size, source),
    }
}

fn computed_font_size(text: &str, parent: f32, source: &str) -> Result<f32, Diagnostic> {
    match css_whitespace::trim(text).to_ascii_lowercase().as_str() {
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
    match css_whitespace::trim(&value).to_ascii_lowercase().as_str() {
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

fn validate(d: &css::Declaration, candidate: bool, padding_candidate: bool, inherited_flex: flex::Flex, inherited_padding: padding::Padding) -> Result<(), Diagnostic> {
    validate_target(d, candidate, padding_candidate, inherited_flex, inherited_padding)?;
    // Target parsers use binary32 fields; their rounding cannot make an invalid
    // authored number (for example a tiny negative padding) into a valid zero.
    if !d.name.starts_with("--") && !variables::contains_var(&d.value)
        && value_grammar::classify(&d.name, &d.value) == value_grammar::Validity::Invalid {
        return Err(unsupported(&d.source, format!("{} has an invalid CSS receiving value", d.name)));
    }
    Ok(())
}
fn validate_target(d: &css::Declaration, candidate: bool, padding_candidate: bool, inherited_flex: flex::Flex, inherited_padding: padding::Padding) -> Result<(), Diagnostic> {
    if d.name.starts_with("--") {
        return variables::validate_value(&d.value, &d.source);
    }
    if variables::contains_var(&d.value) {
        if !["width", "height", "min-width", "min-height", "max-width", "max-height",
             "box-sizing", "font-size", "background", "background-color", "color", "display", "flex-direction", "flex", "flex-grow", "flex-shrink", "flex-basis", "order", "align-self", "justify-content", "margin", "margin-left", "margin-top", "margin-right", "margin-bottom", "padding", "padding-left", "padding-top", "padding-right", "padding-bottom", "gap", "row-gap", "column-gap", "object-fit", "object-position", "image-rendering"].contains(&d.name.as_str()) {
            return Err(unsupported(&d.source, format!("{} has no admitted ordinary-Rive lowering yet", d.name)));
        }
        return variables::validate_value(&d.value, &d.source);
    }
    if d.name == "order" { computed_order(&d.value, 0, &d.source)?; return Ok(()); }
    if d.name == "background" {
        background_shorthand(&d.value, BackgroundColor::Rgba(0), &d.source)?;
        return Ok(());
    }
    let value = css_whitespace::trim(&css::ordinary_value(&d.value)?).to_ascii_lowercase();
    match d.name.as_str() {
        "object-fit" | "object-position" | "image-rendering" => { images::Paint::default().apply(&d.name, &value, images::Paint::default(), &d.source)?; }
        "box-sizing" => { box_sizing::computed(&value, box_sizing::BoxSizing::default(), &d.source)?; }
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
        "display" if matches!(value.as_str(), "flex" | "block" | "inherit") => {}
        "margin" | "margin-left" | "margin-top" | "margin-right" | "margin-bottom" => { margins::Margins::default().apply(&d.name, &value, margins::Margins::default(), &d.source)?; }
        "padding" | "padding-left" | "padding-top" | "padding-right" | "padding-bottom" => {
            let mut descriptor = padding::Padding::default();
            descriptor.apply(&d.name, &value, inherited_padding, if padding_candidate { 0. } else { ROOT_FONT_SIZE }, &d.source)?;
            if !padding_candidate && !descriptor.is_zero() {
                return Err(unsupported(&d.source, "Nonzero padding requires private candidate qualification, including unmatched or overridden declarations"));
            }
        }
        "justify-content" => { spacing::computed(&value, spacing::Spacing::Normal, &d.source)?; }
        "align-self" => { computed_alignment(&value, SelfAlignment::AUTO, &d.source)?; }
        "flex" | "flex-grow" | "flex-shrink" | "flex-basis" => {
            let mut descriptor = flex::Flex::default();
            descriptor.apply(&d.name, &value, inherited_flex, 0., &d.source)?;
            if !candidate && !descriptor.legacy() {
                return Err(unsupported(&d.source, "Nonlegacy flex declarations await aggregate numeric and visual qualification, including unmatched or overridden declarations"));
            }
        }
        "gap" | "row-gap" | "column-gap" => {
            let mut gap=gap::Gap::default();
            gap.apply(&d.name,&value,gap::Gap::default(),0.,&d.source)?;
        }
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
    let value = substitute_ordinary(&declaration.name, &declaration.value, &environment, &declaration.source)?;
    computed_order(&value.native, parent.order, &declaration.source)
}

// Failed substitution and values proven invalid by the receiving grammar
// compute as unset at the same cascade priority. Complete resolution first so
// syntax/resource failures cannot be hidden. Valid and unclassified values
// still undergo target admission, including declarations that lose the cascade.
fn substitute_ordinary(property: &str, value: &str, variables: &variables::Variables, source: &str)
    -> Result<variables::ResolvedValue, Diagnostic> {
    Ok(variables::substitute_with_provenance(value, variables, source)?
        .filter(|value| value_grammar::classify(property, &value.native) != value_grammar::Validity::Invalid)
        .unwrap_or_else(|| variables::ResolvedValue { native: "unset".into(), original: None }))
}

/// A transient declaration pair consumed by typed property computation. The
/// native declaration retains its source/name/importance and existing normalized
/// value. Original selected tokens may be unavailable, never inferred from it.
struct ResolvedDeclaration {
    native: css::Declaration,
    original: Option<std::sync::Arc<String>>,
}
impl std::ops::Deref for ResolvedDeclaration {
    type Target = css::Declaration;
    fn deref(&self) -> &Self::Target { &self.native }
}
fn resolved_declarations(mut declarations: Vec<css::Declaration>, parent: &Style, candidate: bool, padding_candidate: bool)
    -> Result<(Vec<ResolvedDeclaration>, variables::Variables), Diagnostic> {
    for d in &declarations { validate(d, candidate, padding_candidate, parent.flex, parent.padding)?; }
    let variable_values = variables::compute(&declarations, &parent.variables)?;
    declarations.retain(|d| !d.name.starts_with("--"));
    let mut resolved = Vec::with_capacity(declarations.len());
    for mut d in declarations {
        let original = if variables::contains_var(&d.value) {
            let selected = substitute_ordinary(&d.name, &d.value, &variable_values, &d.source)?;
            d.value = selected.native;
            // Substitution cannot bypass property admission, even for a declaration
            // that loses the cascade. Keep the compiler's strict diagnostics.
            validate(&d, candidate, padding_candidate, parent.flex, parent.padding)?;
            selected.original
        } else { Some(std::sync::Arc::new(d.value.clone())) };
        if d.name != "order" { d.value = css::ordinary_value(&d.value)?; }
        resolved.push(ResolvedDeclaration {native:d, original});
    }
    Ok((resolved, variable_values))
}
fn computed(element: ElementRef<'_>, rules: &[css::Rule], parent: &Style, candidate: bool, padding_candidate: bool) -> Result<Style, Diagnostic> {
    let (declarations, variable_values) = resolved_declarations(css::cascade(rules, element)?, parent, candidate, padding_candidate)?;
    if element.value().name() != "img" && declarations.iter().any(|d| d.name == "display" && d.value.eq_ignore_ascii_case("block")) {
        return Err(unsupported(element.attr("id").unwrap_or(element.value().name()), "display:block is currently admitted only for replaced img elements; block container layout needs separate qualification"));
    }
    let mut style = Style { image_paint: images::Paint::inherited(parent.image_paint), numeric: Box::new(computed_provenance::NumericStyle {font:parent.numeric.font.clone(),..computed_provenance::NumericStyle::default()}), variables: variable_values, foreground: parent.foreground, font_size: parent.font_size, ..Style::default() };
    // Font-size-relative units use the parent for font-size itself, but the final
    // computed element font size for other lengths, regardless of source order.
    for d in declarations.iter().filter(|d| d.name == "font-size") {
        style.font_size = computed_font_size(&d.value, parent.font_size, &d.source)?;
        style.numeric.font = computed_provenance::font(d,style.font_size,&parent.numeric.font);
    }
    // Resolve currentColor against the final computed color, irrespective of
    // declaration order. A background currentColor stays a computed keyword
    // through inheritance and resolves against the receiving element at emission.
    for d in declarations.iter().filter(|d| d.name == "color") {
        let value = css_whitespace::trim(&d.value).to_ascii_lowercase();
        style.foreground = match value.as_str() {
            "inherit" | "unset" | "currentcolor" => parent.foreground,
            "initial" => 0xff000000,
            _ => color::parse(&value, &d.source)?,
        };
    }
    for d in &declarations {
        match d.name.as_str() {
            "margin" | "margin-left" | "margin-top" | "margin-right" | "margin-bottom" => style.margins.apply(&d.name, &d.value, parent.margins, &d.source)?,
            "padding" | "padding-left" | "padding-top" | "padding-right" | "padding-bottom" => { style.padding.apply(&d.name, &d.value, parent.padding, style.font_size, &d.source)?; style.numeric.padding = computed_provenance::padding(d, style.padding, &style.numeric.padding, &parent.numeric.padding, &style.numeric.font); },
            "box-sizing" => style.box_sizing = box_sizing::computed(&d.value, parent.box_sizing, &d.source)?,
            "justify-content" => style.spacing = spacing::computed(&d.value, parent.spacing, &d.source)?,
            "align-self" => style.self_alignment = computed_alignment(&d.value, parent.self_alignment, &d.source)?,
            "order" => style.order = computed_order(&d.value, parent.order, &d.source)?,
            "flex" | "flex-grow" | "flex-shrink" | "flex-basis" => { style.flex.apply(&d.name, &d.value, parent.flex, style.font_size, &d.source)?; style.numeric.flex.apply(d,style.flex,&parent.numeric.flex,&style.numeric.font); },
            "gap" | "row-gap" | "column-gap" => style.gap.apply(&d.name,&d.value,parent.gap,style.font_size,&d.source)?,
            "flex-direction" => style.direction = computed_direction(&d.value, parent.direction, &d.source)?,
            "width" => { style.width = computed_size(&d.value, parent.width, style.font_size, &d.source)?; style.numeric.width = computed_provenance::dimension(d,style.width,&parent.numeric.width,&style.numeric.font); },
            "height" => { style.height = computed_size(&d.value, parent.height, style.font_size, &d.source)?; style.numeric.height = computed_provenance::dimension(d,style.height,&parent.numeric.height,&style.numeric.font); },
            "min-width" => { style.min_width = computed_bound(&d.value, parent.min_width, style.font_size, true, &d.source)?; style.numeric.min_width = computed_provenance::dimension(d,style.min_width,&parent.numeric.min_width,&style.numeric.font); },
            "min-height" => { style.min_height = computed_bound(&d.value, parent.min_height, style.font_size, true, &d.source)?; style.numeric.min_height = computed_provenance::dimension(d,style.min_height,&parent.numeric.min_height,&style.numeric.font); },
            "max-width" => { style.max_width = computed_bound(&d.value, parent.max_width, style.font_size, false, &d.source)?; style.numeric.max_width = computed_provenance::dimension(d,style.max_width,&parent.numeric.max_width,&style.numeric.font); },
            "max-height" => { style.max_height = computed_bound(&d.value, parent.max_height, style.font_size, false, &d.source)?; style.numeric.max_height = computed_provenance::dimension(d,style.max_height,&parent.numeric.max_height,&style.numeric.font); },
            "object-fit" | "object-position" | "image-rendering" => style.image_paint.apply(&d.name, &d.value, parent.image_paint, &d.source)?,
            "background" => apply_background_shorthand(&mut style, parent, &d.value, &d.source)?,
            "background-color" => {
                style.background = match css_whitespace::trim(&d.value).to_ascii_lowercase().as_str() {
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
    compile_profile(input, FlexPolicy::Guarded)
}

// Candidate policy is private: an experimental source harness can exercise
// the complete pipeline, while the exported API always uses Guarded.
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub(super) enum FlexPolicy { Guarded, Candidate, PaddingCandidate }
pub(super) fn compile_profile(input: &CompileInput, policy: FlexPolicy) -> Result<CompileOutput, Diagnostic> {
    compile_with_descriptor_capture(input, policy, false).map(|(output, _)| output)
}
#[allow(dead_code)]
pub(super) fn compile_profile_with_descriptors(input: &CompileInput, policy: FlexPolicy) -> Result<(CompileOutput, Vec<flex_descriptor::Group>), Diagnostic> {
    compile_with_descriptor_capture(input, policy, true)
}
fn compile_with_descriptor_capture(input: &CompileInput, policy: FlexPolicy, capture: bool) -> Result<(CompileOutput, Vec<flex_descriptor::Group>), Diagnostic> {
    if ![input.width, input.height].into_iter().all(|v| v.is_finite() && v > 0. && v <= 16384.) {
        return Err(Diagnostic::new("invalid-viewport", "viewport", "Dimensions must be finite and in (0,16384]"));
    }
    if input.html.len().saturating_add(input.css.len()) > 1_048_576 {
        return Err(Diagnostic::new("input-limit", "document", "HTML and CSS exceed 1 MiB"));
    }
    let assets = crate::assets::AssetTable::validate(&input.assets)?;
    let fragment = Html::parse_fragment(&input.html);
    if !fragment.errors.is_empty() { return Err(Diagnostic::new("html-syntax", "html", "Malformed HTML fragment")); }
    let document = Html::parse_document(&format!("<!doctype html><html><head></head><body>{}</body></html>", input.html));
    let body = document.root_element().child_elements().find(|e| e.value().name() == "body")
        .ok_or_else(|| Diagnostic::new("html-syntax", "html", "Missing document body"))?;
    let rules = css::stylesheet(&input.css)?;
    css::validate_rules(&rules, |declaration| validate(declaration, matches!(policy, FlexPolicy::Candidate), matches!(policy, FlexPolicy::Guarded | FlexPolicy::PaddingCandidate), flex::Flex::default(), padding::Padding::default()))?;
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
    let mut output = Emitter { assets, records, map: Vec::new(), ids: BTreeSet::new(), candidate_flex: matches!(policy, FlexPolicy::Candidate), candidate_padding: matches!(policy, FlexPolicy::Guarded | FlexPolicy::PaddingCandidate), descriptor_capture: capture, descriptors: Vec::new() };
    // Match the fixed host body in reset.css for inherited computed values.
    let host_style = Style { numeric: Box::new(computed_provenance::NumericStyle::host()), width: Size::Percent(100.), height: Size::Percent(100.),
        background: BackgroundColor::Rgba(0xffffffff), ..Style::default() };
    output.children(body, 0, &host_style, &rules, "", 0, [true; 2], numeric::Bounds::VIEWPORT, None)?;
    // File object order serves native drawing; public identities stay in DOM
    // preorder, with numeric path components (so /2 precedes /10).
    output.map.sort_by_cached_key(|node| node.path.split('/').skip(1)
        .map(|part| part.parse::<usize>().expect("generated numeric DOM path")).collect::<Vec<_>>());
    if output.map.is_empty() { return Err(Diagnostic::new("empty-document", "html", "At least one box element is required")); }
    let scene_index = if capture { flex_descriptor::scene_index(&output.records) } else { Default::default() };
    let descriptors = output.descriptors.into_iter().map(|pending| pending.finish(&output.records, &scene_index)).collect();
    let assets = output.assets.global_records()?;
    output.records.splice(1..1, assets);
    Ok((CompileOutput { riv: wire::encode(&output.records)?, source_map: output.map }, descriptors))
}

struct Emitter { assets: crate::assets::AssetTable, records: Vec<Record>, map: Vec<SourceNode>, ids: BTreeSet<String>, candidate_flex: bool, candidate_padding: bool, descriptor_capture: bool, descriptors: Vec<flex_descriptor::Pending> }
type AuthoredElement<'a> = (usize, ElementRef<'a>, String, String, i32);
struct ChildrenFrame<'a> {
    elements: std::vec::IntoIter<AuthoredElement<'a>>,
    descriptor_start: usize,
    descriptor_items: Vec<flex_descriptor::Item>,
    children: Vec<baseline::Child>,
    count: usize,
}
struct PreparedChild<'a> {
    element: ElementRef<'a>, object_id: u32, content_id: u32, content_owner: Option<box_sizing::ContentOwner>, path: String, style: Style,
    index: usize, order: i32, effective_alignment: SelfAlignment,
    margin_after: bool, vertical_flex: bool,
    child_chain: [bool; 2], child_bounds: numeric::Bounds,
}
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
    fn children(&mut self, parent: ElementRef<'_>, parent_id: u32, parent_style: &Style, rules: &[css::Rule], path: &str, depth: usize, definite_chain: [bool; 2], numeric_bounds: numeric::Bounds, content_owner: Option<&box_sizing::ContentOwner>) -> Result<Vec<baseline::Child>, Diagnostic> {
        // Recursion retains only heap-backed frame/child state. Large style,
        // record and diagnostic temporaries live in nonrecursive helpers, so
        // the public depth limit also fits the default WASM stack.
        let mut frame = self.prepare_children(parent, parent_id, parent_style, rules, path, depth, definite_chain, numeric_bounds)?;
        while let Some(item) = frame.elements.next() {
            let child = self.start_child(item, parent_id, parent_style, rules, definite_chain, numeric_bounds, &mut frame.descriptor_items)?;
            let descendants = self.children(child.element, child.content_id, &child.style, rules, &child.path, depth + 1, child.child_chain, child.child_bounds, child.content_owner.as_ref())?;
            let result = self.finish_child(&child, &descendants, parent_id, parent_style, frame.children.len() + 1 == frame.count)?;
            frame.children.push(result);
        }
        baseline::emit(self, parent_id, parent_style, &frame.children, path)?;
        if frame.count != 0 { self.capture_parent(parent_style, parent_id, path, frame.descriptor_start, frame.descriptor_items, content_owner)?; }
        Ok(frame.children)
    }
    fn prepare_children<'a>(&mut self, parent: ElementRef<'a>, parent_id: u32, parent_style: &Style,
        rules: &[css::Rule], path: &str, depth: usize, definite_chain: [bool; 2], numeric_bounds: numeric::Bounds)
        -> Result<Box<ChildrenFrame<'a>>, Diagnostic> {
        if depth > 128 { return Err(Diagnostic::new("depth-limit", path, "HTML nesting exceeds 128")); }
        let mut elements = Vec::new();
        for node in parent.children() {
            if node.value().as_text().is_some_and(|t| !css_whitespace::trim(t).is_empty()) {
                return Err(unsupported(if path.is_empty() { "html" } else { path }, "Text rendering is not yet requalified on the immutable runtime"));
            }
            let Some(element) = ElementRef::wrap(node) else { continue; };
            elements.push(element);
        }
        if !self.candidate_flex && !parent_style.gap.is_zero() {
            numeric_bounds.gap(parent_style.gap,elements.len(),path)?;
            if !parent_style.padding.is_zero() || parent_style.spacing.distributes() {
                return Err(unsupported(path,"Gaps with padding or distributed spacing require separate layout qualification"));
            }
        }
        if self.ids.len().saturating_add(elements.len()) > 8192 {
            return Err(Diagnostic::new("object-limit", path, "Document exceeds 8192 authored elements"));
        }
        let mut ordered_elements = Vec::with_capacity(elements.len());
        for (index, element) in elements.into_iter().enumerate() {
            let path = format!("{path}/{index}");
            if !["div", "section", "article", "main", "header", "footer", "aside", "nav", "img"].contains(&element.value().name()) {
                return Err(unsupported(&path, format!("Element {} is not admitted", element.value().name())));
            }
            for (name, _) in element.value().attrs() {
                // HTML parsing has normalized ASCII attribute names. Custom data
                // remains on the source DOM for selectors; it adds no file state.
                let static_data = name.strip_prefix("data-").is_some_and(|suffix| !suffix.is_empty());
                let image_attribute = element.value().name() == "img" && ["src", "alt"].contains(&name);
                if !["id", "class", "style"].contains(&name) && !static_data && !image_attribute { return Err(unsupported(&path, format!("Attribute {name} is not admitted"))); }
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
        // Inspect compact sibling facts before introducing any helper. Keeping
        // whole Styles here would retain each inherited variable environment.
        let mut group = flex::Group::default();
        let mut padded_group = !parent_style.padding.is_zero();
        let mut baseline_group = false;
        for (_, element, path, _, _) in &ordered_elements {
            let style = computed(*element, rules, parent_style, self.candidate_flex, self.candidate_padding)?;
            if !self.candidate_flex && !parent_style.gap.is_zero() && (style.margins.any() || style.self_alignment.is_baseline() || style.self_alignment.is_center() || style.self_alignment.is_end() || !style.padding.is_zero()) {
                return Err(unsupported(path,"Gapped groups with automatic margins, alignment helpers or padded children require separate qualification"));
            }
            group.inspect(&style, parent_style, definite_chain, path)?;
            padded_group |= !style.padding.is_zero();
            baseline_group |= style.self_alignment.is_baseline() && !style.margins.cross(parent_style.direction);
        }
        group.validate(parent_style, path, self.candidate_flex)?;
        if padded_group && baseline_group {
            return Err(unsupported(path, "Padding with baseline groups needs padding-aware metric and origin qualification"));
        }
        let descriptor_start = self.records.len();
        let descriptor_items = Vec::new();
        let count = ordered_elements.len();
        let children = Vec::with_capacity(count);
        if count != 0 { spacing::emit(self, parent_id, parent_style, true)?; }
        Ok(Box::new(ChildrenFrame { elements: ordered_elements.into_iter(), descriptor_start, descriptor_items, children, count }))
    }
    fn start_child<'a>(&mut self, item: AuthoredElement<'a>, parent_id: u32, parent_style: &Style,
        rules: &[css::Rule], definite_chain: [bool; 2], numeric_bounds: numeric::Bounds,
        descriptor_items: &mut Vec<flex_descriptor::Item>) -> Result<Box<PreparedChild<'a>>, Diagnostic> {
        let (index, element, path, id, order) = item;
        let mut style = computed(element, rules, parent_style, self.candidate_flex, self.candidate_padding)?;
        let image = images::plan(element, &mut style, parent_style, &self.assets, &path)?;
        if [style.height, style.min_height, style.max_height].iter().any(|size| matches!(size, Size::Percent(_))) && matches!(parent_style.height, Size::Auto) && parent_id != 0 {
            return Err(unsupported(&path, "Percentage height or height bound inside an auto-height parent needs an immutable-target encoding proof"));
        }
        let lowered = match &image {
            Some(image) => image.lower_outer(&style, &path)?,
            None => box_sizing::lower(&style, &path)?,
        };
        let outer_stretch = image.as_ref().map_or_else(
            || style.self_alignment.stretches() && !style.margins.cross(parent_style.direction),
            |image| image.outer_stretch(&style, parent_style, &lowered));
        let child_bounds = numeric_bounds.child_with_stretch(&style, &lowered, parent_style, outer_stretch, &path)?;
        let flex_plan = flex::lowering(&style, parent_style, definite_chain, &path)?;
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
        if !self.candidate_flex && !style.gap.is_zero() && effective_alignment.wrapper_alignment(parent_style.direction).is_some() {
            return Err(unsupported(&path,"Gap containers inside alignment wrappers require separate qualification"));
        }
        if !style.padding.is_zero() && effective_alignment.wrapper_alignment(parent_style.direction).is_some() {
            return Err(unsupported(&path, "Padding on an alignment wrapper participant needs a separate padding-floor and percentage-containing-block proof"));
        }
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
        let mut sizes = lowered.sizes;
        let mut bounds = lowered.bounds;
        let mut authored_parent = parent_id;
        let mut native_parent_direction = parent_style.direction;
        let mut stretch = outer_stretch;
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
        let content_owner = image.as_ref().and_then(images::Plan::content_owner)
            .or_else(|| box_sizing::content_owner(&style, native_parent_direction));
        // Evaluate both actual owners. Rounded outer content bounds alone can
        // erase a tiny fixed inner size and miss overflow in its descendants.
        let child_bounds = if let Some(owner) = &content_owner { child_bounds.content_owner(owner, &path)? } else { child_bounds };
        let child_bounds = if let Some(image) = &image { image.validate_bounds(child_bounds, numeric_bounds, &path)? } else { child_bounds };
        if image.is_some() && content_owner.is_some() {
            numeric_bounds.image_outer(child_bounds, sizes, style.padding, native_parent_direction, stretch, &path)?;
        }
        let outer_direction = content_owner.map_or(style.direction, |owner| owner.packing);
        let outer_alignment = content_owner.map_or_else(|| style.spacing.alignment(style.direction), |owner| owner.packing.alignment());
        let object_id = self.layout_box(&id, authored_parent, outer_direction, outer_alignment,
            native_parent_direction, sizes, bounds, stretch, authored_margins)?;
        style.padding.emit(&mut self.records[object_id as usize + 2])?;
        if content_owner.is_none() { style.gap.emit(&mut self.records[object_id as usize + 2])?; }
        if let Some(plan) = flex_plan {
            // Cross-alignment wrappers are the actual flex participants.
            // Keep the authored box identity and its inner stretch sizing.
            plan.emit(&mut self.records, if authored_parent == parent_id { object_id } else { authored_parent }, parent_style.direction)?;
        }
        let background = style.background.used(style.foreground);
        if background >> 24 != 0 {
            let fill_id = self.records.len() as u32 - 1;
            let mut fill = Record::new("Fill");fill.set("parentId", Value::Uint(object_id))?;self.records.push(fill);
            let mut paint = Record::new("SolidColor");paint.set("parentId", Value::Uint(fill_id))?;
            paint.set("colorValue", Value::Color(background))?;self.records.push(paint);
        }
        let content_id = if let Some(owner) = &content_owner {
            let inner = self.layout_box("", object_id, style.direction, style.spacing.alignment(style.direction),
                owner.packing, owner.sizes, owner.bounds, true, [false; 4])?;
            style.gap.emit(&mut self.records[inner as usize + 2])?;
            inner
        } else { object_id };
        if let Some(image) = &image { image.emit(&mut self.records, content_id, style.image_paint)?; }
        self.capture_item(descriptor_items, &style, &lowered, parent_style.direction, &id, &path,
            object_id, if authored_parent == parent_id { object_id } else { authored_parent }, index, order);
        self.map.push(SourceNode { id, path: path.clone(), object_id });
        let child_chain = flex::child_chain(definite_chain, &style);
        Ok(Box::new(PreparedChild { element, object_id, content_id, content_owner, path, style, index, order, effective_alignment,
            margin_after, vertical_flex: flex_plan.is_some() && !parent_style.direction.is_row(), child_chain, child_bounds }))
    }
    fn finish_child(&mut self, child: &PreparedChild<'_>, descendants: &[baseline::Child], parent_id: u32,
        parent_style: &Style, last_child: bool) -> Result<baseline::Child, Diagnostic> {
        let style = &child.style; let object_id = child.object_id; let index = child.index; let order = child.order;
        let effective_alignment = child.effective_alignment; let path = &child.path; let margin_after = child.margin_after;
        // A flexible vertical main size is not the authored intrinsic height.
        // Unknown scalar summaries prevent unsound ancestor baseline admission.
        let vertical_flex = child.vertical_flex;
        // An image is replaced content, not an empty box. Until replaced
        // baselines are qualified, propagate unknown metrics to ancestor groups.
        let unknown_metric = vertical_flex || child.element.value().name() == "img";
        let metric = if unknown_metric { None } else { baseline::summarize(&style, &descendants) };
        let used_height = if unknown_metric { None } else { baseline::used_height(&style, &descendants) };
        let last_metric = if unknown_metric { None } else { baseline::summarize_last(&style, &descendants, used_height) };
        let last = effective_alignment.is_last_baseline();
        if last && last_metric.is_none() {
            return Err(unsupported(&path, "Last baseline requires bounded fixed or intrinsic heights and a baseline within the used box; responsive metrics, distributed columns and non-column nested topology require further ordinary-file validation"));
        }
        if effective_alignment.is_baseline() && !last && metric.is_none() {
            return Err(unsupported(&path, "First baseline requires a bounded fixed or intrinsic box metric, or an empty unbounded percentage-height expression; nested columns need packed, undistributed children with fixed descendant metrics and a baseline within their used height"));
        }
        let result = baseline::Child { object_id, index, order, metric, last_metric, used_height, last, vertical_auto_margin: style.margins.vertical(), participates: effective_alignment.is_baseline() };
        if margin_after { spacing::emit_weight(self, parent_id, parent_style.direction, 1.)?; }
        spacing::emit(self, parent_id, parent_style, last_child)?;
        Ok(result)
    }
    // Keep the large optional diagnostic carriers out of each recursive
    // emission frame; the declared 128-level input limit must fit native/WASM.
    fn capture_item(&self, items: &mut Vec<flex_descriptor::Item>, style: &Style,
        sizing: &box_sizing::Lowered, direction: Direction, id: &str, path: &str,
        object: u32, participant: u32, index: usize, order: i32) {
        if self.descriptor_capture { items.push(flex_descriptor::Item::extract_lowered(style,
            sizing, direction, &self.records, id, path, object, participant, index, order)); }
    }
    fn capture_parent(&mut self, style: &Style, parent_id: u32, path: &str,
        record_start: usize, items: Vec<flex_descriptor::Item>, content_owner: Option<&box_sizing::ContentOwner>) -> Result<(), Diagnostic> {
        if self.descriptor_capture {
            let parent = if let Some(owner) = content_owner { flex_descriptor::Parent::extract_content_owner(style, owner) }
                else { flex_descriptor::Parent::extract_lowered(style, &box_sizing::lower(style, path)?) };
            self.descriptors.push(flex_descriptor::Pending {parent,parent_id,path:path.into(),record_start,record_end:self.records.len(),items});
        }
        Ok(())
    }
}

#[cfg(test)]
mod padding_pipeline_tests {
    use super::*;
    fn input(css: &str) -> CompileInput {
        CompileInput { assets: Default::default(),html:"<div id=p><div id=a><div id=leaf></div></div></div>".into(),css:format!("#p{{width:160px;height:120px}}#a{{width:40px;height:30px}}#leaf{{width:10px;height:10px}}{css}"),width:240.,height:160.}
    }
    fn candidate(css:&str)->CompileOutput {compile_profile(&input(css),FlexPolicy::PaddingCandidate).unwrap()}
    #[test]
    fn padding_cascade_variables_font_resolution_and_inheritance_remain_computed() {
        assert_eq!(candidate("#p{--pad:1em 2em;font-size:10px;padding:var(--pad)}"),candidate("#p{font-size:10px;padding:10px 20px}"));
        assert_eq!(candidate("#p{font-size:10px;padding:2em}#a{font-size:30px;padding:inherit}"),candidate("#p{font-size:10px;padding:20px}#a{font-size:30px;padding:20px}"));
        assert_eq!(candidate("#a{padding-left:4px!important;padding:2px}"),candidate("#a{padding:2px 2px 2px 4px}"));
        assert_eq!(candidate("#a{padding:10px;padding:unset}"),candidate(""));
    }
    #[test]
    fn candidate_padding_keeps_authored_identity_and_element_count() {
        let plain=candidate("");let padded=candidate("#p{padding:2% 3% 4% 5%}#a{padding:1px 2px 3px 4px}");
        assert_eq!(plain.source_map,padded.source_map);
        assert_ne!(plain.riv,padded.riv);
        // A padding floor larger than the authored box is native behavior;
        // neither dimensions nor helpers are substituted at compile time.
        assert!(compile_profile(&input("#a{width:1px;height:1px;padding:30px}"),FlexPolicy::PaddingCandidate).is_ok());
    }
    #[test]
    fn unresolved_padding_wrapper_baseline_and_flex_contexts_diagnose() {
        for direction in ["row","row-reverse","column","column-reverse"] {
            let css=format!("#p{{flex-direction:{direction}}}#a{{padding:2px;align-self:center}}");
            assert!(compile_profile(&input(&css),FlexPolicy::PaddingCandidate).is_err());
        }
        for css in ["#p{flex-direction:row;padding:2px}#a{align-self:baseline}","#p{flex-direction:row}#a{padding:2px;align-self:baseline}","#a{height:auto;flex:1 1 0px;padding:2px}"] {
            assert!(compile_profile(&input(css),FlexPolicy::PaddingCandidate).is_err(),"{css}");
        }
        assert!(compile_profile(&input("#a{padding:2px}"),FlexPolicy::Candidate).is_err());
    }
    #[test]
    fn ancestor_baseline_cannot_reuse_unpadded_descendant_metrics() {
        let css="#p{flex-direction:row}#a{align-self:baseline}#leaf{padding:2px}";
        assert!(compile_profile(&input(css),FlexPolicy::PaddingCandidate).is_err());
    }
}

#[cfg(test)]
mod declaration_provenance_tests {
    use super::*;
    fn declarations(css:&str,parent:&Style)->Result<Vec<ResolvedDeclaration>,Diagnostic> {
        let document=Html::parse_document("<div id=a></div>");
        let selector=scraper::Selector::parse("#a").unwrap();
        let element=document.select(&selector).next().unwrap();
        let rules=css::stylesheet(css).unwrap();
        resolved_declarations(css::cascade(&rules,element)?,parent,false,true).map(|(values,_)|values)
    }
    #[test]
    fn winning_variable_tokens_remain_paired_with_actual_native_normalization() {
        let values=declarations("#a{--v:999.123px;width:10px}#a{--v:100.71428680419922px;width:var(--v)}",&Style::default()).unwrap();
        let widths:Vec<_>=values.iter().filter(|d|d.name=="width").collect();
        assert_eq!(widths.len(),2);assert!(widths[0].original.as_ref().unwrap().contains("10px"));
        assert_eq!(widths[1].native.value.trim(),"100.71428680419922px");
        assert!(widths[1].original.as_ref().unwrap().contains("100.71428680419922px"));
        assert!(!widths[1].original.as_ref().unwrap().contains("999.123"));
        let direct=declarations("#a{width:100.71428680419922px}",&Style::default()).unwrap();
        assert_eq!(direct[0].native.value.trim(),"100.71428680419922px");assert!(direct[0].original.as_ref().unwrap().contains("100.71428680419922px"));
    }
    #[test]
    fn frozen_inherited_variables_and_fallbacks_supply_selected_originals() {
        let mut parent=Style::default();
        parent.variables=variables::compute(&css::declarations("--x:1.234567890123em;--alias:var(--x)","test").unwrap(),&variables::Variables::default()).unwrap();
        let values=declarations("#a{--x:9rem;width:var(--alias);height:var(--missing,2.34567890123%)}",&parent).unwrap();
        assert!(values.iter().find(|d|d.name=="width").unwrap().original.as_ref().unwrap().contains("1.234567890123em"));
        assert!(values.iter().find(|d|d.name=="height").unwrap().original.as_ref().unwrap().contains("2.34567890123%"));
        parent.variables.insert("--lost".into(),Some(variables::ResolvedValue {native:"12px".into(),original:None}));
        assert!(declarations("#a{width:var(--lost,99px)}",&parent).unwrap()[0].original.is_none());
    }
    #[test]
    fn production_computation_keeps_fonts_first_and_strict_loser_validation() {
        let mut input=CompileInput { assets: Default::default(),html:"<div id=a></div>".into(),css:"#a{--w:2em;width:var(--w);font-size:20px}".into(),width:240.,height:160.};
        let actual=compile(&input).unwrap();input.css="#a{width:40px;font-size:20px}".into();assert_eq!(actual,compile(&input).unwrap());
        for css in ["#a{--bad:calc(1px);width:var(--bad);width:1px}","#a{--f:1;flex:var(--f);flex:none}"] {assert!(declarations(css,&Style::default()).is_err());}
    }
}

#[cfg(test)]
mod gap_pipeline_tests {
    use super::*;
    fn request(css:&str)->CompileInput {CompileInput{ assets: Default::default(),html:"<div id=p><div id=a></div><div id=b></div></div>".into(),css:format!("#p{{width:200px;height:100px}}#a,#b{{width:20px;height:10px}}{css}"),width:400.,height:200.}}
    #[test]
    fn public_gap_uses_computed_inheritance() {
        let css="#p{--g:.5em 1em;font-size:20px;gap:var(--g)}#a{font-size:10px;gap:inherit}";
        let actual=compile_profile(&request(css),FlexPolicy::Candidate).unwrap();
        let expected=compile_profile(&request("#p{gap:10px 20px}#a{gap:10px 20px}"),FlexPolicy::Candidate).unwrap();
        assert_eq!(actual,expected);
        assert_eq!(actual,compile_profile(&request(css),FlexPolicy::Guarded).unwrap());
        assert!(compile_profile(&request("#unmatched{gap:0}"),FlexPolicy::Guarded).is_ok());
    }
    #[test]
    fn public_gap_rejects_unqualified_contexts_and_keeps_intrinsic_points() {
        for css in ["#p{height:auto;row-gap:5%}","#p{gap:4px;justify-content:space-around}","#p{gap:4px;padding:2px}","#p{gap:4px}#a{margin-left:auto}","#p{gap:4px}#a{align-self:center}","#p{gap:4px}#a{padding:1px}","#p{gap:4px;align-self:baseline}"] {
            assert!(compile_profile(&request(css),FlexPolicy::Guarded).is_err(),"{css}");
        }
        assert!(compile_profile(&request("#p{width:auto;height:auto;gap:4px;align-self:flex-start}"),FlexPolicy::Guarded).is_ok());
        assert!(compile_profile(&request("#p{row-gap:5%;column-gap:10%}"),FlexPolicy::Guarded).is_ok());
        for css in ["#unmatched{gap:-1px}","#p{gap:1px 2px 3px;gap:0}"] {
            assert!(compile_profile(&request(css),FlexPolicy::Guarded).is_err(),"{css}");
        }
        assert_eq!(compile_profile(&request("#p{gap:var(--missing,)}"),FlexPolicy::Guarded).unwrap(),
            compile_profile(&request("#p{gap:unset}"),FlexPolicy::Guarded).unwrap());
    }
    #[test]
    fn candidate_zero_gaps_preserve_bytes_and_nonzero_gaps_exclude_flex_proof() {
        let plain=compile_profile(&request(""),FlexPolicy::Candidate).unwrap();
        assert_eq!(plain,compile_profile(&request("#p{gap:normal}"),FlexPolicy::Candidate).unwrap());
        let (_,groups)=compile_profile_with_descriptors(&request("#p{gap:3px}"),FlexPolicy::Candidate).unwrap();
        let parent=groups.iter().find(|g|g.items.len()==2).unwrap();
        assert!(parent.structural_issues.iter().any(|issue|issue.contains("gaps")));
        assert!(!parent.numerical_admission);
    }
}
