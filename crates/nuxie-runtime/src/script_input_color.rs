//! Pinned `ScriptInputColor` generated-field and handwritten callback
//! semantics.

use crate::data_bind_graph::RuntimeDataBindGraphValue;
use nuxie_binary::RuntimeObject;

/// Generated `CustomPropertyColorBase::m_PropertyValue` default, retained as
/// the exact unsigned ARGB bit pattern used by C++'s signed `int` field.
pub(crate) const DEFAULT_PROPERTY_VALUE: u32 = 0xFF1D_1D1D;

pub(crate) fn value_property_key() -> Option<u16> {
    crate::properties::property_key_for_name("ScriptInputColor", "propertyValue")
}

pub(crate) fn authored_target(
    input: &RuntimeObject,
    property_key: u32,
) -> Option<RuntimeDataBindGraphValue> {
    (value_property_key().map(u32::from) == Some(property_key)).then(|| {
        RuntimeDataBindGraphValue::Color(
            input
                .color_property("propertyValue")
                .unwrap_or(DEFAULT_PROPERTY_VALUE),
        )
    })
}
