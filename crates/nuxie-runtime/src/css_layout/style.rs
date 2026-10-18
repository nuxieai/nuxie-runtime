//! A `LayoutCssStyle` record as a Taffy style (docs/CSS_LAYOUT.md, sections
//! 1.3 and 2.1). Our layer over Rive: Taffy lays the values out with its own
//! CSS algorithms.
use super::keys::{self, Field};
use taffy::prelude::{
    AlignContent, AlignItems, Dimension, Display, FlexDirection, FlexWrap, LengthPercentage,
    LengthPercentageAuto, Position, Rect, Size, Style,
};
use taffy::style::{BoxSizing, Direction, GridAutoFlow, Overflow};

/// Which properties a record contributes, by what it is attached to.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CssRole {
    /// A `LayoutComponent`: every property.
    #[default]
    Box,
    /// An `Artboard`: its own width and height define the root box, so only
    /// container properties apply.
    Artboard,
    /// A `LayoutParticipant`'s host: only item properties apply.
    Participant,
}

/// Read access to one record's values, indexed by property key.
#[derive(Clone, Copy)]
pub struct CssValues<'a> {
    pub(crate) first: u16,
    pub(crate) values: &'a [f32],
}

impl CssValues<'_> {
    pub fn number(&self, key: u16) -> f32 {
        key.checked_sub(self.first)
            .and_then(|index| self.values.get(usize::from(index)))
            .copied()
            .unwrap_or_else(|| keys::default_value(key))
    }
    /// An enum value; a value outside the property's range (only reachable
    /// at run time, since load refuses it) reads as the initial value.
    pub fn choice(&self, key: u16) -> u8 {
        let value = self.number(key);
        match keys::field(key) {
            Some(Field::Enum { max, default }) => {
                if value.is_finite() && value >= 0.0 && value <= f32::from(max) {
                    value.round() as u8
                } else {
                    default
                }
            }
            _ => 0,
        }
    }
    /// A length-percentage stored at `key` (points) and `key + 1` (percent).
    fn parts(&self, key: u16) -> (f32, f32) {
        (self.number(key), self.number(key + 1))
    }
}

/// Why a record cannot load, or None when every value is supported.
pub fn refusal(values: CssValues<'_>, type_key: u16) -> Option<String> {
    let (first, count) = keys::range(type_key)?;
    for key in first..first + count as u16 {
        let value = values.number(key);
        if !value.is_finite() {
            return Some(format!("property {key} is not a finite number"));
        }
        if let Some(Field::Enum { max, .. }) = keys::field(key)
            && (value < 0.0 || value > f32::from(max) || value.fract() != 0.0)
        {
            return Some(format!("property {key} value {value} is out of range"));
        }
    }
    let mixed = |key: u16| {
        let (length, percent) = values.parts(key);
        (length != 0.0 && percent != 0.0).then(|| {
            format!(
                "property {key} mixes a length and a percentage; mixed calc() values are not supported yet"
            )
        })
    };
    let read_when = |kind: u16, read: &[u8]| read.contains(&values.choice(kind));
    match type_key {
        keys::LAYOUT_CSS_STYLE => {
            let mut checks: Vec<u16> = Vec::new();
            for (kind_key, read) in [
                (keys::FLEX_BASIS_KIND, &[1u8, 5][..]),
                (keys::WIDTH_KIND, &[1, 5]),
                (keys::HEIGHT_KIND, &[1, 5]),
                (keys::MIN_WIDTH_KIND, &[1]),
                (keys::MIN_HEIGHT_KIND, &[1]),
                (keys::MAX_WIDTH_KIND, &[1]),
                (keys::MAX_HEIGHT_KIND, &[1]),
            ] {
                if read_when(kind_key, read) {
                    checks.push(kind_key + 1);
                }
            }
            for side in 0..4 {
                if read_when(keys::MARGIN_TOP_KIND + side * 3, &[0]) {
                    checks.push(keys::MARGIN_TOP_KIND + side * 3 + 1);
                }
                if read_when(keys::INSET_TOP_KIND + side * 3, &[1]) {
                    checks.push(keys::INSET_TOP_KIND + side * 3 + 1);
                }
                checks.push(keys::PADDING_TOP + side * 2);
            }
            checks.extend([keys::ROW_GAP, keys::COLUMN_GAP]);
            checks.into_iter().find_map(mixed)
        }
        keys::BOX_GRADIENT => [
            keys::GRADIENT_CENTER_X,
            keys::GRADIENT_CENTER_Y,
            keys::GRADIENT_RADIUS_X,
            keys::GRADIENT_RADIUS_Y,
        ]
        .into_iter()
        .find_map(mixed),
        _ => None,
    }
}

