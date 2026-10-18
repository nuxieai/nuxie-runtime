//! Keys, value kinds and defaults of the CSS layout records, as
//! defs/nuxie/*.json allocate them (docs/CSS_LAYOUT.md, section 1).
//!
//! The four record types share one contiguous property band, 60022 to
//! 60118, so a key alone names its owner type and its slot.

/// The first and last property keys of the CSS record band.
pub const FIRST_KEY: u16 = 60022;
pub const LAST_KEY: u16 = 60118;

pub const LAYOUT_CSS_STYLE: u16 = 60002;
pub const BOX_BORDER: u16 = 60003;
pub const BOX_GRADIENT: u16 = 60004;
pub const BOX_SHADOW: u16 = 60005;

// LayoutCssStyle.
pub const DISPLAY: u16 = 60022;
pub const BOX_SIZING: u16 = 60023;
pub const POSITION: u16 = 60024;
pub const OVERFLOW_X: u16 = 60025;
pub const OVERFLOW_Y: u16 = 60026;
pub const DIRECTION: u16 = 60027;
pub const FLEX_DIRECTION: u16 = 60028;
pub const FLEX_WRAP: u16 = 60029;
pub const FLEX_GROW: u16 = 60030;
pub const FLEX_SHRINK: u16 = 60031;
pub const FLEX_BASIS_KIND: u16 = 60032;
pub const ALIGN_ITEMS: u16 = 60035;
pub const ALIGN_SELF: u16 = 60036;
pub const JUSTIFY_ITEMS: u16 = 60037;
pub const JUSTIFY_SELF: u16 = 60038;
pub const ALIGN_CONTENT: u16 = 60039;
pub const JUSTIFY_CONTENT: u16 = 60040;
pub const GRID_AUTO_FLOW: u16 = 60041;
pub const ASPECT_RATIO: u16 = 60042;
pub const WIDTH_KIND: u16 = 60043;
pub const HEIGHT_KIND: u16 = 60046;
pub const MIN_WIDTH_KIND: u16 = 60049;
pub const MIN_HEIGHT_KIND: u16 = 60052;
pub const MAX_WIDTH_KIND: u16 = 60055;
pub const MAX_HEIGHT_KIND: u16 = 60058;
/// Margin kind, length and percent per side: top, right, bottom, left.
pub const MARGIN_TOP_KIND: u16 = 60061;
/// Inset kind, length and percent per side: top, right, bottom, left.
pub const INSET_TOP_KIND: u16 = 60073;
/// Padding length and percent per side: top, right, bottom, left.
pub const PADDING_TOP: u16 = 60085;
/// Border widths: top, right, bottom, left.
pub const BORDER_TOP: u16 = 60093;
pub const ROW_GAP: u16 = 60097;
pub const COLUMN_GAP: u16 = 60099;
/// Radius percent parts: top left, top right, bottom right, bottom left.
pub const RADIUS_TOP_LEFT_PERCENT: u16 = 60101;

