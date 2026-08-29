use std::ops::{Index, IndexMut};
use taffy::geometry::MinMax;
use taffy::prelude::{
    AlignContent, AlignItems, AlignSelf, Dimension, Display, FlexDirection, FlexWrap,
    GridPlacement, GridTemplateComponent, LengthPercentage, LengthPercentageAuto, Line,
    MaxTrackSizingFunction, MinTrackSizingFunction, Position, Rect, Size, Style, TaffyGridLine,
    TrackSizingFunction,
};
use taffy::style::Direction;

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
#[derive(Clone, Copy, Debug, Default)]
pub struct YGValue {
    pub value: f32,
    pub unit: YGUnit,
}
impl YGValue {
    pub fn new(value: f32, unit: YGUnit) -> Self {
        Self { value, unit }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum YGDimension {
    Width,
    Height,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum YGEdge {
    Left,
    Top,
    Right,
    Bottom,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum YGGutter {
    Column,
    Row,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct YGSize {
    pub width: f32,
    pub height: f32,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum YGMeasureMode {
    #[default]
    Undefined,
    Exactly,
    AtMost,
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
#[derive(Clone, Copy, Debug, Default)]
pub struct YGFloatOptional(pub Option<f32>);
impl YGFloatOptional {
    pub fn new(v: f32) -> Self {
        Self((!v.is_nan()).then_some(v))
    }
}

#[derive(Clone, Debug)]
pub enum YGStyleSizeLength {
    Points(f32),
    Percent(f32),
    Fr(f32),
    Auto,
}
impl YGStyleSizeLength {
    pub fn points(v: f32) -> Self {
        Self::Points(v)
    }
    pub fn percent(v: f32) -> Self {
        Self::Percent(v)
    }
    pub fn stretch(v: f32) -> Self {
        Self::Fr(v)
    }
    pub fn auto() -> Self {
        Self::Auto
    }
}
#[derive(Clone, Debug)]
pub struct YGGridTrackSize {
    min: YGStyleSizeLength,
    max: YGStyleSizeLength,
}
impl YGGridTrackSize {
    pub fn minmax(min: YGStyleSizeLength, max: YGStyleSizeLength) -> Self {
        Self { min, max }
    }
    pub fn length(v: f32) -> Self {
        Self::minmax(YGStyleSizeLength::Points(v), YGStyleSizeLength::Points(v))
    }
    pub fn percent(v: f32) -> Self {
        Self::minmax(YGStyleSizeLength::Percent(v), YGStyleSizeLength::Percent(v))
    }
    pub fn fr(v: f32) -> Self {
        Self::minmax(YGStyleSizeLength::Auto, YGStyleSizeLength::Fr(v))
    }
    pub fn auto() -> Self {
        Self::minmax(YGStyleSizeLength::Auto, YGStyleSizeLength::Auto)
    }
    fn taffy(&self) -> TrackSizingFunction {
        MinMax {
            min: min_track(&self.min),
            max: max_track(&self.max),
        }
    }
}
pub type GridTrackList = Vec<YGGridTrackSize>;
#[derive(Clone, Debug, Default)]
pub enum YGGridLine {
    #[default]
    Auto,
    Line(i16),
    Span(u16),
}
impl YGGridLine {
    pub fn auto() -> Self {
        Self::Auto
    }
    pub fn from_integer(v: i32) -> Self {
        Self::Line(v as i16)
    }
    pub fn span(v: i32) -> Self {
        Self::Span(v.max(0) as u16)
    }
    fn taffy(&self) -> GridPlacement {
        match *self {
            Self::Auto => GridPlacement::Auto,
            Self::Line(v) => GridPlacement::from_line_index(v),
            Self::Span(v) => GridPlacement::Span(v),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct YGDimensions([YGValue; 2]);
#[derive(Clone, Debug, Default)]
pub struct YGEdges([YGValue; 4]);
#[derive(Clone, Debug, Default)]
pub struct YGGaps([YGValue; 2]);

#[derive(Clone, Debug)]
pub struct YGStyle {
    pub taffy: Style,
    dimensions: YGDimensions,
    min_dimensions: YGDimensions,
    max_dimensions: YGDimensions,
    margin: YGEdges,
    position: YGEdges,
    padding: YGEdges,
    border: YGEdges,
    gap: YGGaps,
    grid_template_columns: GridTrackList,
    grid_template_rows: GridTrackList,
    grid_auto_columns: GridTrackList,
    grid_auto_rows: GridTrackList,
    grid_column: Line<YGGridLine>,
    grid_row: Line<YGGridLine>,
    pub stack: bool,
}
impl Default for YGStyle {
    fn default() -> Self {
        Self {
            // Yoga's non-web default is Column. A styleless LayoutComponent
            // never runs its style appliers, so this initial value is live.
            // Authored LayoutComponentStyle still supplies its own direction.
            taffy: Style {
                flex_direction: FlexDirection::Column,
                ..Style::default()
            },
            dimensions: YGDimensions::default(),
            min_dimensions: YGDimensions::default(),
            max_dimensions: YGDimensions::default(),
            margin: YGEdges::default(),
            position: YGEdges::default(),
            padding: YGEdges::default(),
            border: YGEdges::default(),
            gap: YGGaps::default(),
            grid_template_columns: Vec::new(),
            grid_template_rows: Vec::new(),
            grid_auto_columns: Vec::new(),
            grid_auto_rows: Vec::new(),
            grid_column: Line {
                start: YGGridLine::Auto,
                end: YGGridLine::Auto,
            },
            grid_row: Line {
                start: YGGridLine::Auto,
                end: YGGridLine::Auto,
            },
            stack: false,
        }
    }
}
impl YGStyle {
    pub fn dimensions_mut(&mut self) -> &mut YGDimensions {
        &mut self.dimensions
    }
    pub fn min_dimensions_mut(&mut self) -> &mut YGDimensions {
        &mut self.min_dimensions
    }
    pub fn max_dimensions_mut(&mut self) -> &mut YGDimensions {
        &mut self.max_dimensions
    }
    pub fn margin_mut(&mut self) -> &mut YGEdges {
        &mut self.margin
    }
    pub fn position_mut(&mut self) -> &mut YGEdges {
        &mut self.position
    }
    pub fn padding_mut(&mut self) -> &mut YGEdges {
        &mut self.padding
    }
    pub fn border_mut(&mut self) -> &mut YGEdges {
        &mut self.border
    }
    pub fn gap_mut(&mut self) -> &mut YGGaps {
        &mut self.gap
    }
    pub fn set_display(&mut self, v: YGDisplay) {
        self.taffy.display = match v {
            YGDisplay::None => Display::None,
            YGDisplay::Grid => Display::Grid,
            YGDisplay::Flex => Display::Flex,
        }
    }
    pub fn set_align_self(&mut self, v: YGAlign) {
        self.taffy.align_self = align_self(v)
    }
    pub fn set_align_items(&mut self, v: YGAlign) {
        self.taffy.align_items = align_items(v)
    }
    pub fn set_align_content(&mut self, v: YGAlign) {
        self.taffy.align_content = align_content(v)
    }
    pub fn set_justify_items(&mut self, v: YGJustify) {
        self.taffy.justify_items = justify_items(v)
    }
    pub fn set_justify_self(&mut self, v: YGJustify) {
        self.taffy.justify_self = justify_self(v)
    }
    pub fn set_justify_content(&mut self, v: YGJustify) {
        self.taffy.justify_content = justify_content(v)
    }
    pub fn set_flex_direction(&mut self, v: YGFlexDirection) {
        self.taffy.flex_direction = match v {
            YGFlexDirection::Column => FlexDirection::Column,
            YGFlexDirection::ColumnReverse => FlexDirection::ColumnReverse,
            YGFlexDirection::Row => FlexDirection::Row,
            YGFlexDirection::RowReverse => FlexDirection::RowReverse,
        }
    }
    pub fn set_flex_wrap(&mut self, v: YGWrap) {
        self.taffy.flex_wrap = match v {
            YGWrap::NoWrap => FlexWrap::NoWrap,
            YGWrap::Wrap => FlexWrap::Wrap,
            YGWrap::WrapReverse => FlexWrap::WrapReverse,
        }
    }
    pub fn set_direction(&mut self, v: YGDirection) {
        self.taffy.direction = match v {
            YGDirection::Rtl => Direction::Rtl,
            _ => Direction::Ltr,
        }
    }
    pub fn set_position_type(&mut self, v: YGPositionType) {
        // The pinned Yoga treats Static like Relative, including insets
        // (YGNode::setPosition explicitly leaves static inset handling TODO).
        self.taffy.position = if v == YGPositionType::Absolute {
            Position::Absolute
        } else {
            Position::Relative
        }
    }
    pub fn set_aspect_ratio(&mut self, v: YGFloatOptional) {
        self.taffy.aspect_ratio = v.0
    }
    pub fn set_flex_grow(&mut self, v: YGFloatOptional) {
        self.taffy.flex_grow = v.0.unwrap_or(0.0)
    }
    pub fn set_flex_shrink(&mut self, v: YGFloatOptional) {
        self.taffy.flex_shrink = v.0.unwrap_or(0.0)
    }
    pub fn set_flex_basis(&mut self, v: YGValue) {
        self.taffy.flex_basis = dimension(v)
    }
    pub fn set_grid_template_columns(&mut self, v: GridTrackList) {
        self.grid_template_columns = v
    }
    pub fn set_grid_template_rows(&mut self, v: GridTrackList) {
        self.grid_template_rows = v
    }
    pub fn set_grid_auto_columns(&mut self, v: GridTrackList) {
        self.grid_auto_columns = v
    }
    pub fn set_grid_auto_rows(&mut self, v: GridTrackList) {
        self.grid_auto_rows = v
    }
    pub fn set_grid_column_start(&mut self, v: YGGridLine) {
        self.grid_column.start = v
    }
    pub fn set_grid_column_end(&mut self, v: YGGridLine) {
        self.grid_column.end = v
    }
    pub fn set_grid_row_start(&mut self, v: YGGridLine) {
        self.grid_row.start = v
    }
    pub fn set_grid_row_end(&mut self, v: YGGridLine) {
        self.grid_row.end = v
    }
    pub fn is_grid(&self) -> bool {
        self.taffy.display == Display::Grid
    }
    pub fn is_stack(&self) -> bool {
        self.stack
    }
    pub(crate) fn taffy_calculation_root_style(
        &self,
        owner_width: f32,
        owner_height: f32,
    ) -> Style {
        fn resolve(value: YGValue, owner: f32) -> Option<f32> {
            let value = match value.unit {
                YGUnit::Point => value.value,
                YGUnit::Percent => value.value * owner / 100.0,
                YGUnit::Auto | YGUnit::Undefined => return None,
            };
            (!value.is_nan()).then_some(value)
        }

        let mut style = self.taffy_style();
        for (index, owner, edges) in [(0, owner_width, [0, 2]), (1, owner_height, [1, 3])] {
            let dimension = self.dimensions.0[index];
            let maximum = self.max_dimensions.0[index];
            let minimum = self.min_dimensions.0[index];
            let resolved_dimension = if maximum.unit != YGUnit::Undefined
                && maximum.unit == minimum.unit
                && maximum.value == minimum.value
            {
                maximum
            } else {
                dimension
            };
            let style_defined = match resolved_dimension.unit {
                YGUnit::Auto | YGUnit::Undefined => false,
                YGUnit::Point => !(resolved_dimension.value < 0.0),
                YGUnit::Percent => !(resolved_dimension.value < 0.0 || owner.is_nan()),
            };
            // YGNodeCalculateLayout uses Exactly for a defined owner size when
            // the root has neither a style dimension nor a maximum. Taffy's
            // flex root otherwise treats this only as available space. Adapt
            // the transient solve node, never the authored Rive dimensions.
            if !style_defined && resolve(maximum, owner).is_none() && !owner.is_nan() {
                let margin = edges
                    .into_iter()
                    .map(|edge| resolve(self.margin.0[edge], owner_width).unwrap_or(0.0))
                    .sum::<f32>();
                let exact = Dimension::length((owner - margin).max(0.0));
                if index == 0 {
                    style.size.width = exact;
                } else {
                    style.size.height = exact;
                }
            }
        }
        style
    }

    pub fn taffy_style(&self) -> Style {
        let mut s = self.taffy.clone();
        s.size = Size {
            width: dimension(self.dimensions.0[0]),
            height: dimension(self.dimensions.0[1]),
        };
        s.min_size = Size {
            width: dimension(self.min_dimensions.0[0]),
            height: dimension(self.min_dimensions.0[1]),
        };
        s.max_size = Size {
            width: dimension(self.max_dimensions.0[0]),
            height: dimension(self.max_dimensions.0[1]),
        };
        // Yoga resolves an unspecified margin to zero; only an explicitly
        // authored Auto margin participates in free-space distribution.
        s.margin = rect_auto(self.margin.0.map(|value| {
            if value.unit == YGUnit::Undefined {
                YGValue::new(0.0, YGUnit::Point)
            } else {
                value
            }
        }));
        s.inset = rect_auto(self.position.0);
        s.padding = rect(self.padding.0);
        s.border = rect(self.border.0);
        s.gap = Size {
            width: length(self.gap.0[0]),
            height: length(self.gap.0[1]),
        };
        s.grid_template_columns = self
            .grid_template_columns
            .iter()
            .map(|v| GridTemplateComponent::Single(v.taffy()))
            .collect();
        s.grid_template_rows = self
            .grid_template_rows
            .iter()
            .map(|v| GridTemplateComponent::Single(v.taffy()))
            .collect();
        s.grid_auto_columns = self
            .grid_auto_columns
            .iter()
            .map(YGGridTrackSize::taffy)
            .collect();
        s.grid_auto_rows = self
            .grid_auto_rows
            .iter()
            .map(YGGridTrackSize::taffy)
            .collect();
        s.grid_column = Line {
            start: self.grid_column.start.taffy(),
            end: self.grid_column.end.taffy(),
        };
        s.grid_row = Line {
            start: self.grid_row.start.taffy(),
            end: self.grid_row.end.taffy(),
        };
        s
    }
}
impl Index<YGDimension> for YGDimensions {
    type Output = YGValue;
    fn index(&self, i: YGDimension) -> &Self::Output {
        &self.0[match i {
            YGDimension::Width => 0,
            YGDimension::Height => 1,
        }]
    }
}
impl IndexMut<YGDimension> for YGDimensions {
    fn index_mut(&mut self, i: YGDimension) -> &mut Self::Output {
        &mut self.0[match i {
            YGDimension::Width => 0,
            YGDimension::Height => 1,
        }]
    }
}
impl Index<YGEdge> for YGEdges {
    type Output = YGValue;
    fn index(&self, i: YGEdge) -> &Self::Output {
        &self.0[match i {
            YGEdge::Left => 0,
            YGEdge::Top => 1,
            YGEdge::Right => 2,
            YGEdge::Bottom => 3,
        }]
    }
}
impl IndexMut<YGEdge> for YGEdges {
    fn index_mut(&mut self, i: YGEdge) -> &mut Self::Output {
        &mut self.0[match i {
            YGEdge::Left => 0,
            YGEdge::Top => 1,
            YGEdge::Right => 2,
            YGEdge::Bottom => 3,
        }]
    }
}
impl Index<YGGutter> for YGGaps {
    type Output = YGValue;
    fn index(&self, i: YGGutter) -> &Self::Output {
        &self.0[match i {
            YGGutter::Column => 0,
            YGGutter::Row => 1,
        }]
    }
}
impl IndexMut<YGGutter> for YGGaps {
    fn index_mut(&mut self, i: YGGutter) -> &mut Self::Output {
        &mut self.0[match i {
            YGGutter::Column => 0,
            YGGutter::Row => 1,
        }]
    }
}
fn dimension(v: YGValue) -> Dimension {
    match v.unit {
        YGUnit::Point => Dimension::length(v.value),
        // The pinned Yoga owner resolves percentages as
        // `value * owner_size * 0.01f`; retain the authored percentage so the
        // vendored Taffy seam can preserve that exact `f32` operation order.
        YGUnit::Percent => Dimension::rive_yoga_percent(v.value),
        YGUnit::Auto => Dimension::auto(),
        YGUnit::Undefined => Dimension::auto(),
    }
}
fn length(v: YGValue) -> LengthPercentage {
    match v.unit {
        YGUnit::Percent => LengthPercentage::percent(v.value / 100.0),
        _ => LengthPercentage::length(if v.value.is_nan() { 0.0 } else { v.value }),
    }
}
fn auto(v: YGValue) -> LengthPercentageAuto {
    match v.unit {
        YGUnit::Percent => LengthPercentageAuto::percent(v.value / 100.0),
        YGUnit::Auto | YGUnit::Undefined => LengthPercentageAuto::auto(),
        YGUnit::Point => LengthPercentageAuto::length(v.value),
    }
}
fn rect(v: [YGValue; 4]) -> Rect<LengthPercentage> {
    Rect {
        left: length(v[0]),
        top: length(v[1]),
        right: length(v[2]),
        bottom: length(v[3]),
    }
}
fn rect_auto(v: [YGValue; 4]) -> Rect<LengthPercentageAuto> {
    Rect {
        left: auto(v[0]),
        top: auto(v[1]),
        right: auto(v[2]),
        bottom: auto(v[3]),
    }
}
fn min_track(v: &YGStyleSizeLength) -> MinTrackSizingFunction {
    match *v {
        YGStyleSizeLength::Points(x) => MinTrackSizingFunction::length(x),
        YGStyleSizeLength::Percent(x) => MinTrackSizingFunction::percent(x / 100.0),
        _ => MinTrackSizingFunction::auto(),
    }
}
fn max_track(v: &YGStyleSizeLength) -> MaxTrackSizingFunction {
    match *v {
        YGStyleSizeLength::Points(x) => MaxTrackSizingFunction::length(x),
        YGStyleSizeLength::Percent(x) => MaxTrackSizingFunction::percent(x / 100.0),
        YGStyleSizeLength::Fr(x) => MaxTrackSizingFunction::fr(x),
        YGStyleSizeLength::Auto => MaxTrackSizingFunction::auto(),
    }
}
fn align_items(v: YGAlign) -> Option<AlignItems> {
    match v {
        YGAlign::Auto => None,
        YGAlign::FlexStart | YGAlign::Start => Some(AlignItems::FLEX_START),
        YGAlign::Center => Some(AlignItems::CENTER),
        YGAlign::FlexEnd | YGAlign::End => Some(AlignItems::FLEX_END),
        YGAlign::Baseline => Some(AlignItems::BASELINE),
        _ => Some(AlignItems::STRETCH),
    }
}
fn align_self(v: YGAlign) -> Option<AlignSelf> {
    align_items(v).map(|v| v)
}
fn align_content(v: YGAlign) -> Option<AlignContent> {
    match v {
        YGAlign::FlexStart | YGAlign::Start => Some(AlignContent::FLEX_START),
        YGAlign::Center => Some(AlignContent::CENTER),
        YGAlign::FlexEnd | YGAlign::End => Some(AlignContent::FLEX_END),
        YGAlign::SpaceBetween => Some(AlignContent::SPACE_BETWEEN),
        YGAlign::SpaceAround => Some(AlignContent::SPACE_AROUND),
        YGAlign::Stretch => Some(AlignContent::STRETCH),
        _ => None,
    }
}
fn justify_items(v: YGJustify) -> Option<AlignItems> {
    match v {
        YGJustify::Auto => None,
        YGJustify::FlexStart | YGJustify::Start => Some(AlignItems::FLEX_START),
        YGJustify::Center => Some(AlignItems::CENTER),
        YGJustify::FlexEnd | YGJustify::End => Some(AlignItems::FLEX_END),
        _ => Some(AlignItems::STRETCH),
    }
}
fn justify_self(v: YGJustify) -> Option<AlignSelf> {
    justify_items(v).map(|v| v)
}
fn justify_content(v: YGJustify) -> Option<taffy::prelude::JustifyContent> {
    Some(match v {
        YGJustify::Center => taffy::prelude::JustifyContent::CENTER,
        YGJustify::FlexEnd | YGJustify::End => taffy::prelude::JustifyContent::FLEX_END,
        YGJustify::SpaceBetween => taffy::prelude::JustifyContent::SPACE_BETWEEN,
        YGJustify::SpaceAround => taffy::prelude::JustifyContent::SPACE_AROUND,
        YGJustify::SpaceEvenly => taffy::prelude::JustifyContent::SPACE_EVENLY,
        _ => taffy::prelude::JustifyContent::FLEX_START,
    })
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
    pub parent_is_row: bool,
    pub is_ltr: bool,
    pub has_layout_parent: bool,
}

pub trait LayoutStyleApplier {
    fn apply_base_style(&self, _style: &mut YGStyle, _context: &LayoutSyncContext) {}
    fn apply_container_style(&self, _style: &mut YGStyle, _context: &LayoutSyncContext) {}
    fn apply_item_style(&self, _style: &mut YGStyle, _context: &LayoutSyncContext) {}
}
