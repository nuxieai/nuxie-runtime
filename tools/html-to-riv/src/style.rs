use crate::wire::{Record, Value};
use crate::{Diagnostic, color, css::Declaration, length};

/// Rive's existing flexWrapValue encoding; both wrapping modes need line policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WrapMode { NoWrap, Wrap, WrapReverse }
impl WrapMode {
    pub(crate) fn wraps(self) -> bool { self != Self::NoWrap }
    fn rive_value(self) -> u32 {
        match self { Self::NoWrap => 0, Self::Wrap => 1, Self::WrapReverse => 2 }
    }
}

/// Physical margin values retain auto until runtime layout distributes space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Margin {
    Auto,
    Px(f32),
    Percent(f32),
}
impl Margin {
    fn parse(value: &str, source: &str) -> Result<Self, Diagnostic> {
        if value.eq_ignore_ascii_case("auto") {
            Ok(Self::Auto)
        } else {
            let mut input = cssparser::ParserInput::new(value);
            let mut parser = cssparser::Parser::new(&mut input);
            let margin = match parser.next() {
                Ok(cssparser::Token::Dimension { value, unit, .. })
                    if unit.eq_ignore_ascii_case("px") && value.is_finite() && value.abs() <= 1_000_000.0 => Self::Px(*value),
                Ok(cssparser::Token::Percentage { unit_value, .. })
                    if unit_value.is_finite() && unit_value.abs() <= 100.0 => Self::Percent(value.strip_suffix('%').and_then(|v| v.parse::<f32>().ok()).unwrap_or(*unit_value * 100.0)),
                Ok(cssparser::Token::Number { value, .. }) if *value == 0.0 => Self::Px(0.0),
                _ => return Err(Diagnostic::new("unsupported-value", source,
                    "Margin requires auto or a finite signed length (at most 1000000px) or percentage (at most 10000%)")),
            };
            parser.expect_exhausted().map_err(|_| Diagnostic::new("unsupported-value", source,
                "Margin requires one length, percentage or auto"))?;
            Ok(margin)
        }
    }
}