// BoxBorder, BoxGradient and BoxShadow.
pub const BORDER_SIDES: u16 = 60105;
pub const GRADIENT_ANGLE: u16 = 60106;
pub const GRADIENT_SHAPE: u16 = 60107;
pub const GRADIENT_SIZE: u16 = 60108;
pub const GRADIENT_CENTER_X: u16 = 60109;
pub const GRADIENT_CENTER_Y: u16 = 60111;
pub const GRADIENT_RADIUS_X: u16 = 60113;
pub const GRADIENT_RADIUS_Y: u16 = 60115;
pub const GRADIENT_TO_CORNER: u16 = 60117;
pub const SHADOW_SPREAD: u16 = 60118;

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
    e(4),   // 60022 display
    e(1),   // 60023 boxSizing
    e(2),   // 60024 position
    e(4),   // 60025 overflowX
    e(4),   // 60026 overflowY
    e(2),   // 60027 direction
    e(3),   // 60028 flexDirection
    e(2),   // 60029 flexWrap
    N,      // 60030 flexGrow
    n(1.0), // 60031 flexShrink
    e(7),   // 60032 flexBasisKind
    N,      // 60033 flexBasis
    N,      // 60034 flexBasisPercent
    e(10),  // 60035 alignItems
    e(10),  // 60036 alignSelf
    e(10),  // 60037 justifyItems
    e(10),  // 60038 justifySelf
    e(9),   // 60039 alignContent
    e(9),   // 60040 justifyContent
    e(3),   // 60041 gridAutoFlow
    N,      // 60042 aspectRatio
    e(7),   // 60043 widthKind
    N,      // 60044 width
    N,      // 60045 widthPercent
    e(7),   // 60046 heightKind
    N,      // 60047 height
    N,      // 60048 heightPercent
    e(2),   // 60049 minWidthKind
    N,      // 60050 minWidth
    N,      // 60051 minWidthPercent
    e(2),   // 60052 minHeightKind
    N,      // 60053 minHeight
    N,      // 60054 minHeightPercent
    e(2),   // 60055 maxWidthKind
    N,      // 60056 maxWidth
    N,      // 60057 maxWidthPercent
    e(2),   // 60058 maxHeightKind
    N,      // 60059 maxHeight
    N,      // 60060 maxHeightPercent
    e(1),   // 60061 marginTopKind
    N,      // 60062 marginTop
    N,      // 60063 marginTopPercent
    e(1),   // 60064 marginRightKind
    N,      // 60065 marginRight
    N,      // 60066 marginRightPercent
    e(1),   // 60067 marginBottomKind
    N,      // 60068 marginBottom
    N,      // 60069 marginBottomPercent
    e(1),   // 60070 marginLeftKind
    N,      // 60071 marginLeft
    N,      // 60072 marginLeftPercent
    e(1),   // 60073 insetTopKind
    N,      // 60074 insetTop
    N,      // 60075 insetTopPercent
    e(1),   // 60076 insetRightKind
    N,      // 60077 insetRight
    N,      // 60078 insetRightPercent
    e(1),   // 60079 insetBottomKind
    N,      // 60080 insetBottom
    N,      // 60081 insetBottomPercent
    e(1),   // 60082 insetLeftKind
    N,      // 60083 insetLeft
    N,      // 60084 insetLeftPercent
    N,      // 60085 paddingTop
    N,      // 60086 paddingTopPercent
    N,      // 60087 paddingRight
    N,      // 60088 paddingRightPercent
    N,      // 60089 paddingBottom
    N,      // 60090 paddingBottomPercent
    N,      // 60091 paddingLeft
    N,      // 60092 paddingLeftPercent
    N,      // 60093 borderTop
    N,      // 60094 borderRight
    N,      // 60095 borderBottom
    N,      // 60096 borderLeft
    N,      // 60097 rowGap
    N,      // 60098 rowGapPercent
    N,      // 60099 columnGap
    N,      // 60100 columnGapPercent
    N,      // 60101 radiusTopLeftPercent
    N,      // 60102 radiusTopRightPercent
    N,      // 60103 radiusBottomRightPercent
    N,      // 60104 radiusBottomLeftPercent
    Field::Enum {
        max: 15,
        default: 15,
    }, // 60105 BoxBorder sides
    n(180.0), // 60106 BoxGradient angle
    e(1),   // 60107 shape
    e(4),   // 60108 size
    N,      // 60109 centerX
    n(50.0), // 60110 centerXPercent
    N,      // 60111 centerY
    n(50.0), // 60112 centerYPercent
    N,      // 60113 radiusX
    N,      // 60114 radiusXPercent
    N,      // 60115 radiusY
    N,      // 60116 radiusYPercent
    e(4),   // 60117 toCorner
    N,      // 60118 BoxShadow spread
];

/// The type that owns `key`, and its first property key.
pub fn owner(key: u16) -> Option<(u16, u16)> {
    match key {
        60022..=60104 => Some((LAYOUT_CSS_STYLE, 60022)),
        BORDER_SIDES => Some((BOX_BORDER, BORDER_SIDES)),
        60106..=60117 => Some((BOX_GRADIENT, GRADIENT_ANGLE)),
        SHADOW_SPREAD => Some((BOX_SHADOW, SHADOW_SPREAD)),
        _ => None,
    }
}

/// The property range of a record type, as (first key, count).
pub fn range(type_key: u16) -> Option<(u16, usize)> {
    match type_key {
        LAYOUT_CSS_STYLE => Some((60022, 83)),
        BOX_BORDER => Some((BORDER_SIDES, 1)),
        BOX_GRADIENT => Some((GRADIENT_ANGLE, 12)),
        BOX_SHADOW => Some((SHADOW_SPREAD, 1)),
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
