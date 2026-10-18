//! Keys, value kinds and defaults of the CSS layout records, as
//! defs/nuxie/*.json allocate them (docs/CSS_LAYOUT.md, section 1).
//!
//! The five record types share one contiguous property band, 61000 to
//! 61102, so a key alone names its owner type and its slot.
//! `LayoutCssStyle`'s z-index keys (61099 and 61100) and `BoxPaint`'s
//! percentages (61101 and 61102) came after the band filled; a type's slots
//! span from its first key to its last, and only its own keys are its.

/// The first and last property keys of the CSS record band.
pub const FIRST_KEY: u16 = 61000;
pub const LAST_KEY: u16 = 61102;

pub const LAYOUT_CSS_STYLE: u16 = 61000;
pub const BOX_BORDER: u16 = 61001;
pub const BOX_GRADIENT: u16 = 61002;
pub const BOX_SHADOW: u16 = 61003;
pub const BOX_PAINT: u16 = 61004;

// LayoutCssStyle.
pub const DISPLAY: u16 = 61000;
pub const BOX_SIZING: u16 = 61001;
pub const POSITION: u16 = 61002;
pub const OVERFLOW_X: u16 = 61003;
pub const OVERFLOW_Y: u16 = 61004;
pub const DIRECTION: u16 = 61005;
pub const FLEX_DIRECTION: u16 = 61006;
pub const FLEX_WRAP: u16 = 61007;
pub const FLEX_GROW: u16 = 61008;
pub const FLEX_SHRINK: u16 = 61009;
pub const FLEX_BASIS_KIND: u16 = 61010;
pub const ALIGN_ITEMS: u16 = 61013;
pub const ALIGN_SELF: u16 = 61014;
pub const JUSTIFY_ITEMS: u16 = 61015;
pub const JUSTIFY_SELF: u16 = 61016;
pub const ALIGN_CONTENT: u16 = 61017;
pub const JUSTIFY_CONTENT: u16 = 61018;
pub const GRID_AUTO_FLOW: u16 = 61019;
pub const ASPECT_RATIO: u16 = 61020;
pub const WIDTH_KIND: u16 = 61021;
pub const HEIGHT_KIND: u16 = 61024;
pub const MIN_WIDTH_KIND: u16 = 61027;
pub const MIN_HEIGHT_KIND: u16 = 61030;
pub const MAX_WIDTH_KIND: u16 = 61033;
pub const MAX_HEIGHT_KIND: u16 = 61036;
/// Margin kind, length and percent per side: top, right, bottom, left.
pub const MARGIN_TOP_KIND: u16 = 61039;
/// Inset kind, length and percent per side: top, right, bottom, left.
pub const INSET_TOP_KIND: u16 = 61051;
/// Padding length and percent per side: top, right, bottom, left.
pub const PADDING_TOP: u16 = 61063;
/// Border widths: top, right, bottom, left.
pub const BORDER_TOP: u16 = 61071;
pub const ROW_GAP: u16 = 61075;
pub const COLUMN_GAP: u16 = 61077;
/// Radius percent parts: top left, top right, bottom right, bottom left.
pub const RADIUS_TOP_LEFT_PERCENT: u16 = 61079;

// BoxBorder, BoxGradient and BoxShadow.
pub const BORDER_SIDES: u16 = 61083;
pub const GRADIENT_ANGLE: u16 = 61084;
pub const GRADIENT_SHAPE: u16 = 61085;
pub const GRADIENT_SIZE: u16 = 61086;
pub const GRADIENT_CENTER_X: u16 = 61087;
pub const GRADIENT_CENTER_Y: u16 = 61089;
pub const GRADIENT_RADIUS_X: u16 = 61091;
pub const GRADIENT_RADIUS_Y: u16 = 61093;
pub const GRADIENT_TO_CORNER: u16 = 61095;
pub const SHADOW_SPREAD: u16 = 61096;
pub const PAINT_WIDTH: u16 = 61097;
pub const PAINT_HEIGHT: u16 = 61098;
pub const Z_INDEX_KIND: u16 = 61099;
pub const Z_INDEX: u16 = 61100;
pub const PAINT_WIDTH_PERCENT: u16 = 61101;
pub const PAINT_HEIGHT_PERCENT: u16 = 61102;

