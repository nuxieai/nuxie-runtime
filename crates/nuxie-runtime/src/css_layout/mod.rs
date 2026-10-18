//! CSS layout records: our layer over Rive's layout, specified in
//! docs/CSS_LAYOUT.md. A `LayoutCssStyle` puts its box or participant in CSS
//! mode; Taffy then lays it out with its own CSS algorithms.
pub mod keys;
pub mod paint;
pub(crate) mod paint_order;
pub mod record;
pub mod style;

pub use record::{BoxBorder, BoxGradient, BoxPaint, BoxShadow, CssRecord, LayoutCssStyle};
pub use style::{CssRole, CssValues};

/// A CSS record by its type key, or None for any other key.
pub(crate) fn make_core(type_key: i32) -> Option<Box<dyn crate::source::core::CoreObject>> {
    Some(match u16::try_from(type_key).ok()? {
        keys::LAYOUT_CSS_STYLE => Box::new(LayoutCssStyle::default()),
        keys::BOX_BORDER => Box::new(BoxBorder::default()),
        keys::BOX_GRADIENT => Box::new(BoxGradient::default()),
        keys::BOX_SHADOW => Box::new(BoxShadow::default()),
        keys::BOX_PAINT => Box::new(BoxPaint::default()),
        _ => return None,
    })
}
