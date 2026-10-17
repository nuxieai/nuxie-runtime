//! Rive's authored layout model expressed as CSS for stock Taffy.
//!
//! Layout is a deliberate divergence from upstream Rive, which lays out with
//! its Yoga fork (docs/PORTING.md, "Layout"). The enums below decode Rive's
//! serialized layout values, which keep Yoga's numbering. Style appliers write
//! those values straight into each node's `taffy::Style`; [`rive_default_style`]
//! states every place Yoga's defaults differ from CSS's, so ordinary files keep
//! their authored look under CSS semantics without chasing Yoga's numbers.
use taffy::geometry::MinMax;
use taffy::prelude::{
    AlignContent, AlignItems, AlignSelf, Dimension, Display, FlexDirection, FlexWrap,
    GridPlacement, GridTemplateComponent, JustifyContent, LengthPercentage, LengthPercentageAuto,
    MaxTrackSizingFunction, MinTrackSizingFunction, Position, Rect, Size, Style, TaffyGridLine,
    TrackSizingFunction,
};
use taffy::style::{BoxSizing, Direction};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum YGUnit {
    #[default]
    Undefined,
    Point,
    Percent,
    Auto,
}
impl From<u8> for YGUnit {
    fn from(value: u8) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<u32> for YGUnit {
    fn from(v: u32) -> Self {
        match v {
            1 => Self::Point,
            2 => Self::Percent,
            3 => Self::Auto,
            _ => Self::Undefined,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum YGDisplay {
    #[default]
    Flex,
    None,
    Grid,
}
impl From<u8> for YGDisplay {
    fn from(value: u8) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<u32> for YGDisplay {
    fn from(v: u32) -> Self {
        match v {
            1 => Self::None,
            2 => Self::Grid,
            _ => Self::Flex,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum YGAlign {
    Auto,
    FlexStart,
    Center,
    FlexEnd,
    #[default]
    Stretch,
    Baseline,
    SpaceBetween,
    SpaceAround,
    Start,
    End,
}
impl From<u8> for YGAlign {
    fn from(value: u8) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<u32> for YGAlign {
    fn from(v: u32) -> Self {
        match v {
            0 => Self::Auto,
            1 => Self::FlexStart,
            2 => Self::Center,
            3 => Self::FlexEnd,
            5 => Self::Baseline,
            6 => Self::SpaceBetween,
            7 => Self::SpaceAround,
            8 => Self::Start,
            9 => Self::End,
            _ => Self::Stretch,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum YGJustify {
    #[default]
    FlexStart,
    Center,
    FlexEnd,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Auto,
    Stretch,
    Start,
    End,
}
impl From<u8> for YGJustify {
    fn from(value: u8) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<u32> for YGJustify {
    fn from(v: u32) -> Self {
        match v {
            1 => Self::Center,
            2 => Self::FlexEnd,
            3 => Self::SpaceBetween,
            4 => Self::SpaceAround,
            5 => Self::SpaceEvenly,
            6 => Self::Auto,
            7 => Self::Stretch,
            8 => Self::Start,
            9 => Self::End,
            _ => Self::FlexStart,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum YGFlexDirection {
    #[default]
    Column,
    ColumnReverse,
    Row,
    RowReverse,
}
impl From<u8> for YGFlexDirection {
    fn from(value: u8) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<u32> for YGFlexDirection {
    fn from(v: u32) -> Self {
        match v {
            0 => Self::Column,
            1 => Self::ColumnReverse,
            3 => Self::RowReverse,
            _ => Self::Row,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum YGDirection {
    #[default]
    Inherit,
    Ltr,
    Rtl,
}
impl From<u8> for YGDirection {
    fn from(value: u8) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<u32> for YGDirection {
    fn from(v: u32) -> Self {
        match v {
            1 => Self::Ltr,
            2 => Self::Rtl,
            _ => Self::Inherit,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum YGWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}
impl From<u8> for YGWrap {
    fn from(value: u8) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<u32> for YGWrap {
    fn from(v: u32) -> Self {
        match v {
            1 => Self::Wrap,
            2 => Self::WrapReverse,
            _ => Self::NoWrap,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum YGOverflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
}
impl From<u8> for YGOverflow {
    fn from(value: u8) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<u32> for YGOverflow {
    fn from(v: u32) -> Self {
        match v {
            1 => Self::Hidden,
            2 => Self::Scroll,
            _ => Self::Visible,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum YGPositionType {
    Static = 0,
    #[default]
    Relative = 1,
    Absolute = 2,
}
impl From<u8> for YGPositionType {
    fn from(value: u8) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<u32> for YGPositionType {
    fn from(v: u32) -> Self {
        match v {
            0 => Self::Static,
            1 => Self::Relative,
            2 => Self::Absolute,
            _ => panic!("invalid YGPositionType value: {v}"),
        }
    }
}

/// A grid track as Rive authors it: a minimum and maximum sizing function.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RiveTrackLength {
    Points(f32),
    Percent(f32),
    Fr(f32),
    Auto,
}

/// The explicit CSS baseline for a Rive layout node. Yoga's defaults differ
/// from CSS's, and Rive files are authored against Yoga's, so each differing
/// property is stated here instead of being left to Taffy's CSS defaults:
///
/// - every node is positioned: an absolute child is placed against its own
///   parent, never hoisted to a further ancestor;
/// - `flex-direction: column`, `align-content: flex-start`,
///   `align-items: stretch` and `justify-content: flex-start`;
/// - border-box sizing with zero margins, padding and borders.
///
/// Flex shrinking and automatic minimum sizes follow CSS, including Fixed
/// items. Taffy resolves `min-size: auto` from the content contribution.
pub fn rive_default_style() -> Style {
    Style {
        display: Display::Flex,
        box_sizing: BoxSizing::BorderBox,
        direction: Direction::Ltr,
        position: Position::Relative,
        inset: Rect::auto(),
        size: Size::auto(),
        min_size: Size::auto(),
        max_size: Size::auto(),
        aspect_ratio: None,
        margin: Rect::zero(),
        padding: Rect::zero(),
        border: Rect::zero(),
        align_items: AlignItems::STRETCH,
        align_self: None,
        justify_items: AlignItems::STRETCH,
        justify_self: None,
        align_content: AlignContent::FLEX_START,
        justify_content: JustifyContent::FLEX_START,
        gap: Size::zero(),
        flex_direction: FlexDirection::Column,
        flex_wrap: FlexWrap::NoWrap,
        flex_grow: 0.0,
        flex_shrink: 1.0,
        flex_basis: Dimension::auto(),
        ..Style::DEFAULT
    }
}

fn defined(value: f32) -> Option<f32> {
    (!value.is_nan()).then_some(value)
}

/// A width or height. Percentages use Taffy's native 0..1 representation.
pub fn rive_dimension(value: f32, unit: YGUnit) -> Dimension {
    match (unit, defined(value)) {
        (YGUnit::Point, Some(value)) => Dimension::length(value),
        (YGUnit::Percent, Some(value)) => Dimension::percent(value / 100.0),
        _ => Dimension::auto(),
    }
}

/// A hugging axis sizes to its content. Rive bounds it by the space its
/// parent offers unless the author marks it unbounded; CSS states exactly
/// that as `fit-content` versus `max-content`.
pub fn rive_hug_dimension(unbounded: bool) -> Dimension {
    if unbounded {
        Dimension::max_content()
    } else {
        Dimension::fit_content()
    }
}

/// A minimum or maximum size.
pub fn rive_size_limit(value: f32, unit: YGUnit) -> LengthPercentageAuto {
    match (unit, defined(value)) {
        (YGUnit::Point, Some(value)) => LengthPercentageAuto::length(value),
        (YGUnit::Percent, Some(value)) => LengthPercentageAuto::percent(value / 100.0),
        _ => LengthPercentageAuto::auto(),
    }
}

/// Padding, border and gaps: an authored number without a percent unit is a
/// length, and an unset value is zero.
pub fn rive_length(value: f32, unit: YGUnit) -> LengthPercentage {
    match (unit, defined(value)) {
        (YGUnit::Percent, Some(value)) => LengthPercentage::percent(value / 100.0),
        (_, Some(value)) => LengthPercentage::length(value),
        (_, None) => LengthPercentage::length(0.0),
    }
}

/// A margin: only an explicit Auto takes part in free-space distribution; an
/// unset margin is zero.
pub fn rive_margin(value: f32, unit: YGUnit) -> LengthPercentageAuto {
    match (unit, defined(value)) {
        (YGUnit::Auto, _) => LengthPercentageAuto::auto(),
        (YGUnit::Percent, Some(value)) => LengthPercentageAuto::percent(value / 100.0),
        (YGUnit::Point, Some(value)) => LengthPercentageAuto::length(value),
        _ => LengthPercentageAuto::length(0.0),
    }
}

/// An inset (position edge). Unset edges are `auto`, not zero.
pub fn rive_inset(value: f32, unit: YGUnit) -> LengthPercentageAuto {
    match (unit, defined(value)) {
        (YGUnit::Percent, Some(value)) => LengthPercentageAuto::percent(value / 100.0),
        (YGUnit::Point, Some(value)) => LengthPercentageAuto::length(value),
        _ => LengthPercentageAuto::auto(),
    }
}

/// Logical start/end edges resolve to physical sides by direction.
pub fn physical_edges<T>(is_ltr: bool, start: T, end: T, top: T, bottom: T) -> Rect<T> {
    if is_ltr {
        Rect {
            left: start,
            right: end,
            top,
            bottom,
        }
    } else {
        Rect {
            left: end,
            right: start,
            top,
            bottom,
        }
    }
}

pub fn rive_display(value: YGDisplay) -> Display {
    match value {
        YGDisplay::None => Display::None,
        YGDisplay::Grid => Display::Grid,
        YGDisplay::Flex => Display::Flex,
    }
}

pub fn rive_flex_direction(value: YGFlexDirection) -> FlexDirection {
    match value {
        YGFlexDirection::Column => FlexDirection::Column,
        YGFlexDirection::ColumnReverse => FlexDirection::ColumnReverse,
        YGFlexDirection::Row => FlexDirection::Row,
        YGFlexDirection::RowReverse => FlexDirection::RowReverse,
    }
}

pub fn rive_flex_wrap(value: YGWrap) -> FlexWrap {
    match value {
        YGWrap::NoWrap => FlexWrap::NoWrap,
        YGWrap::Wrap => FlexWrap::Wrap,
        YGWrap::WrapReverse => FlexWrap::WrapReverse,
    }
}

pub fn rive_direction(is_ltr: bool) -> Direction {
    if is_ltr {
        Direction::Ltr
    } else {
        Direction::Rtl
    }
}

/// Static and relative both stay in flow and both contain absolute children.
pub fn rive_position(value: YGPositionType) -> Position {
    if value == YGPositionType::Absolute {
        Position::Absolute
    } else {
        Position::Relative
    }
}

/// `align-items`; Auto keeps Yoga's stretch default.
pub fn rive_align_items(value: YGAlign) -> AlignItems {
    rive_align_self(value).unwrap_or(AlignItems::STRETCH)
}

/// `align-self`; Auto defers to the container's `align-items`.
pub fn rive_align_self(value: YGAlign) -> Option<AlignSelf> {
    match value {
        YGAlign::Auto => None,
        YGAlign::FlexStart | YGAlign::Start => Some(AlignItems::FLEX_START),
        YGAlign::Center => Some(AlignItems::CENTER),
        YGAlign::FlexEnd | YGAlign::End => Some(AlignItems::FLEX_END),
        YGAlign::Baseline => Some(AlignItems::BASELINE),
        _ => Some(AlignItems::STRETCH),
    }
}

pub fn rive_align_content(value: YGAlign) -> AlignContent {
    match value {
        YGAlign::Center => AlignContent::CENTER,
        YGAlign::FlexEnd | YGAlign::End => AlignContent::FLEX_END,
        YGAlign::SpaceBetween => AlignContent::SPACE_BETWEEN,
        YGAlign::SpaceAround => AlignContent::SPACE_AROUND,
        YGAlign::Stretch => AlignContent::STRETCH,
        _ => AlignContent::FLEX_START,
    }
}

/// Grid `justify-items`; Auto is stretch, as for `align-items`.
pub fn rive_justify_items(value: YGJustify) -> AlignItems {
    rive_justify_self(value).unwrap_or(AlignItems::STRETCH)
}

/// Grid `justify-self`; Auto defers to the container's `justify-items`.
pub fn rive_justify_self(value: YGJustify) -> Option<AlignSelf> {
    match value {
        YGJustify::Auto => None,
        YGJustify::FlexStart | YGJustify::Start => Some(AlignItems::FLEX_START),
        YGJustify::Center => Some(AlignItems::CENTER),
        YGJustify::FlexEnd | YGJustify::End => Some(AlignItems::FLEX_END),
        _ => Some(AlignItems::STRETCH),
    }
}

pub fn rive_justify_content(value: YGJustify) -> JustifyContent {
    match value {
        YGJustify::Center => JustifyContent::CENTER,
        YGJustify::FlexEnd | YGJustify::End => JustifyContent::FLEX_END,
        YGJustify::SpaceBetween => JustifyContent::SPACE_BETWEEN,
        YGJustify::SpaceAround => JustifyContent::SPACE_AROUND,
        YGJustify::SpaceEvenly => JustifyContent::SPACE_EVENLY,
        _ => JustifyContent::FLEX_START,
    }
}

/// A grid track. Percentages use Taffy's native 0..1 representation; an fr
/// minimum is not a valid CSS minimum and resolves as `auto`.
pub fn rive_track(min: RiveTrackLength, max: RiveTrackLength) -> TrackSizingFunction {
    MinMax {
        min: match min {
            RiveTrackLength::Points(value) => MinTrackSizingFunction::length(value),
            RiveTrackLength::Percent(value) => MinTrackSizingFunction::percent(value / 100.0),
            RiveTrackLength::Fr(_) | RiveTrackLength::Auto => MinTrackSizingFunction::auto(),
        },
        max: match max {
            RiveTrackLength::Points(value) => MaxTrackSizingFunction::length(value),
            RiveTrackLength::Percent(value) => MaxTrackSizingFunction::percent(value / 100.0),
            RiveTrackLength::Fr(value) => MaxTrackSizingFunction::fr(value),
            RiveTrackLength::Auto => MaxTrackSizingFunction::auto(),
        },
    }
}

pub fn rive_template(tracks: &[TrackSizingFunction]) -> Vec<GridTemplateComponent<String>> {
    tracks
        .iter()
        .map(|track| GridTemplateComponent::Single(*track))
        .collect()
}

/// Rive's authored grid line (one-based, negative from the end). Stock Taffy
/// addresses lines as `i16`; lines past that range saturate.
pub fn rive_grid_line(line: i32) -> GridPlacement {
    GridPlacement::from_line_index(line.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16)
}

pub fn rive_grid_span(span: u32) -> GridPlacement {
    GridPlacement::Span(span.min(u32::from(u16::MAX)) as u16)
}

pub(crate) struct LayoutParentStyleSnapshot {
    pub owner: crate::mechanical_port::source::core::CoreHandle,
    pub is_grid: bool,
    pub is_stack: bool,
    pub justify_items: u32,
    pub is_row: bool,
    pub is_ltr: bool,
}

#[derive(Default, Clone, Copy)]
pub struct LayoutSyncContext {
    pub parent_is_grid: bool,
    pub parent_is_stack: bool,
    pub container_justify_items: u32,
    pub inline_hugs: bool,
    /// Resolved by the owner, including a hosted artboard's sizing override.
    pub width_fills: bool,
    pub parent_is_row: bool,
    pub is_ltr: bool,
    pub has_layout_parent: bool,
    /// Which properties a `LayoutCssStyle` record contributes here.
    pub css_role: crate::css_layout::CssRole,
}

pub trait LayoutStyleApplier {
    fn apply_base_style(&self, _style: &mut Style, _context: &LayoutSyncContext) {}
    fn apply_container_style(&self, _style: &mut Style, _context: &LayoutSyncContext) {}
    fn apply_item_style(&self, _style: &mut Style, _context: &LayoutSyncContext) {}
    /// Explicit grid placement follows item-style resets, independent of file order.
    fn apply_placement_style(&self, _style: &mut Style, _context: &LayoutSyncContext) {}
    /// A `LayoutCssStyle` record, after every other sweep: in CSS mode its
    /// values replace what the Rive modes translated to (docs/CSS_LAYOUT.md).
    fn apply_css_style(&self, _style: &mut Style, _context: &LayoutSyncContext) {}
}