/// How one property is stored. Enums are `uint8` on the wire and their
/// largest valid value is given; numbers are `double` (an f32 at run time).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Field {
    Enum { max: u8, default: u8 },
    Number { default: f32 },
}

const fn e(max: u8) -> Field {
    Field::Enum { max, default: 0 }
}
const N: Field = Field::Number { default: 0.0 };
const fn n(default: f32) -> Field {
    Field::Number { default }
}

/// Every property of the band, in key order from [`FIRST_KEY`].
const FIELDS: [Field; (LAST_KEY - FIRST_KEY + 1) as usize] = [
    e(4),   // 61000 display
    e(1),   // 61001 boxSizing
    e(2),   // 61002 position
    e(4),   // 61003 overflowX
    e(4),   // 61004 overflowY
    e(2),   // 61005 direction
    e(3),   // 61006 flexDirection
    e(2),   // 61007 flexWrap
    N,      // 61008 flexGrow
    n(1.0), // 61009 flexShrink
    e(7),   // 61010 flexBasisKind
    N,      // 61011 flexBasis
    N,      // 61012 flexBasisPercent
    e(10),  // 61013 alignItems
    e(10),  // 61014 alignSelf
    e(10),  // 61015 justifyItems
    e(10),  // 61016 justifySelf
    e(9),   // 61017 alignContent
    e(9),   // 61018 justifyContent
    e(3),   // 61019 gridAutoFlow
    N,      // 61020 aspectRatio
    e(7),   // 61021 widthKind
    N,      // 61022 width
    N,      // 61023 widthPercent
    e(7),   // 61024 heightKind
    N,      // 61025 height
    N,      // 61026 heightPercent
    e(2),   // 61027 minWidthKind
    N,      // 61028 minWidth
    N,      // 61029 minWidthPercent
    e(2),   // 61030 minHeightKind
    N,      // 61031 minHeight
    N,      // 61032 minHeightPercent
    e(2),   // 61033 maxWidthKind
    N,      // 61034 maxWidth
    N,      // 61035 maxWidthPercent
    e(2),   // 61036 maxHeightKind
    N,      // 61037 maxHeight
    N,      // 61038 maxHeightPercent
    e(1),   // 61039 marginTopKind
    N,      // 61040 marginTop
    N,      // 61041 marginTopPercent
    e(1),   // 61042 marginRightKind
    N,      // 61043 marginRight
    N,      // 61044 marginRightPercent
    e(1),   // 61045 marginBottomKind
    N,      // 61046 marginBottom
    N,      // 61047 marginBottomPercent
    e(1),   // 61048 marginLeftKind
    N,      // 61049 marginLeft
    N,      // 61050 marginLeftPercent
    e(1),   // 61051 insetTopKind
    N,      // 61052 insetTop
    N,      // 61053 insetTopPercent
    e(1),   // 61054 insetRightKind
    N,      // 61055 insetRight
    N,      // 61056 insetRightPercent
    e(1),   // 61057 insetBottomKind
    N,      // 61058 insetBottom
    N,      // 61059 insetBottomPercent
    e(1),   // 61060 insetLeftKind
    N,      // 61061 insetLeft
    N,      // 61062 insetLeftPercent
    N,      // 61063 paddingTop
    N,      // 61064 paddingTopPercent
    N,      // 61065 paddingRight
    N,      // 61066 paddingRightPercent
    N,      // 61067 paddingBottom
    N,      // 61068 paddingBottomPercent
    N,      // 61069 paddingLeft
    N,      // 61070 paddingLeftPercent
    N,      // 61071 borderTop
    N,      // 61072 borderRight
    N,      // 61073 borderBottom
    N,      // 61074 borderLeft
    N,      // 61075 rowGap
    N,      // 61076 rowGapPercent
    N,      // 61077 columnGap
    N,      // 61078 columnGapPercent
    N,      // 61079 radiusTopLeftPercent
    N,      // 61080 radiusTopRightPercent
    N,      // 61081 radiusBottomRightPercent
    N,      // 61082 radiusBottomLeftPercent
    Field::Enum {
        max: 15,
        default: 15,
    }, // 61083 BoxBorder sides
    n(180.0), // 61084 BoxGradient angle
    e(1),   // 61085 shape
    e(4),   // 61086 size
    N,      // 61087 centerX
    n(50.0), // 61088 centerXPercent
    N,      // 61089 centerY
    n(50.0), // 61090 centerYPercent
    N,      // 61091 radiusX
    N,      // 61092 radiusXPercent
    N,      // 61093 radiusY
    N,      // 61094 radiusYPercent
    e(4),   // 61095 toCorner
    N,      // 61096 BoxShadow spread
    N,      // 61097 BoxPaint width
    N,      // 61098 height
    e(1),   // 61099 LayoutCssStyle zIndexKind
    N,      // 61100 zIndex
    n(100.0), // 61101 BoxPaint widthPercent
    n(100.0), // 61102 heightPercent
];

