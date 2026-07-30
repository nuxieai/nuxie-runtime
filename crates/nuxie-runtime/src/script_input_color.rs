//! Pinned `src/script_input_color.cpp` property semantics.

use crate::data_bind_graph::RuntimeDataBindGraphValue;
use nuxie_binary::RuntimeObject;

pub(crate) fn value_property_key() -> Option<u16> {
    crate::properties::property_key_for_name("ScriptInputColor", "propertyValue")
}

pub(crate) fn authored_target(
    input: &RuntimeObject,
    property_key: u32,
) -> Option<RuntimeDataBindGraphValue> {
    (value_property_key().map(u32::from) == Some(property_key)).then(|| {
        RuntimeDataBindGraphValue::Color(input.color_property("propertyValue").unwrap_or(0))
    })
}