/// Until mixed values arrive (docs/CSS_LAYOUT.md, 1.2), one part is used: a
/// value made mixed at run time resolves as its percentage alone.
fn length_percentage(length: f32, percent: f32) -> LengthPercentage {
    if percent != 0.0 {
        LengthPercentage::percent(percent / 100.0)
    } else {
        LengthPercentage::length(length)
    }
}
fn length_percentage_auto(length: f32, percent: f32) -> LengthPercentageAuto {
    if percent != 0.0 {
        LengthPercentageAuto::percent(percent / 100.0)
    } else {
        LengthPercentageAuto::length(length)
    }
}
fn non_negative((length, percent): (f32, f32)) -> (f32, f32) {
    (length.max(0.0), percent.max(0.0))
}

impl CssValues<'_> {
    fn size(&self, kind_key: u16) -> Dimension {
        let (length, percent) = non_negative(self.parts(kind_key + 1));
        match self.choice(kind_key) {
            1 if percent != 0.0 => Dimension::percent(percent / 100.0),
            1 => Dimension::length(length),
            2 => Dimension::min_content(),
            3 => Dimension::max_content(),
            4 => Dimension::fit_content(),
            5 if percent != 0.0 => Dimension::fit_content_percent(percent / 100.0),
            5 => Dimension::fit_content_px(length),
            // `content` is accepted for flex-basis only.
            6 if kind_key == keys::FLEX_BASIS_KIND => Dimension::content(),
            _ => Dimension::auto(),
        }
    }
    /// A minimum (`auto`) or maximum (`none`), both Taffy's `auto`.
    fn limit(&self, kind_key: u16) -> LengthPercentageAuto {
        if self.choice(kind_key) == 1 {
            let (length, percent) = non_negative(self.parts(kind_key + 1));
            length_percentage_auto(length, percent)
        } else {
            LengthPercentageAuto::auto()
        }
    }
    fn margin(&self, kind_key: u16) -> LengthPercentageAuto {
        if self.choice(kind_key) == 1 {
            LengthPercentageAuto::auto()
        } else {
            let (length, percent) = self.parts(kind_key + 1);
            length_percentage_auto(length, percent)
        }
    }
    fn inset(&self, kind_key: u16) -> LengthPercentageAuto {
        if self.choice(kind_key) == 1 {
            let (length, percent) = self.parts(kind_key + 1);
            length_percentage_auto(length, percent)
        } else {
            LengthPercentageAuto::auto()
        }
    }
    fn padding(&self, key: u16) -> LengthPercentage {
        let (length, percent) = non_negative(self.parts(key));
        length_percentage(length, percent)
    }
    fn sides<T>(&self, first: u16, stride: u16, read: impl Fn(u16) -> T) -> Rect<T> {
        Rect {
            top: read(first),
            right: read(first + stride),
            bottom: read(first + stride * 2),
            left: read(first + stride * 3),
        }
    }

    fn display(&self) -> Display {
        match self.choice(keys::DISPLAY) {
            1 => Display::Flex,
            2 => Display::Grid,
            3 => Display::None,
            4 => Display::FlowRoot,
            _ => Display::Block,
        }
    }
    /// The record's own direction, or None when it inherits.
    pub fn direction(&self) -> Option<bool> {
        match self.choice(keys::DIRECTION) {
            1 => Some(true),
            2 => Some(false),
            _ => None,
        }
    }
    pub fn display_none(&self) -> bool {
        self.choice(keys::DISPLAY) == 3
    }
    /// Computed overflow: when one axis is a scroll container (hidden,
    /// scroll or auto), `visible` on the other computes to `auto` and
    /// `clip` to `hidden`, as CSS Overflow 3 says. `auto` is a scroll
    /// container like `scroll`; scrollbars take no space.
    pub fn overflow(&self) -> taffy::geometry::Point<Overflow> {
        let axis = |key: u16| match self.choice(key) {
            1 => Overflow::Clip,
            2 => Overflow::Hidden,
            3 | 4 => Overflow::Scroll,
            _ => Overflow::Visible,
        };
        let (mut x, mut y) = (axis(keys::OVERFLOW_X), axis(keys::OVERFLOW_Y));
        let computed = |own: Overflow, other: Overflow| {
            if other.is_scroll_container() {
                match own {
                    Overflow::Visible => Overflow::Scroll,
                    Overflow::Clip => Overflow::Hidden,
                    own => own,
                }
            } else {
                own
            }
        };
        (x, y) = (computed(x, y), computed(y, x));
        taffy::geometry::Point { x, y }
    }
}

fn item_alignment(value: u8) -> Option<AlignItems> {
    Some(match value {
        1 => AlignItems::NORMAL,
        2 => AlignItems::START,
        3 => AlignItems::END,
        4 => AlignItems::FLEX_START,
        5 => AlignItems::FLEX_END,
        6 => AlignItems::SELF_START,
        7 => AlignItems::SELF_END,
        8 => AlignItems::CENTER,
        9 => AlignItems::BASELINE,
        10 => AlignItems::STRETCH,
        _ => return None,
    })
}