/// The type that owns `key`, and its first property key.
pub fn owner(key: u16) -> Option<(u16, u16)> {
    match key {
        61000..=61082 => Some((LAYOUT_CSS_STYLE, 61000)),
        BORDER_SIDES => Some((BOX_BORDER, BORDER_SIDES)),
        61084..=61095 => Some((BOX_GRADIENT, GRADIENT_ANGLE)),
        SHADOW_SPREAD => Some((BOX_SHADOW, SHADOW_SPREAD)),
        PAINT_WIDTH..=PAINT_HEIGHT | PAINT_WIDTH_PERCENT..=PAINT_HEIGHT_PERCENT => {
            Some((BOX_PAINT, PAINT_WIDTH))
        }
        Z_INDEX_KIND..=Z_INDEX => Some((LAYOUT_CSS_STYLE, 61000)),
        _ => None,
    }
}

/// The property range of a record type, as (first key, count).
pub fn range(type_key: u16) -> Option<(u16, usize)> {
    match type_key {
        // Its slots span the band to its z-index keys.
        LAYOUT_CSS_STYLE => Some((61000, 101)),
        BOX_BORDER => Some((BORDER_SIDES, 1)),
        BOX_GRADIENT => Some((GRADIENT_ANGLE, 12)),
        BOX_SHADOW => Some((SHADOW_SPREAD, 1)),
        // Its slots span to its percentages.
        BOX_PAINT => Some((PAINT_WIDTH, 6)),
        _ => None,
    }
}

pub fn field(key: u16) -> Option<Field> {
    owner(key)?;
    FIELDS.get(usize::from(key - FIRST_KEY)).copied()
}

/// The registry's field id for a key: 0 for a uint (an enum), 2 for a
/// double, as `CoreRegistry::property_field_id` numbers them.
pub fn field_id(key: i32) -> Option<i32> {
    match field(u16::try_from(key).ok()?)? {
        Field::Enum { .. } => Some(0),
        Field::Number { .. } => Some(2),
    }
}

pub fn default_value(key: u16) -> f32 {
    match field(key) {
        Some(Field::Enum { default, .. }) => f32::from(default),
        Some(Field::Number { default }) => default,
        None => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table is checked against the schema generated from the
    /// authoritative definition files in defs/nuxie/.
    #[test]
    fn keys_match_the_nuxie_definitions() {
        let mut seen = 0;
        for (file, type_key) in [
            ("nuxie/layout_css_style.json", LAYOUT_CSS_STYLE),
            ("nuxie/box_border.json", BOX_BORDER),
            ("nuxie/box_gradient.json", BOX_GRADIENT),
            ("nuxie/box_shadow.json", BOX_SHADOW),
            ("nuxie/box_paint.json", BOX_PAINT),
        ] {
            let definition = nuxie_schema::definition_by_type_key(type_key).expect(file);
            assert_eq!(definition.file, file, "{file}");
            for property in definition.properties {
                let (name, key) = (property.name, property.key.int);
                assert_eq!(owner(key).map(|(owner, _)| owner), Some(type_key), "{name}");
                let initial: f32 = property.initial_value.expect(name).parse().expect(name);
                match (property.declared_type, field(key)) {
                    ("uint8", Some(Field::Enum { default, .. })) => {
                        assert_eq!(f32::from(default), initial, "{name}")
                    }
                    ("double", Some(Field::Number { default })) => {
                        assert_eq!(default, initial, "{name}")
                    }
                    other => panic!("{name}: {other:?}"),
                }
                seen += 1;
            }
            let (first, count) = range(type_key).unwrap();
            assert!(owner(first).is_some() && owner(first + count as u16 - 1).is_some());
        }
        assert_eq!(seen, usize::from(LAST_KEY - FIRST_KEY + 1));
    }
}