/// Percentages remain relative to the containing block's inline size at layout time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Spacing {
    Px(f32),
    Percent(f32),
}
impl Spacing {
    fn parse(value: &str, source: &str) -> Result<Self, Diagnostic> {
        match size(value, source)? {
            Size::Px(v) => Ok(Self::Px(v)),
            Size::Percent(v) => Ok(Self::Percent(v)),
            Size::Auto => Err(Diagnostic::new("unsupported-value", source,
                "Padding requires a nonnegative length or percentage")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Size {
    Auto,
    Px(f32),
    Percent(f32),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum LineHeight {
    Normal,
    Px(f32),
    Number(f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WhiteSpace {
    Normal,
    NoWrap,
    Pre,
    PreWrap,
    PreLine,
}
impl WhiteSpace {
    pub(crate) fn preserves_spaces(self) -> bool {
        matches!(self, Self::Pre | Self::PreWrap)
    }
    pub(crate) fn nowrap(self) -> bool {
        matches!(self, Self::NoWrap | Self::Pre)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CaseTransform {
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

#[derive(Clone, Debug)]
pub(crate) struct Style {
    pub custom_properties: crate::custom_properties::Computed,
    pub decoration: crate::decoration::Decoration,
    pub decoration_origins: Vec<crate::decoration::Origin>,
    pub width: Size,
    pub height: Size,
    pub content_box: bool,
    pub aspect_ratio: crate::aspect_ratio::AspectRatio,
    pub row: bool,
    pub reverse: bool,
    pub order: i32,
    pub z_index: Option<i32>,
    pub opacity: f32,
    pub align_self: crate::AlignSelf,
    pub align_content: crate::AlignContent,
    pub hidden: bool,
    pub overflow: crate::overflow::Overflow,
    pub overflow_clip_margin: crate::overflow_clip_margin::ClipMargin,
    pub text_ellipsis: bool,
    pub block_text: bool,
    pub padding: [Spacing; 4],       // top right bottom left
    pub gap: [f32; 2],           // row, column
    pub gradient: Option<crate::gradient::LinearGradient>,
    pub background: Option<u32>, // None resolves to this element's final color
    /// Physical edges in CSS top/right/bottom/left order.
    pub borders: [crate::border::Border; 4],
    pub inherited_color: u32,
    pub font_family: String,
    pub font_size: f32,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    pub white_space: WhiteSpace,
    pub text_transform: CaseTransform,
    pub language: String,
    pub font_weight: u16,
    pub line_height: LineHeight,
    pub color: u32,
    pub text_align: u32, // left, right, center
    pub align: u32,      // start, center, end, stretch
    pub justify: u32,    // start, center, end, space-between, space-around, space-evenly
    pub radii: [[crate::corner_radii::RadiusValue; 2]; 4],
    pub margin: [Margin; 4],
    pub position: crate::position::PositionMode,
    pub insets: [crate::position::Offset; 4],
    pub limits: [Option<Size>; 4], // min-width, max-width, min-height, max-height
    pub wrap: WrapMode,
    pub grow: f32,
    pub shrink: f32,
    pub basis: Size,
    pub definite_width: bool,
    pub definite_height: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            custom_properties: Default::default(),
            decoration: crate::decoration::Decoration::default(),
            decoration_origins: Vec::new(),
            width: Size::Auto,
            height: Size::Auto,
            content_box: false,
            aspect_ratio: Default::default(),
            row: false,
            reverse: false,
            order: 0,
            z_index: None,
            opacity: 1.0,
            align_self: crate::AlignSelf::Auto,
            align_content: crate::AlignContent::FlexStart,
            hidden: false,
            overflow: crate::overflow::Overflow::default(),
            overflow_clip_margin: crate::overflow_clip_margin::ClipMargin::default(),
            text_ellipsis: false,
            block_text: false,
            padding: [Spacing::Px(0.0); 4],
            gap: [0.0; 2],
            gradient: None,
            background: Some(0),
            borders: [crate::border::Border {
                width: crate::border::BorderWidth::Px(0.0),
                style: crate::border::BorderStyle::None,
                color: crate::border::BorderColor::Current,
            }; 4],
            inherited_color: 0xff000000,
            font_family: "Inter".into(),
            font_size: 16.0,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            white_space: WhiteSpace::Normal,
            text_transform: CaseTransform::None,
            language: String::new(),
            font_weight: 400,
            line_height: LineHeight::Px(24.0),
            color: 0xff000000,
            text_align: 0,
            align: 3,
            justify: 0,
            radii: [[crate::corner_radii::RadiusValue::Pixels(0.0); 2]; 4],
            margin: [Margin::Px(0.0); 4],
            position: crate::position::PositionMode::Static,
            insets: [crate::position::Offset::Auto; 4],
            limits: [None; 4],
            wrap: WrapMode::NoWrap,
            grow: 0.0,
            shrink: 0.0,
            basis: Size::Auto,
            definite_width: false,
            definite_height: false,
        }
    }
}

fn resolve_font_lengths(d: &Declaration, basis: Option<f32>) -> Result<Declaration, Diagnostic> {
    let mut resolved = d.clone();
    if !matches!(
        d.name.as_str(),
        "width"
            | "height"
            | "min-width"
            | "max-width"
            | "min-height"
            | "max-height"
            | "flex"
            | "flex-basis"
            | "margin"
            | "margin-top"
            | "margin-right"
            | "margin-bottom"
            | "margin-left"
            | "inset" | "top" | "right" | "bottom" | "left"
            | "padding"
            | "padding-top"
            | "padding-right"
            | "padding-bottom"
            | "padding-left"
            | "gap"
            | "row-gap"
            | "column-gap"
            | "border-radius" | "border-top-left-radius" | "border-top-right-radius" | "border-bottom-right-radius" | "border-bottom-left-radius"
            | "font-size"
            | "text-decoration-thickness"
            | "text-underline-offset"
            | "letter-spacing"
            | "word-spacing"
            | "line-height"
    ) {
        return Ok(resolved);
    }
    resolved.value = d
        .value
        .split_ascii_whitespace()
        .map(|token| {
            let lower = token.to_ascii_lowercase();
            let (unit_name, unit_basis) = if lower.ends_with("rem") {
                // Authored fragments cannot restyle the host html element.
                ("rem", Some(16.0))
            } else if lower.ends_with("em") {
                ("em", basis)
            } else {
                return Ok(token.to_owned());
            };
            let mut input = cssparser::ParserInput::new(token);
            let mut parser = cssparser::Parser::new(&mut input);
            let coefficient = match parser.next() {
                Ok(cssparser::Token::Dimension { value, unit, .. })
                    if unit.eq_ignore_ascii_case(unit_name) =>
                {
                    Some(*value)
                }
                _ => None,
            };
            let value = coefficient
                .filter(|_| parser.expect_exhausted().is_ok())
                .filter(|v| v.is_finite() && v.abs() <= 1_000_000.0 && (*v >= 0.0 || matches!(d.name.as_str(), "letter-spacing" | "word-spacing" | "text-underline-offset" | "margin" | "margin-top" | "margin-right" | "margin-bottom" | "margin-left" | "inset" | "top" | "right" | "bottom" | "left")))
                .ok_or_else(|| {
                    Diagnostic::new(
                        "unsupported-value",
                        &d.source,
                        "Expected a finite em/rem coefficient (negative only for margins, insets, letter/word spacing and underline offset) at most 1000000",
                    )
                })?;
            let pixels = match unit_basis {
                Some(basis) => value * basis,
                None => {
                    if value == 0.0 {
                        0.0
                    } else {
                        1.0
                    }
                }
            };
            if !pixels.is_finite() || pixels.abs() > 1_000_000.0 || (pixels < 0.0 && !matches!(d.name.as_str(), "letter-spacing" | "word-spacing" | "text-underline-offset" | "margin" | "margin-top" | "margin-right" | "margin-bottom" | "margin-left" | "inset" | "top" | "right" | "bottom" | "left")) {
                return Err(Diagnostic::new(
                    "unsupported-value",
                    &d.source,
                    "Resolved font-relative length exceeds the 1000000px limit",
                ));
            }
            Ok(format!("{pixels}px"))
        })
        .collect::<Result<Vec<_>, Diagnostic>>()?
        .join(" ");
    Ok(resolved)
}

fn size(value: &str, source: &str) -> Result<Size, Diagnostic> {
    if value == "auto" {
        return Ok(Size::Auto);
    }
    if let Some(v) = value.strip_suffix('%') {
        let v = v
            .parse::<f32>()
            .ok()
            .filter(|v| v.is_finite() && *v >= 0.0 && *v <= 10000.0)
            .ok_or_else(|| Diagnostic::new("unsupported-value", source, "Invalid percentage"))?;
        return Ok(Size::Percent(v));
    }
    Ok(Size::Px(length(value, source)?))
}

fn factor(value: &str, source: &str) -> Result<f32, Diagnostic> {
    value
        .parse::<f32>()
        .ok()
        .filter(|v| v.is_finite() && *v >= 0.0 && *v <= 10000.0)
        .ok_or_else(|| {
            Diagnostic::new(
                "unsupported-value",
                source,
                "Expected a finite flex factor in [0, 10000]",
            )
        })
}

fn flex(value: &str, source: &str) -> Result<(f32, f32, Size), Diagnostic> {
    match value {
        "none" => return Ok((0.0, 0.0, Size::Auto)),
        "auto" => return Ok((1.0, 1.0, Size::Auto)),
        "initial" => return Ok((0.0, 1.0, Size::Auto)),
        _ => {}
    }
    let tokens: Vec<_> = value.split_ascii_whitespace().collect();
    let zero = Size::Percent(0.0); // Match Chromium's omitted basis (0%, not 0px).
    match tokens.as_slice() {
        [a] => match factor(a, source) {
            Ok(grow) => Ok((grow, 1.0, zero)),
            Err(_) => Ok((1.0, 1.0, size(a, source)?)),
        },
        [a, b] => match (factor(a, source), factor(b, source)) {
            (Ok(grow), Ok(shrink)) => Ok((grow, shrink, zero)),
            (Ok(grow), Err(_)) => Ok((grow, 1.0, size(b, source)?)),
            (Err(_), Ok(grow)) => Ok((grow, 1.0, size(a, source)?)),
            _ => Err(Diagnostic::new(
                "unsupported-value",
                source,
                "Expected flex factors and at most one basis",
            )),
        },
        [a, b, c] => {
            if let (Ok(grow), Ok(shrink)) = (factor(a, source), factor(b, source)) {
                Ok((grow, shrink, size(c, source)?))
            } else {
                Ok((factor(b, source)?, factor(c, source)?, size(a, source)?))
            }
        }
        _ => Err(Diagnostic::new(
            "unsupported-value",
            source,
            "Expected none, auto, or one to three flex components",
        )),
    }
}

impl Style {
    pub fn content_auto_basis(&self, parent: &Self) -> bool {
        (self.grow > 0.0 || self.shrink > 0.0)
            && self.basis == Size::Auto
            && (if parent.row { self.width } else { self.height }) == Size::Auto
    }

    pub fn requires_indefinite_basis(&self, parent: &Self) -> bool {
        let parent_main_definite = if parent.row { parent.definite_width } else { parent.definite_height };
        let main = if parent.row { self.width } else { self.height };
        let effective_basis = if self.grow > 0.0 && self.basis == Size::Auto { main } else { self.basis };
        matches!(effective_basis, Size::Percent(_)) && !parent_main_definite
    }

    pub fn resolve_flex(&mut self, parent: &Self, _source: &str) -> Result<(), Diagnostic> {
        let parent_main_definite = if parent.row { parent.definite_width } else { parent.definite_height };
        self.definite_width = match self.width {
            Size::Px(_) => true,
            Size::Percent(_) => parent.definite_width,
            Size::Auto => !parent.row && parent.align == 3 && parent.definite_width,
        };
        self.definite_height = match self.height {
            Size::Px(_) => true,
            Size::Percent(_) => parent.definite_height,
            Size::Auto => parent.row && parent.align == 3 && parent.definite_height,
        };
        if self.grow > 0.0 && parent_main_definite || self.grow == 0.0 && self.basis != Size::Auto {
            if parent.row {
                self.definite_width = true;
            } else {
                self.definite_height = true;
            }
        }
        Ok(())
    }
    pub fn used_line_height(&self, normal_height: f32) -> f32 {
        match self.line_height {
            LineHeight::Normal => normal_height,
            LineHeight::Px(value) => value,
            LineHeight::Number(value) => value * self.font_size,
        }
    }
    pub fn background(&self) -> u32 {
        self.background.unwrap_or(self.color)
    }
    pub fn inherited(parent: &Self) -> Self {
        Self {
            custom_properties: parent.custom_properties.clone(),
            decoration: crate::decoration::Decoration {
                offset: parent.decoration.offset,
                skip_ink: parent.decoration.skip_ink,
                ..crate::decoration::Decoration::default()
            },
            decoration_origins: parent.decoration_origins.clone(),
            font_family: parent.font_family.clone(),
            font_size: parent.font_size,
            letter_spacing: parent.letter_spacing,
            word_spacing: parent.word_spacing,
            white_space: parent.white_space,
            text_transform: parent.text_transform,
            language: parent.language.clone(),
            font_weight: parent.font_weight,
            line_height: parent.line_height,
            color: parent.color,
            text_align: parent.text_align,
            inherited_color: parent.color,
            ..Self::default()
        }
    }
    pub fn compute(declarations: &[Declaration], parent: &Self) -> Result<Self, Diagnostic> {
        let mut custom = Vec::new();
        for d in declarations.iter().filter(|d| d.name.starts_with("--")) {
            let value = crate::custom_properties::Value::parse(&d.value)?;
            value.validate_custom()?;
            custom.push((d.name.clone(), value));
        }
        let names: std::collections::BTreeSet<_> = parent.custom_properties.keys().chain(custom.iter().map(|(n, _)| n)).collect();
        if names.len() > 512 { return Err(Diagnostic::new("input-limit", "css", "At most 512 custom properties per element are supported")); }
        let computed_custom = crate::custom_properties::compute(&parent.custom_properties, &custom);
        let mut expanded = Vec::new();
        for d in declarations.iter().filter(|d| !d.name.starts_with("--")) {
            let mut d = d.clone();
            let value = crate::custom_properties::Value::parse(&d.value)?;
            if value.has_variables() {
                d.value = match value.resolve(&computed_custom) {
                    Some(text) => {
                        if crate::substitution_validity::invalid_top_level_component(&d.name, &text) {
                            "unset".into()
                        } else {
                            let normalized = crate::css::property_value(&d.name, &text)?;
                            if crate::substitution_validity::definitely_invalid(&d.name, &normalized) { "unset".into() } else { normalized }
                        }
                    },
                    None => "unset".into(),
                };
            }
            expanded.extend(Self::expand_font(&d)?);
        }
        // Font size is computed first, independent of declaration order. Its em
        // basis is the parent's computed size, including repeated declarations.
        let mut font = Self::inherited(parent);
        for d in expanded
            .iter()
            .filter(|d| matches!(d.name.as_str(), "font-size" | "font"))
        {
            font.apply_cascaded(&resolve_font_lengths(d, Some(parent.font_size))?, parent)?;
        }
        let mut result = Self::inherited(parent);
        result.custom_properties = computed_custom;
        result.font_size = font.font_size;
        for d in &expanded {
            let basis = if d.name == "font-size" {
                parent.font_size
            } else {
                font.font_size
            };
            if d.name == "aspect-ratio" && !["initial", "inherit", "unset"].iter().any(|keyword| d.value.eq_ignore_ascii_case(keyword)) {
                // Use the final font pass even when an earlier font-size
                // declaration was reapplied during this cascade walk.
                result.aspect_ratio = crate::aspect_ratio::AspectRatio::parse_with_font_size(&d.value, &d.source, font.font_size)?;
            } else if d.name == "overflow-clip-margin" && !["initial", "inherit", "unset"].iter().any(|keyword| d.value.eq_ignore_ascii_case(keyword)) {
                result.overflow_clip_margin = crate::overflow_clip_margin::ClipMargin::parse(&d.value, font.font_size, &d.source)?;
            } else {
                result.apply_cascaded(&resolve_font_lengths(d, Some(basis))?, parent)?;
            }
        }
        if let Some(gradient) = &mut result.gradient {
            for stop in &mut gradient.stops {
                let pixels = match stop.position {
                    Some(crate::gradient::Position::Em(v)) => Some(v as f64 * font.font_size as f64),
                    Some(crate::gradient::Position::Rem(v)) => Some(v as f64 * 16.),
                    _ => None,
                };
                if let Some(pixels) = pixels {
                    let pixels = pixels as f32;
                    if !pixels.is_finite() { return Err(Diagnostic::new("unsupported-linear-gradient", "css", "Gradient stop length exceeds the finite runtime range")); }
                    stop.position = Some(crate::gradient::Position::Pixels(pixels));
                }
            }
        }
        // Resolve once after the whole cascade. Explicit inheritance copies
        // computed pixel widths, including zero for none/hidden styles.
        for border in &mut result.borders {
            border.width = crate::border::BorderWidth::Px(border.used_width(font.font_size, "css")?);
        }
        let overflow = result.overflow.computed();
        if matches!(overflow.x, crate::overflow::Value::Auto) || matches!(overflow.y, crate::overflow::Value::Auto) {
            let source = expanded.iter().rev().find(|d| matches!(d.name.as_str(), "overflow"|"overflow-x"|"overflow-y"))
                .map(|d|d.source.as_str()).unwrap_or("css");
            return Err(Diagnostic::new("unsupported-computed-overflow", source,
                "visible paired with hidden computes to auto; scrolling overflow is unsupported. Use clip with visible for a one-axis clip"));
        }
        if let Some(origin) = result.decoration.origin(result.color) {
            result.decoration_origins.push(origin);
        }
        Ok(result)
    }

    pub fn validate_declaration(d: &Declaration) -> Result<(), Diagnostic> {
        let value = crate::custom_properties::Value::parse(&d.value)?;
        if d.name.starts_with("--") { return value.validate_custom(); }
        if value.has_variables() {
            let mut initial = d.clone(); initial.value = "initial".into();
            for component in Self::expand_font(&initial)? {
                match Self::default().apply_initial(&component) {
                    Err(e) if e.code == "unsupported-initial-value" => {},
                    result => result?,
                }
            }
            return Ok(());
        }
        let parent = Self::default();
        let mut style = Self::default();
        for d in Self::expand_font(d)? {
            // Unmatched declarations have no font context. Check em syntax and
            // sign here; enforce resolved pixel limits at the matching element.
            style.apply_cascaded(&resolve_font_lengths(&d, None)?, &parent)?;
        }
        Ok(())
    }

    fn apply_cascaded(&mut self, d: &Declaration, parent: &Self) -> Result<(), Diagnostic> {
        let inherited_property = matches!(
            d.name.as_str(),
            "color"
                | "font"
                | "font-family"
                | "font-size"
                | "font-weight"
                | "line-height"
                | "text-align"
                | "letter-spacing"
                | "word-spacing"
                | "white-space"
                | "text-transform"
                | "text-underline-offset"
                | "text-decoration-skip-ink"
        );
        if d.value.eq_ignore_ascii_case("initial")
            || (d.value.eq_ignore_ascii_case("unset") && !inherited_property)
        {
            return self.apply_initial(d);
        }
        if !d.value.eq_ignore_ascii_case("inherit")
            && !(d.value.eq_ignore_ascii_case("unset") && inherited_property)
        {
            return self.apply(d);
        }
        // Copy computed values, retaining percentages and currentColor rather
        // than baking the parent's used dimensions or resolved background.
        match d.name.as_str() {
            name if crate::border::side_property(name).is_some() => {
                let (edge, component) = crate::border::side_property(name).unwrap();
                match component {
                    "border" => self.borders[edge] = parent.borders[edge],
                    "border-width" => self.borders[edge].width = parent.borders[edge].width,
                    "border-style" => self.borders[edge].style = parent.borders[edge].style,
                    "border-color" => self.borders[edge].color = parent.borders[edge].color,
                    _ => unreachable!(),
                }
            },
            "border" => self.borders = parent.borders,
            "border-width" => for (border, inherited) in self.borders.iter_mut().zip(parent.borders) { border.width = inherited.width; },
            "border-style" => for (border, inherited) in self.borders.iter_mut().zip(parent.borders) { border.style = inherited.style; },
            "border-color" => for (border, inherited) in self.borders.iter_mut().zip(parent.borders) { border.color = inherited.color; },
            "overflow-clip-margin" => self.overflow_clip_margin = parent.overflow_clip_margin,
            "overflow" => self.overflow = parent.overflow.computed(),
            "overflow-x" => self.overflow.x = parent.overflow.computed().x,
            "overflow-y" => self.overflow.y = parent.overflow.computed().y,
            "text-overflow" => self.text_ellipsis = parent.text_ellipsis,
            "width" => self.width = parent.width,
            "height" => self.height = parent.height,
            "display" => {
                self.hidden = parent.hidden;
                self.block_text = parent.block_text;
            }
            "flex-direction" => { self.row = parent.row; self.reverse = parent.reverse; },
            "order" => self.order = parent.order,
            "z-index" => self.z_index = parent.z_index,
            "opacity" => self.opacity = parent.opacity,
            "flex-wrap" => self.wrap = parent.wrap,
            "flex-grow" => self.grow = parent.grow,
            "flex-shrink" => self.shrink = parent.shrink,
            "flex-basis" => self.basis = parent.basis,
            "flex" => {
                (self.grow, self.shrink, self.basis) = (parent.grow, parent.shrink, parent.basis)
            }
            "padding" => self.padding = parent.padding,
            "margin" => self.margin = parent.margin,
            "position" => self.position = parent.position,
            "inset" => self.insets = parent.insets,
            "top" => self.insets[0] = parent.insets[0],
            "right" => self.insets[1] = parent.insets[1],
            "bottom" => self.insets[2] = parent.insets[2],
            "left" => self.insets[3] = parent.insets[3],
            "padding-top" => self.padding[0] = parent.padding[0],
            "padding-right" => self.padding[1] = parent.padding[1],
            "padding-bottom" => self.padding[2] = parent.padding[2],
            "padding-left" => self.padding[3] = parent.padding[3],
            "margin-top" => self.margin[0] = parent.margin[0],
            "margin-right" => self.margin[1] = parent.margin[1],
            "margin-bottom" => self.margin[2] = parent.margin[2],
            "margin-left" => self.margin[3] = parent.margin[3],
            "gap" => self.gap = parent.gap,
            "row-gap" => self.gap[0] = parent.gap[0],
            "column-gap" => self.gap[1] = parent.gap[1],
            "min-width" => self.limits[0] = parent.limits[0],
            "max-width" => self.limits[1] = parent.limits[1],
            "min-height" => self.limits[2] = parent.limits[2],
            "max-height" => self.limits[3] = parent.limits[3],
            "background" => { self.background = parent.background; self.gradient = parent.gradient.clone(); },
            "background-color" => self.background = parent.background,
            "background-image" => self.gradient = parent.gradient.clone(),
            "border-radius" => self.radii = parent.radii,
            "border-top-left-radius" => self.radii[0] = parent.radii[0],
            "border-top-right-radius" => self.radii[1] = parent.radii[1],
            "border-bottom-right-radius" => self.radii[2] = parent.radii[2],
            "border-bottom-left-radius" => self.radii[3] = parent.radii[3],
            "align-items" => self.align = parent.align,
            "align-self" => self.align_self = parent.align_self,
            "align-content" => self.align_content = parent.align_content,
            "justify-content" => self.justify = parent.justify,
            "box-sizing" => self.content_box = parent.content_box,
            "aspect-ratio" => self.aspect_ratio = parent.aspect_ratio,
            "color" => self.color = parent.color,
            "font" => {
                self.font_family = parent.font_family.clone();
                self.font_size = parent.font_size;
                self.font_weight = parent.font_weight;
                self.line_height = parent.line_height;
            }
            "font-family" => self.font_family = parent.font_family.clone(),
            "font-size" => self.font_size = parent.font_size,
            "font-weight" => self.font_weight = parent.font_weight,
            "line-height" => self.line_height = parent.line_height,
            "text-align" => self.text_align = parent.text_align,
            "letter-spacing" => self.letter_spacing = parent.letter_spacing,
            "word-spacing" => self.word_spacing = parent.word_spacing,
            "white-space" => self.white_space = parent.white_space,
            "text-transform" => self.text_transform = parent.text_transform,
            "text-decoration-line" => {
                self.decoration.underline = parent.decoration.underline;
                self.decoration.strikethrough = parent.decoration.strikethrough;
            }
            "text-decoration-color" => self.decoration.color = parent.decoration.color,
            "text-decoration-thickness" => self.decoration.thickness = parent.decoration.thickness,
            "text-underline-offset" => self.decoration.offset = parent.decoration.offset,
            "text-decoration-skip-ink" => self.decoration.skip_ink = parent.decoration.skip_ink,
            "text-decoration-style" | "text-underline-position" => {}
            _ => return self.apply(d),
        }
        Ok(())
    }
    fn apply_initial(&mut self, d: &Declaration) -> Result<(), Diagnostic> {
        if let Some((_, component)) = crate::border::side_property(&d.name) {
            let mut initial = d.clone();
            initial.value = match component { "border" => "medium none currentcolor", "border-width" => "medium",
                "border-style" => "none", "border-color" => "currentcolor", _ => unreachable!() }.into();
            return self.apply(&initial);
        }
        // CSS initial values are independent of the author's reset stylesheet.
        // Values requiring unsupported layout or environment-dependent fonts/
        // colors are rejected rather than replaced by convenient profile values.
        let value = match d.name.as_str() {
            "box-sizing" => "content-box",
            "position" => "static",
            "inset" | "top" | "right" | "bottom" | "left" => "auto",
            "aspect-ratio" => "auto",
            "width" | "height" | "flex-basis" => "auto",
            "flex" => "0 1 auto",
            "flex-grow" => "0",
            "flex-shrink" => "1",
            "order" => "0",
            "z-index" => "auto",
            "opacity" => "1",
            "flex-direction" => "row",
            "flex-wrap" => "nowrap",
            "padding" | "margin" | "padding-top" | "padding-right" | "padding-bottom"
            | "padding-left" | "margin-top" | "margin-right" | "margin-bottom" | "margin-left"
            | "gap" | "row-gap" | "column-gap" | "border-radius" | "border-top-left-radius" | "border-top-right-radius" | "border-bottom-right-radius" | "border-bottom-left-radius" => "0",
            "border" => "medium none currentcolor",
            "border-width" => "medium",
            "border-style" => "none",
            "border-color" => "currentcolor",
            "overflow-clip-margin" => "0px",
            "overflow" | "overflow-x" | "overflow-y" => "visible",
            "text-overflow" => "clip",
            "align-self" => "auto",
            "align-content" => "stretch",
            "align-items" => "stretch", // normal behaves as stretch in flex layout.
            "justify-content" => "flex-start", // normal behaves as start in flex layout.
            "background" | "background-color" => "transparent",
            "background-image" => "none",
            "font-size" => "16px", // medium in the pinned Chromium reference environment.
            "font-weight" => "400",
            "line-height" | "letter-spacing" | "word-spacing" | "white-space" => "normal",
            "text-transform" | "text-decoration-line" => "none",
            "text-decoration-color" => "currentcolor",
            "text-decoration-style" => "solid",
            "text-decoration-thickness"
            | "text-underline-offset"
            | "text-decoration-skip-ink"
            | "text-underline-position" => "auto",
            "text-align" => "left", // start in the supported left-to-right profile.
            "max-width" => {
                self.limits[1] = None;
                return Ok(());
            }
            "max-height" => {
                self.limits[3] = None;
                return Ok(());
            }
            "display" | "min-width" | "min-height" | "font" | "font-family"
            | "color" => {
                return Err(Diagnostic::new(
                    "unsupported-initial-value",
                    &d.source,
                    format!(
                        "The CSS initial value of {} is outside the supported layout/font/color profile",
                        d.name
                    ),
                ));
            }
            _ => return self.apply(d),
        };
        let mut initial = d.clone();
        initial.value = value.into();
        self.apply(&initial)
    }
    fn expand_font(d: &Declaration) -> Result<Vec<Declaration>, Diagnostic> {
        if d.name == "text-decoration" {
            return crate::decoration::expand(d);
        }
        if d.name != "font"
            || ["inherit", "initial", "unset"]
                .iter()
                .any(|v| d.value.eq_ignore_ascii_case(v))
        {
            return Ok(vec![d.clone()]);
        }
        use cssparser::{Parser, ParserInput, ToCss, Token};
        let invalid = || {
            Diagnostic::new(
                "unsupported-font-shorthand",
                &d.source,
                "Expected optional normal/weight prefixes, positive px/em/rem size, optional / line-height and one font family; styled faces and fallback lists are not yet supported",
            )
        };
        let mut input = ParserInput::new(&d.value);
        let mut parser = Parser::new(&mut input);
        let mut weight = None;
        let mut normals = 0;
        let size = loop {
            let token = parser.next().map_err(|_| invalid())?;
            match token {
                Token::Dimension { unit, .. }
                    if unit.eq_ignore_ascii_case("px")
                        || unit.eq_ignore_ascii_case("em")
                        || unit.eq_ignore_ascii_case("rem") =>
                {
                    break token.to_css_string();
                }
                Token::Ident(value) if value.eq_ignore_ascii_case("normal") => normals += 1,
                Token::Ident(value) if value.eq_ignore_ascii_case("bold") && weight.is_none() => {
                    weight = Some("700".to_owned());
                }
                Token::Number {
                    int_value: Some(value),
                    ..
                } if (100..=900).contains(value) && weight.is_none() => {
                    weight = Some(value.to_string());
                }
                _ => return Err(invalid()),
            }
            // Four optional slots: style, variant, weight and width. Only
            // normal is representable for the three non-weight slots today.
            if normals + usize::from(weight.is_some()) > 4 {
                return Err(invalid());
            }
        };
        let line_height = if parser.try_parse(|p| p.expect_delim('/')).is_ok() {
            match parser.next().map_err(|_| invalid())? {
                token @ Token::Number { .. } | token @ Token::Dimension { .. } => {
                    token.to_css_string()
                }
                Token::Ident(value) if value.eq_ignore_ascii_case("normal") => "normal".into(),
                _ => return Err(invalid()),
            }
        } else {
            "normal".into()
        };
        let family_start = parser.position();
        parser.expect_ident_or_string().map_err(|_| invalid())?;
        parser.expect_exhausted().map_err(|_| invalid())?;
        let family = parser.slice_from(family_start).trim().to_owned();
        // Omitted weight resets to normal. Other font subproperties currently
        // cannot be authored and are fixed at normal throughout this profile.
        Ok([
            ("font-size", size),
            ("line-height", line_height),
            ("font-family", family),
            ("font-weight", weight.unwrap_or_else(|| "400".into())),
        ]
        .into_iter()
        .map(|(name, value)| Declaration {
            name: name.into(),
            value,
            important: d.important,
            source: d.source.clone(),
        })
        .collect())
    }
    pub fn apply(&mut self, d: &Declaration) -> Result<(), Diagnostic> {
        if let Some((edge, component)) = crate::border::side_property(&d.name) {
            let border = &mut self.borders[edge];
            match component {
                "border" => *border = crate::border::Border::parse(&d.value, &d.source)?,
                "border-width" => border.width = crate::border::BorderWidth::parse(&d.value, &d.source)?,
                "border-style" => border.style = crate::border::BorderStyle::parse(&d.value, &d.source)?,
                "border-color" => border.color = crate::border::BorderColor::parse(&d.value, &d.source)?,
                _ => unreachable!(),
            }
            return Ok(());
        }
        let mut normalized = d.clone();
        if normalized.name != "font-family" && normalized.name != "font" {
            normalized.value.make_ascii_lowercase();
        }
        let d = &normalized;
        if self
            .decoration
            .apply(&d.name, &d.value, self.font_size, &d.source)?
        {
            return Ok(());
        }

        match d.name.as_str() {
            "width" => self.width = size(&d.value, &d.source)?,
            "height" => self.height = size(&d.value, &d.source)?,
            "flex-grow" => self.grow = factor(&d.value, &d.source)?,
            "flex-shrink" => self.shrink = factor(&d.value, &d.source)?,
            "flex-basis" => self.basis = size(&d.value, &d.source)?,
            "flex" => (self.grow, self.shrink, self.basis) = flex(&d.value, &d.source)?,
            "min-width" | "max-width" | "min-height" | "max-height" => {
                let index = match d.name.as_str() {
                    "min-width" => 0,
                    "max-width" => 1,
                    "min-height" => 2,
                    _ => 3,
                };
                let limit = size(&d.value, &d.source)?;
                if limit == Size::Auto {
                    return Err(Diagnostic::new(
                        "unsupported-value",
                        &d.source,
                        "Expected a nonnegative length or percentage limit",
                    ));
                }
                self.limits[index] = Some(limit);
            }
            "flex-wrap" => {
                self.wrap = match d.value.as_str() {
                    "wrap" => WrapMode::Wrap,
                    "wrap-reverse" => WrapMode::WrapReverse,
                    "nowrap" => WrapMode::NoWrap,
                    _ => {
                        return Err(Diagnostic::new(
                            "unsupported-value",
                            &d.source,
                            "Expected nowrap, wrap or wrap-reverse",
                        ));
                    }
                }
            }
            "text-overflow" => {
                self.text_ellipsis = match d.value.as_str() {
                    "clip" => false,
                    "ellipsis" => true,
                    _ => return Err(Diagnostic::new("unsupported-text-overflow", &d.source,
                        "Expected clip or ellipsis; strings, two-value syntax and fade are not supported")),
                };
            }
            "overflow-clip-margin" => self.overflow_clip_margin = crate::overflow_clip_margin::ClipMargin::parse(&d.value, self.font_size, &d.source)?,
            "overflow" | "overflow-x" | "overflow-y" => {
                let parse = |value: &str| match value {
                    "visible" => Ok(crate::overflow::Value::Visible),
                    "clip" => Ok(crate::overflow::Value::Clip),
                    "hidden" => Ok(crate::overflow::Value::Hidden),
                    _ => Err(Diagnostic::new("unsupported-overflow", &d.source,
                        "Expected visible, clip or static hidden; scrolling values are unsupported")),
                };
                let values: Vec<_> = d.value.split_ascii_whitespace().collect();
                match (d.name.as_str(), values.as_slice()) {
                    ("overflow", [x]) => self.overflow = crate::overflow::Overflow::uniform(parse(x)?),
                    ("overflow", [x,y]) => self.overflow = crate::overflow::Overflow { x:parse(x)?, y:parse(y)? },
                    ("overflow-x", [x]) => self.overflow.x = parse(x)?,
                    ("overflow-y", [y]) => self.overflow.y = parse(y)?,
                    _ => return Err(Diagnostic::new("unsupported-overflow", &d.source,
                        "overflow accepts one or two values; axis longhands accept one value")),
                }
            }
            "display" => {
                self.block_text = d.value == "block";
                self.hidden = match d.value.as_str() {
                    "flex" | "block" => false,
                    "none" => true,
                    _ => {
                        return Err(Diagnostic::new(
                            "unsupported-display",
                            &d.source,
                            "Only flex, text-only block and none are supported; CSS Grid is intentionally unsupported",
                        ));
                    }
                }
            }
            "opacity" => self.opacity = crate::opacity::parse(&d.value, &d.source)?,
            "z-index" => {
                self.z_index = if d.value == "auto" { None } else {
                    let digits = d.value.strip_prefix(['+', '-']).unwrap_or(&d.value);
                    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                        return Err(Diagnostic::new("unsupported-value", &d.source, "z-index requires auto or a signed integer"));
                    }
                    Some(d.value.parse().map_err(|_| Diagnostic::new("unsupported-value", &d.source,
                        "z-index must fit a signed 32-bit integer"))?)
                };
            }
            "order" => {
                let digits = d.value.strip_prefix(['+', '-']).unwrap_or(&d.value);
                if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(Diagnostic::new("unsupported-value", &d.source, "order requires a signed integer"));
                }
                self.order = d.value.parse().map_err(|_| Diagnostic::new(
                    "unsupported-value", &d.source, "order must fit a signed 32-bit integer",
                ))?;
            }
            "flex-direction" => {
                (self.row, self.reverse) = match d.value.as_str() {
                    "row" => (true, false),
                    "row-reverse" => (true, true),
                    "column" => (false, false),
                    "column-reverse" => (false, true),
                    _ => {
                        return Err(Diagnostic::new(
                            "unsupported-value",
                            &d.source,
                            "Expected row, row-reverse, column or column-reverse",
                        ));
                    }
                }
            }
            "position" => self.position = match d.value.as_str() {
                "static" => crate::position::PositionMode::Static, "relative" => crate::position::PositionMode::Relative,
                "absolute" => crate::position::PositionMode::Absolute,
                _ => return Err(Diagnostic::new("unsupported-position", &d.source,
                    "Expected static, relative or absolute; fixed and sticky positioning are not supported")),
            },
            "inset" => self.insets = crate::position::Offset::shorthand(&d.value, &d.source)?,
            "top" | "right" | "bottom" | "left" => {
                let index = match d.name.as_str() { "top" => 0, "right" => 1, "bottom" => 2, _ => 3 };
                self.insets[index] = crate::position::Offset::parse(&d.value, &d.source)?;
            }
            "margin" => {
                let values = d.value.split_ascii_whitespace()
                    .map(|v| Margin::parse(v, &d.source))
                    .collect::<Result<Vec<_>, _>>()?;
                self.margin = match values.as_slice() {
                    [a] => [*a; 4],
                    [a, b] => [*a, *b, *a, *b],
                    [a, b, c] => [*a, *b, *c, *b],
                    [a, b, c, d] => [*a, *b, *c, *d],
                    _ => return Err(Diagnostic::new(
                        "unsupported-value", &d.source,
                        "Margin takes one to four signed lengths, percentages or auto",
                    )),
                };
            }
            "padding" => {
                let values = d
                    .value
                    .split_ascii_whitespace()
                    .map(|v| Spacing::parse(v, &d.source))
                    .collect::<Result<Vec<_>, _>>()?;
                let edges = match values.as_slice() {
                    [a] => [*a; 4],
                    [a, b] => [*a, *b, *a, *b],
                    [a, b, c] => [*a, *b, *c, *b],
                    [a, b, c, d] => [*a, *b, *c, *d],
                    _ => {
                        return Err(Diagnostic::new(
                            "unsupported-value",
                            &d.source,
                            "Padding takes one to four nonnegative lengths or percentages",
                        ));
                    }
                };
                self.padding = edges;
            }
            "margin-top" | "margin-right" | "margin-bottom" | "margin-left" => {
                let i = match d.name.as_str() {
                    "margin-top" => 0,
                    "margin-right" => 1,
                    "margin-bottom" => 2,
                    _ => 3,
                };
                self.margin[i] = Margin::parse(&d.value, &d.source)?;
            }
            "padding-top" | "padding-right" | "padding-bottom" | "padding-left" => {
                let i = match d.name.as_str() {
                    "padding-top" => 0,
                    "padding-right" => 1,
                    "padding-bottom" => 2,
                    _ => 3,
                };
                self.padding[i] = Spacing::parse(&d.value, &d.source)?;
            }
            "gap" => {
                let values = d
                    .value
                    .split_ascii_whitespace()
                    .map(|v| length(v, &d.source))
                    .collect::<Result<Vec<_>, _>>()?;
                self.gap = match values.as_slice() {
                    [a] => [*a; 2],
                    [a, b] => [*a, *b],
                    _ => {
                        return Err(Diagnostic::new(
                            "unsupported-value",
                            &d.source,
                            "Gap takes one or two px lengths",
                        ));
                    }
                };
            }
            "row-gap" => self.gap[0] = length(&d.value, &d.source)?,
            "column-gap" => self.gap[1] = length(&d.value, &d.source)?,
            "background-image" => {
                self.gradient = if d.value == "none" { None } else { Some(crate::gradient::parse(&d.value, &d.source)?) };
            }
            "background" if crate::gradient::starts_linear(&d.value) => {
                self.gradient = Some(crate::gradient::parse(&d.value, &d.source)?);
                self.background = Some(0);
            }
            "background" if d.value == "none" => { self.background = Some(0); self.gradient = None; },
            "background" | "background-color" => {
                if d.name == "background" { self.gradient = None; }
                self.background = if d.value == "currentcolor" {
                    None
                } else {
                    Some(color(&d.value, &d.source)?)
                };
            }
            "border-radius" => self.radii = crate::corner_radii::shorthand(&d.value, &d.source)?,
            "border-top-left-radius" => self.radii[0] = crate::corner_radii::longhand(&d.value, &d.source)?,
            "border-top-right-radius" => self.radii[1] = crate::corner_radii::longhand(&d.value, &d.source)?,
            "border-bottom-right-radius" => self.radii[2] = crate::corner_radii::longhand(&d.value, &d.source)?,
            "border-bottom-left-radius" => self.radii[3] = crate::corner_radii::longhand(&d.value, &d.source)?,
            "border" => self.borders = [crate::border::Border::parse(&d.value, &d.source)?; 4],
            "border-width" => {
                let values = crate::border::parse_side_values(&d.value, &d.source, crate::border::BorderWidth::parse)?;
                for (border, value) in self.borders.iter_mut().zip(values) { border.width = value; }
            },
            "border-style" => {
                let values = crate::border::parse_side_values(&d.value, &d.source, crate::border::BorderStyle::parse)?;
                for (border, value) in self.borders.iter_mut().zip(values) { border.style = value; }
            },
            "border-color" => {
                let values = crate::border::parse_side_values(&d.value, &d.source, crate::border::BorderColor::parse)?;
                for (border, value) in self.borders.iter_mut().zip(values) { border.color = value; }
            },
            "align-content" => {
                self.align_content = match d.value.as_str() {
                    "flex-start" => crate::AlignContent::FlexStart,
                    "center" => crate::AlignContent::Center,
                    "flex-end" => crate::AlignContent::FlexEnd,
                    "stretch" => crate::AlignContent::Stretch,
                    "space-between" => crate::AlignContent::SpaceBetween,
                    "space-around" => crate::AlignContent::SpaceAround,
                    "space-evenly" => crate::AlignContent::SpaceEvenly,
                    _ => return Err(Diagnostic::new("unsupported-value", &d.source,
                        "Expected flex-start, center, flex-end, stretch, space-between, space-around or space-evenly")),
                };
            }
            "align-self" => {
                self.align_self = match d.value.as_str() {
                    "auto" => crate::AlignSelf::Auto,
                    "flex-start" => crate::AlignSelf::FlexStart,
                    "center" => crate::AlignSelf::Center,
                    "flex-end" => crate::AlignSelf::FlexEnd,
                    "stretch" => crate::AlignSelf::Stretch,
                    "baseline" => crate::AlignSelf::Baseline,
                    _ => return Err(Diagnostic::new("unsupported-value", &d.source,
                        "Expected auto, flex-start, center, flex-end, stretch or baseline")),
                };
            }
            "align-items" => {
                self.align = match d.value.as_str() {
                    "flex-start" => 0,
                    "center" => 1,
                    "flex-end" => 2,
                    "stretch" => 3,
                    _ => {
                        return Err(Diagnostic::new(
                            "unsupported-value",
                            &d.source,
                            "Expected flex-start, center, flex-end or stretch",
                        ));
                    }
                }
            }
            "justify-content" => {
                self.justify = match d.value.as_str() {
                    "flex-start" => 0,
                    "center" => 1,
                    "flex-end" => 2,
                    "space-between" => 3,
                    "space-around" => 4,
                    "space-evenly" => 5,
                    _ => {
                        return Err(Diagnostic::new(
                            "unsupported-value",
                            &d.source,
                            "Expected flex-start, center, flex-end, space-between, space-around or space-evenly",
                        ));
                    }
                }
            }
            "aspect-ratio" => self.aspect_ratio = crate::aspect_ratio::AspectRatio::parse(&d.value, &d.source)?,
            "box-sizing" if matches!(d.value.as_str(), "border-box" | "content-box") => {
                self.content_box = d.value == "content-box";
            }
            "color" => {
                self.color = if d.value == "currentcolor" {
                    self.inherited_color
                } else {
                    color(&d.value, &d.source)?
                };
            }
            "text-align" => {
                self.text_align = match d.value.as_str() {
                    "left" => 0,
                    "right" => 1,
                    "center" => 2,
                    _ => {
                        return Err(Diagnostic::new(
                            "unsupported-value",
                            &d.source,
                            "Expected left, center or right text alignment",
                        ));
                    }
                };
            }
            "font" => {
                return Err(Diagnostic::new(
                    "unsupported-font-shorthand",
                    &d.source,
                    "Font shorthand must be expanded before computing style",
                ));
            }
            "font-family" => {
                let mut input = cssparser::ParserInput::new(&d.value);
                let mut p = cssparser::Parser::new(&mut input);
                let family = p
                    .expect_ident_or_string()
                    .map_err(|_| {
                        Diagnostic::new(
                            "unsupported-value",
                            &d.source,
                            "Expected one font family, quoted if it contains spaces",
                        )
                    })?
                    .to_string();
                p.expect_exhausted().map_err(|_| {
                    Diagnostic::new(
                        "unsupported-value",
                        &d.source,
                        "Font fallback lists are unsupported",
                    )
                })?;
                self.font_family = family;
            }
            "text-transform" => {
                self.text_transform = match d.value.as_str() {
                    "none" => CaseTransform::None,
                    "uppercase" => CaseTransform::Uppercase,
                    "lowercase" => CaseTransform::Lowercase,
                    "capitalize" => CaseTransform::Capitalize,
                    _ => {
                        return Err(Diagnostic::new(
                            "unsupported-value",
                            &d.source,
                            "Expected none, uppercase, lowercase or capitalize; language-specific transforms are not implemented yet",
                        ));
                    }
                };
            }
            "white-space" => {
                self.white_space = match d.value.as_str() {
                    "normal" => WhiteSpace::Normal,
                    "nowrap" => WhiteSpace::NoWrap,
                    "pre" => WhiteSpace::Pre,
                    "pre-wrap" => WhiteSpace::PreWrap,
                    "pre-line" => WhiteSpace::PreLine,
                    _ => {
                        return Err(Diagnostic::new(
                            "unsupported-value",
                            &d.source,
                            "Expected normal, nowrap, pre, pre-wrap or pre-line; other preserved whitespace modes require separate support",
                        ));
                    }
                };
            }
            "letter-spacing" | "word-spacing" => {
                let spacing = if d.value.eq_ignore_ascii_case("normal") {
                    0.0
                } else {
                    let mut input = cssparser::ParserInput::new(&d.value);
                    let mut parser = cssparser::Parser::new(&mut input);
                    let value = match parser.next() {
                        Ok(cssparser::Token::Dimension { value, unit, .. })
                            if unit.eq_ignore_ascii_case("px") =>
                        {
                            Some(*value)
                        }
                        Ok(cssparser::Token::Number { value, .. }) if *value == 0.0 => Some(0.0),
                        _ => None,
                    };
                    value
                        .filter(|_| parser.expect_exhausted().is_ok())
                        .filter(|v| v.is_finite() && v.abs() <= 1_000_000.0)
                        .ok_or_else(|| {
                            Diagnostic::new(
                                "unsupported-value",
                                &d.source,
                                "Expected normal or a finite px/em/rem spacing within +/-1000000px",
                            )
                        })?
                };
                if d.name == "word-spacing" {
                    self.word_spacing = spacing;
                } else {
                    self.letter_spacing = spacing;
                }
            }
            "font-size" => {
                // Match Chromium's computed-font ceiling before inheritance
                // and em resolution, not only when emitting a text object.
                self.font_size = length(&d.value, &d.source)?.min(10_000.0);
                if self.font_size == 0.0 {
                    return Err(Diagnostic::new(
                        "unsupported-value",
                        &d.source,
                        "Font size must be positive",
                    ));
                }
            }
            "font-weight" => {
                self.font_weight = match d.value.as_str() {
                    "normal" => 400,
                    "bold" => 700,
                    v => v
                        .parse::<u16>()
                        .ok()
                        .filter(|w| (100..=900).contains(w))
                        .ok_or_else(|| {
                            Diagnostic::new(
                                "unsupported-value",
                                &d.source,
                                "Expected normal, bold or weight 100–900",
                            )
                        })?,
                };
            }
            "line-height" => {
                self.line_height = if d.value == "normal" {
                    LineHeight::Normal
                } else if let Ok(number) = d.value.parse::<f32>() {
                    if !number.is_finite() || number <= 0.0 || number > 10000.0 {
                        return Err(Diagnostic::new(
                            "unsupported-value",
                            &d.source,
                            "Expected a positive finite line-height multiplier at most 10000",
                        ));
                    }
                    LineHeight::Number(number)
                } else {
                    let pixels = length(&d.value, &d.source)?;
                    if pixels == 0.0 {
                        return Err(Diagnostic::new(
                            "unsupported-value",
                            &d.source,
                            "Line height must be positive",
                        ));
                    }
                    LineHeight::Px(pixels)
                };
            }

            _ => {
                return Err(Diagnostic::new(
                    "unsupported-property",
                    &d.source,
                    format!("Unsupported CSS property: {}", d.name),
                ));
            }
        }
        Ok(())
    }

    pub fn write(
        &self,
        layout: &mut Record,
        style: &mut Record,
        parent: &Self,
    ) -> Result<(), Diagnostic> {
        if let Some(ratio) = self.aspect_ratio.ratio {
            style.set("aspectRatio", Value::Float(ratio))?;
        }
        let parent_row = parent.row;
        let independent_factors = self.grow != self.shrink || self.content_auto_basis(parent)
            || matches!(self.basis, Size::Percent(_));
        for (name, size, cross) in [
            ("width", self.width, !parent_row),
            ("height", self.height, parent_row),
        ] {
            let main = !cross;
            let size = if main && !independent_factors && self.basis != Size::Auto && self.grow == 0.0 {
                self.basis
            } else {
                size
            };
            let (value, units, mut scale) = match size {
                Size::Auto => (0.0, 3, if cross && parent.align == 3 { 1 } else { 2 }),
                Size::Px(v) => (v, 1, 0),
                Size::Percent(v) => (v, 2, 0),
            };
            if main && independent_factors {
                // Keep the authored main dimension distinct from the flex basis.
                // The checked runtime installs the independent factor policy.
                let (basis, basis_units) = match self.basis {
                    Size::Auto => (0.0, 3),
                    Size::Px(value) => (value, 1),
                    Size::Percent(value) => (value, 2),
                };
                style.set("flexBasis", Value::Float(basis))?;
                style.set("flexBasisUnitsValue", Value::Uint(basis_units))?;
                let minimum = if parent_row { 0 } else { 2 };
                if self.limits[minimum].is_none() {
                    let key = if parent_row { "minWidth" } else { "minHeight" };
                    style.set(key, Value::Float(0.0))?;
                    style.set(&format!("{key}UnitsValue"), Value::Uint(1))?;
                }
            } else if main && self.grow > 0.0 {
                scale = 1;
                // Fill exposes one shared grow/shrink weight. Resolve auto
                // basis through the authored main size before Fill clears it.
                let basis = if self.basis == Size::Auto {
                    size
                } else {
                    self.basis
                };
                let (basis, units) = match basis {
                    Size::Auto => (0.0, 3),
                    Size::Px(v) => (v, 1),
                    Size::Percent(v) => (v, 2),
                };
                style.set("flexBasis", Value::Float(basis))?;
                style.set("flexBasisUnitsValue", Value::Uint(units))?;
                layout.set(
                    if parent_row {
                        "fractionalWidth"
                    } else {
                        "fractionalHeight"
                    },
                    Value::Float(self.grow),
                )?;
                let minimum = if parent_row { 0 } else { 2 };
                if self.limits[minimum].is_none() {
                    let key = if parent_row { "minWidth" } else { "minHeight" };
                    style.set(key, Value::Float(0.0))?;
                    style.set(&format!("{key}UnitsValue"), Value::Uint(1))?;
                }
            }
            layout.set(name, Value::Float(value))?;
            style.set(&format!("{name}UnitsValue"), Value::Uint(units))?;
            style.set(
                if name == "width" {
                    "layoutWidthScaleType"
                } else {
                    "layoutHeightScaleType"
                },
                Value::Uint(scale),
            )?;
        }
        style.set(
            "flexDirectionValue",
            Value::Uint((if self.row { 2 } else { 0 }) + u32::from(self.reverse)),
        )?;
        style.set("flexWrapValue", Value::Uint(self.wrap.rive_value()))?;
        style.set("displayValue", Value::Uint(u32::from(self.hidden)))?;
        let align = self.align.min(2);
        let align = if self.align == 3 { 0 } else { align };
        // Distributed values use an explicit host policy, not the Rive enum.
        let justify = if self.justify >= 4 { 0 } else { self.justify };
        let alignment = if justify == 3 {
            9 + align
        } else if self.row {
            justify + 3 * align
        } else {
            3 * justify + align
        };
        style.set("layoutAlignmentType", Value::Uint(alignment))?;
        for (side, border) in ["Top", "Right", "Bottom", "Left"].into_iter().zip(self.borders) {
            let border_width = border.used_width(self.font_size, "css.border")?;
            if border_width > 0.0 {
                style.set(&format!("border{side}"), Value::Float(border_width))?;
                style.set(&format!("border{side}UnitsValue"), Value::Uint(1))?;
            }
        }
        // Circular lengths retain the established wire representation. Other
        // pairs are installed from the versioned per-axis runtime requirement.
        let mut circular = [0.; 4];
        for (index, pair) in self.radii.iter().enumerate() {
            match pair {
                [crate::corner_radii::RadiusValue::Pixels(x), crate::corner_radii::RadiusValue::Pixels(y)] if x == y => circular[index] = *x,
                _ => { circular = [0.;4]; break; },
            }
        }
        for (corner, index) in [("TL", 0), ("TR", 1), ("BL", 3), ("BR", 2)] {
            style.set(&format!("cornerRadius{corner}"), Value::Float(circular[index]))?;
        }
        if circular.iter().any(|radius| *radius != circular[0]) {
            style.set("linkCornerRadius", Value::Bool(false))?;
        }
        if self.position.is_positioned() {
            style.set("positionTypeValue", Value::Uint(if self.position == crate::position::PositionMode::Absolute { 2 } else { 1 }))?;
            for (side, value) in ["Top", "Right", "Bottom", "Left"].into_iter().zip(self.insets) {
                value.emit(style, side)?;
            }
        }
        for (side, value) in ["Top", "Right", "Bottom", "Left"]
            .into_iter()
            .zip(self.padding)
        {
            let (value, unit) = match value {
                Spacing::Px(v) => (v, 1),
                Spacing::Percent(v) => (v, 2),
            };
            style.set(&format!("padding{side}"), Value::Float(value))?;
            style.set(&format!("padding{side}UnitsValue"), Value::Uint(unit))?;
        }
        for (side, value) in ["Top", "Right", "Bottom", "Left"]
            .into_iter()
            .zip(self.margin)
        {
            let (value, unit) = match value {
                Margin::Auto => (0.0, 3),
                Margin::Px(value) => (value, 1),
                Margin::Percent(value) => (value, 2),
            };
            style.set(&format!("margin{side}"), Value::Float(value))?;
            style.set(&format!("margin{side}UnitsValue"), Value::Uint(unit))?;
        }
        for (name, value) in ["minWidth", "maxWidth", "minHeight", "maxHeight"]
            .into_iter()
            .zip(self.limits)
        {
            if let Some(value) = value {
                let (v, units) = match value {
                    Size::Px(v) => (v, 1),
                    Size::Percent(v) => (v, 2),
                    Size::Auto => continue,
                };
                style.set(name, Value::Float(v))?;
                style.set(&format!("{name}UnitsValue"), Value::Uint(units))?;
            }
        }
        for (axis, value) in ["Vertical", "Horizontal"].into_iter().zip(self.gap) {
            style.set(&format!("gap{axis}"), Value::Float(value))?;
            style.set(&format!("gap{axis}UnitsValue"), Value::Uint(1))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod corner_radius_tests {
    use super::*;

    fn declaration(value: &str) -> Declaration {
        Declaration { name: "border-radius".into(), value: value.into(), important: false, source: "test.radius".into() }
    }

    #[test]
    fn elliptical_style_preserves_percentages_and_inherits_selected_pair() {
        use crate::corner_radii::RadiusValue::{Pixels, Percent};
        let parent = Style::compute(&[declaration("10% 20% / 8px 16px")], &Style::default()).unwrap();
        let mut selected = declaration("inherit");
        selected.name = "border-top-right-radius".into();
        let style = Style::compute(&[declaration("4px / 12px"), selected], &parent).unwrap();
        assert_eq!(style.radii, [[Pixels(4.),Pixels(12.)],[Percent(20.),Pixels(16.)],
            [Pixels(4.),Pixels(12.)],[Pixels(4.),Pixels(12.)]]);
        let mut font = declaration("20px");
        font.name = "font-size".into();
        let style = Style::compute(&[declaration("1em / 50%"), font], &Style::default()).unwrap();
        assert_eq!(style.radii, [[Pixels(20.),Percent(50.)];4]);
    }

    #[test]
    fn shorthand_expands_in_physical_corner_order() {
        for (value, expected) in [
            ("4px", [4., 4., 4., 4.]),
            ("4px 8px", [4., 8., 4., 8.]),
            ("4px 8px 12px", [4., 8., 12., 8.]),
            ("4px 8px 12px 16px", [4., 8., 12., 16.]),
        ] {
            let style = Style::compute(&[declaration(value)], &Style::default()).unwrap();
            assert_eq!(style.radii, expected.map(|r| [crate::corner_radii::RadiusValue::Pixels(r); 2]));
        }
    }

    #[test]
    fn shorthand_global_resets_and_relative_units() {
        let parent = Style::compute(&[declaration("4px 8px 12px 16px")], &Style::default()).unwrap();
        assert_eq!(Style::compute(&[declaration("inherit")], &parent).unwrap().radii, parent.radii);
        for value in ["initial", "unset"] {
            assert_eq!(Style::compute(&[declaration(value)], &parent).unwrap().radii, [[crate::corner_radii::RadiusValue::Pixels(0.); 2]; 4]);
        }
        let style = Style::compute(&[declaration("1em 2rem")], &Style::default()).unwrap();
        assert_eq!(style.radii, [16., 32., 16., 32.].map(|r| [crate::corner_radii::RadiusValue::Pixels(r); 2]));
        for value in ["-1px", "1px 2px 3px 4px 5px"] {
            assert!(Style::compute(&[declaration(value)], &parent).is_err(), "{value}");
        }
    }
}

#[cfg(test)]
mod opacity_style_tests {
    use super::*;

    fn declarations(values: &[(&str, &str)]) -> Vec<Declaration> {
        values.iter().map(|(name, value)| Declaration {
            name: (*name).into(), value: (*value).into(), important: false,
            source: "test.opacity".into(),
        }).collect()
    }

    #[test]
    fn opacity_is_not_inherited_except_explicit_inherit() {
        let parent = Style::compute(&declarations(&[("opacity", "25%")]), &Style::default()).unwrap();
        assert_eq!(parent.opacity, 0.25);
        assert_eq!(Style::compute(&[], &parent).unwrap().opacity, 1.0);
        for (value, expected) in [("inherit", 0.25), ("initial", 1.0), ("unset", 1.0)] {
            let child = Style::compute(&declarations(&[("opacity", ".5"), ("opacity", value)]), &parent).unwrap();
            assert_eq!(child.opacity, expected, "{value}");
        }
    }

    #[test]
    fn opacity_substitution_clamps_and_invalidates_at_computed_value_time() {
        let parent = Style::compute(&declarations(&[("--alpha", "25%"), ("opacity", ".5")]), &Style::default()).unwrap();
        for (value, expected) in [("var(--alpha)", 0.25), ("var(--missing, 150%)", 1.0),
            ("var(--missing, -2)", 0.0), ("var(--missing)", 1.0),
            ("var(--missing, 2px)", 1.0), ("var(--missing, .2 .3)", 1.0),
            ("var(--missing, inherit)", 0.5)] {
            let child = Style::compute(&declarations(&[("opacity", ".1"), ("opacity", value)]), &parent).unwrap();
            assert_eq!(child.opacity, expected, "{value}");
        }
    }

    #[test]
    fn opacity_math_retains_profile_diagnostic_after_substitution() {
        for value in ["calc(.5)", "var(--missing, calc(.5))"] {
            let result = Style::compute(&declarations(&[("opacity", value)]), &Style::default());
            assert!(matches!(result, Err(error) if matches!(error.code.as_str(), "unsupported-opacity" | "unsupported-css-value")), "{value}");
        }
    }
}

#[cfg(test)]
mod gradient_style_tests {
    use super::*;
    fn compute(values: &[(&str, &str)], parent: &Style) -> Style {
        let declarations = values.iter().map(|(name,value)| Declaration {
            name: (*name).into(), value: (*value).into(), important: false, source: "test.gradient".into(),
        }).collect::<Vec<_>>();
        Style::compute(&declarations, parent).unwrap()
    }
    #[test]
    fn gradient_and_solid_background_reset_independently() {
        let parent = Style::default();
        let s = compute(&[("background", "linear-gradient(red,blue)"),("background-color", "lime")], &parent);
        assert!(s.gradient.is_some()); assert_eq!(s.background(), 0xff00ff00);
        for value in ["none","red","initial","unset"] {
            let s = compute(&[("background", "linear-gradient(red,blue)"),("background",value)], &parent);
            assert!(s.gradient.is_none(), "{value}");
        }
        let s = compute(&[("background-color","lime"),("background-image","linear-gradient(red,blue)"),("background-image","none")], &parent);
        assert!(s.gradient.is_none()); assert_eq!(s.background(), 0xff00ff00);
    }
    #[test]
    fn computed_relative_stops_survive_explicit_inheritance() {
        use crate::gradient::{Position,Color};
        let parent = compute(&[("font-size","10px"),("background-image","linear-gradient(currentColor 2em,blue 2rem)"),("font-size","20px")], &Style::default());
        let gradient = parent.gradient.as_ref().unwrap();
        assert_eq!(gradient.stops[0].position, Some(Position::Pixels(40.)));
        assert_eq!(gradient.stops[1].position, Some(Position::Pixels(32.)));
        assert_eq!(gradient.stops[0].color, Color::CurrentColor);
        assert!(compute(&[], &parent).gradient.is_none());
        let child = compute(&[("font-size","30px"),("background-image","inherit")], &parent);
        assert_eq!(child.gradient, parent.gradient);
        assert!(compute(&[("background-image","unset")], &parent).gradient.is_none());
    }
    #[test]
    fn gradient_variable_fallback_is_retained() {
        let s = compute(&[("background-image","var(--missing, linear-gradient(red,blue))")], &Style::default());
        assert!(s.gradient.is_some());
        for value in ["var(--missing, red)", "var(--missing, 20px)", "var(--missing, none none)"] {
            assert!(compute(&[("background-image", value)], &Style::default()).gradient.is_none());
        }
    }
    #[test]
    fn public_emission_requires_gradient_capability() {
        let input = crate::CompileInput {html:"<div id=\"a\"></div>".into(),css:"#a{width:100px;height:100px;background:linear-gradient(red,blue)}".into(),width:390.,height:320.,..Default::default()};
        let output = crate::compile(&input).unwrap();
        assert_eq!(output.runtime_requirements.version, 26);
        assert!(output.runtime_requirements.capabilities.contains(&crate::RuntimeCapability::LayoutCssLinearGradientV1));
    }
}