fn content_alignment(value: u8) -> AlignContent {
    match value {
        1 => AlignContent::START,
        2 => AlignContent::END,
        3 => AlignContent::FLEX_START,
        4 => AlignContent::FLEX_END,
        5 => AlignContent::CENTER,
        6 => AlignContent::STRETCH,
        7 => AlignContent::SPACE_BETWEEN,
        8 => AlignContent::SPACE_EVENLY,
        9 => AlignContent::SPACE_AROUND,
        _ => AlignContent::NORMAL,
    }
}

/// Write the record into `style`. `inherited_ltr` is the direction the node
/// inherits; the role says which properties apply.
pub fn apply(values: CssValues<'_>, style: &mut Style, role: CssRole, inherited_ltr: bool) {
    let ltr = values.direction().unwrap_or(inherited_ltr);
    if role != CssRole::Participant {
        apply_container(values, style, ltr);
    }
    match role {
        CssRole::Box => {
            style.box_sizing = match values.choice(keys::BOX_SIZING) {
                1 => BoxSizing::BorderBox,
                _ => BoxSizing::ContentBox,
            };
            style.display = values.display();
            apply_item(values, style);
        }
        CssRole::Participant => {
            // A participant has no children: the record decides only
            // whether it shows, over its Rive display.
            style.display = if values.display_none() {
                Display::None
            } else {
                Display::Flex
            };
            apply_item(values, style);
        }
        CssRole::Artboard => {
            // The artboard's width and height are its border box.
            style.box_sizing = BoxSizing::BorderBox;
            style.display = values.display();
        }
    }
}

fn apply_container(values: CssValues<'_>, style: &mut Style, ltr: bool) {
    style.direction = if ltr { Direction::Ltr } else { Direction::Rtl };
    style.overflow = values.overflow();
    style.scrollbar_width = 0.0;
    style.flex_direction = match values.choice(keys::FLEX_DIRECTION) {
        1 => FlexDirection::RowReverse,
        2 => FlexDirection::Column,
        3 => FlexDirection::ColumnReverse,
        _ => FlexDirection::Row,
    };
    style.flex_wrap = match values.choice(keys::FLEX_WRAP) {
        1 => FlexWrap::Wrap,
        2 => FlexWrap::WrapReverse,
        _ => FlexWrap::NoWrap,
    };
    style.align_items =
        item_alignment(values.choice(keys::ALIGN_ITEMS)).unwrap_or(AlignItems::NORMAL);
    style.justify_items =
        item_alignment(values.choice(keys::JUSTIFY_ITEMS)).unwrap_or(AlignItems::NORMAL);
    style.align_content = content_alignment(values.choice(keys::ALIGN_CONTENT));
    style.justify_content = content_alignment(values.choice(keys::JUSTIFY_CONTENT));
    style.grid_auto_flow = match values.choice(keys::GRID_AUTO_FLOW) {
        1 => GridAutoFlow::Column,
        2 => GridAutoFlow::RowDense,
        3 => GridAutoFlow::ColumnDense,
        _ => GridAutoFlow::Row,
    };
    style.padding = values.sides(keys::PADDING_TOP, 2, |key| values.padding(key));
    style.border = values.sides(keys::BORDER_TOP, 1, |key| {
        LengthPercentage::length(values.number(key).max(0.0))
    });
    let gap = |key: u16| {
        let (length, percent) = non_negative(values.parts(key));
        length_percentage(length, percent)
    };
    style.gap = Size {
        width: gap(keys::COLUMN_GAP),
        height: gap(keys::ROW_GAP),
    };
}

fn apply_item(values: CssValues<'_>, style: &mut Style) {
    style.position = match values.choice(keys::POSITION) {
        1 => Position::Relative,
        2 => Position::Absolute,
        _ => Position::Static,
    };
    style.inset = values.sides(keys::INSET_TOP_KIND, 3, |key| values.inset(key));
    style.margin = values.sides(keys::MARGIN_TOP_KIND, 3, |key| values.margin(key));
    style.size = Size {
        width: values.size(keys::WIDTH_KIND),
        height: values.size(keys::HEIGHT_KIND),
    };
    style.min_size = Size {
        width: values.limit(keys::MIN_WIDTH_KIND),
        height: values.limit(keys::MIN_HEIGHT_KIND),
    };
    style.max_size = Size {
        width: values.limit(keys::MAX_WIDTH_KIND),
        height: values.limit(keys::MAX_HEIGHT_KIND),
    };
    let ratio = values.number(keys::ASPECT_RATIO);
    style.aspect_ratio = (ratio.is_finite() && ratio > 0.0).then_some(ratio);
    style.flex_grow = values.number(keys::FLEX_GROW).max(0.0);
    style.flex_shrink = values.number(keys::FLEX_SHRINK).max(0.0);
    style.flex_basis = values.size(keys::FLEX_BASIS_KIND);
    style.align_self = item_alignment(values.choice(keys::ALIGN_SELF));
    style.justify_self = item_alignment(values.choice(keys::JUSTIFY_SELF));
}
