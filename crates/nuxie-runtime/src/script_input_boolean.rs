//! Pinned `ScriptInputBoolean` generated-field and handwritten callback
//! semantics.

use crate::data_bind_graph::RuntimeDataBindGraphValue;
use nuxie_binary::RuntimeObject;

/// Generated `CustomPropertyBooleanBase::m_PropertyValue` default.
pub(crate) const DEFAULT_PROPERTY_VALUE: bool = false;

pub(crate) fn value_property_key() -> Option<u16> {
    crate::properties::property_key_for_name("ScriptInputBoolean", "propertyValue")
}

/// Generated `propertyValue() const` over the occurrence-owned Core value.
/// An absent serialized field is the generated `false` backing-field default.
pub(crate) fn property_value(value: Option<&RuntimeDataBindGraphValue>) -> bool {
    match value {
        Some(RuntimeDataBindGraphValue::Boolean(value)) => *value,
        _ => DEFAULT_PROPERTY_VALUE,
    }
}

/// Generated `ScriptInputBooleanBase::clone` + `copy`: construct a fresh
/// Boolean input occurrence and copy the backing field without firing its
/// changed callback or property notification.
pub(crate) fn clone_property_value(
    value: Option<&RuntimeDataBindGraphValue>,
) -> RuntimeDataBindGraphValue {
    RuntimeDataBindGraphValue::Boolean(property_value(value))
}

/// Generated equality-guarded `propertyValue(bool)` setter. Returning `true`
/// publishes the exact point after the backing field changed and before the
/// handwritten callback and inherited property notification run.
pub(crate) fn set_property_value(
    value: &mut Option<RuntimeDataBindGraphValue>,
    next: bool,
) -> bool {
    if property_value(value.as_ref()) == next {
        return false;
    }
    *value = Some(RuntimeDataBindGraphValue::Boolean(next));
    true
}

/// Handwritten `propertyValueChanged` and `initScriptedValue` both project the
/// current generated value through `ScriptedObject::setBooleanInput`.
pub(crate) fn scripted_value(
    value: Option<&RuntimeDataBindGraphValue>,
) -> RuntimeDataBindGraphValue {
    RuntimeDataBindGraphValue::Boolean(property_value(value))
}

pub(crate) fn authored_target(
    input: &RuntimeObject,
    property_key: u32,
) -> Option<RuntimeDataBindGraphValue> {
    (value_property_key().map(u32::from) == Some(property_key)).then(|| {
        RuntimeDataBindGraphValue::Boolean(
            input
                .bool_property("propertyValue")
                .unwrap_or(DEFAULT_PROPERTY_VALUE),
        )
    })
}
