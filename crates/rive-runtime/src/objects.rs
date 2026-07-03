use crate::InstanceSlot;
#[cfg(test)]
use rive_binary::RuntimeObject;
use rive_binary::{FieldValue, RuntimeFile, StringValue};
use rive_schema::{
    FieldKind, core_registry_setter_field_kind_by_property_key, definition_by_name,
    definition_by_type_key,
};

mod generated_objects {
    include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"));
}

use generated_objects::InstanceObjectStorage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstanceString {
    value: Option<String>,
    raw: Vec<u8>,
}

impl InstanceString {
    pub(crate) fn from_static(value: &'static str) -> Self {
        Self {
            value: Some(value.to_owned()),
            raw: value.as_bytes().to_vec(),
        }
    }

    pub(crate) fn from_string_value(value: &StringValue) -> Self {
        Self {
            value: value.value.clone(),
            raw: value.raw.clone(),
        }
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        self.raw.as_slice()
    }
}

pub(crate) fn set_optional_field<T: PartialEq>(field: &mut Option<T>, value: T) -> bool {
    if field.as_ref().is_some_and(|current| current == &value) {
        return false;
    }
    *field = Some(value);
    true
}

#[derive(Debug, Clone)]
pub(crate) struct InstanceObjectArena {
    objects: Vec<Option<InstanceObjectStorage>>,
}

impl InstanceObjectArena {
    pub(crate) fn from_slots(file: &RuntimeFile, slots: &[InstanceSlot]) -> Self {
        let mut objects = vec![None; slots.len()];
        for slot in slots {
            if slot.local_id >= objects.len() {
                objects.resize(slot.local_id + 1, None);
            }
            objects[slot.local_id] = file
                .object(slot.source_global_id as usize)
                .and_then(InstanceObjectStorage::from_runtime_object);
        }
        Self { objects }
    }

    #[cfg(test)]
    pub(crate) fn empty_for_slots(len: usize) -> Self {
        Self {
            objects: vec![None; len],
        }
    }

    #[cfg(test)]
    pub(crate) fn from_runtime_objects(objects: Vec<Option<RuntimeObject>>) -> Self {
        Self {
            objects: objects
                .iter()
                .map(|object| {
                    object
                        .as_ref()
                        .and_then(InstanceObjectStorage::from_runtime_object)
                })
                .collect(),
        }
    }

    fn object(&self, local_id: usize) -> Option<&InstanceObjectStorage> {
        self.objects.get(local_id)?.as_ref()
    }

    fn object_mut(&mut self, local_id: usize) -> Option<&mut InstanceObjectStorage> {
        self.objects.get_mut(local_id)?.as_mut()
    }

    pub(crate) fn property_kind(&self, local_id: usize, property_key: u16) -> Option<FieldKind> {
        let object = self.object(local_id)?;
        runtime_property_metadata_by_key(object.type_key(), property_key)
            .map(|(_, property)| property.runtime_type)
    }

    pub(crate) fn color_property(&self, local_id: usize, property_key: u16) -> Option<u32> {
        self.object(local_id)
            .and_then(|object| object.color_property(property_key))
    }

    pub(crate) fn set_color_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: u32,
    ) -> bool {
        self.set_property_value(local_id, property_key, FieldValue::Color(value))
    }

    pub(crate) fn bool_property(&self, local_id: usize, property_key: u16) -> Option<bool> {
        self.object(local_id)
            .and_then(|object| object.bool_property(property_key))
    }

    pub(crate) fn set_bool_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: bool,
    ) -> bool {
        self.set_property_value(local_id, property_key, FieldValue::Bool(value))
    }

    pub(crate) fn uint_property(&self, local_id: usize, property_key: u16) -> Option<u64> {
        self.object(local_id)
            .and_then(|object| object.uint_property(property_key))
    }

    pub(crate) fn double_property(&self, local_id: usize, property_key: u16) -> Option<f32> {
        self.object(local_id)
            .and_then(|object| object.double_property(property_key))
    }

    pub(crate) fn set_double_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: f32,
    ) -> bool {
        self.set_property_value(local_id, property_key, FieldValue::Double(value))
    }

    pub(crate) fn set_uint_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: u64,
    ) -> bool {
        self.set_property_value(local_id, property_key, FieldValue::Uint(value))
    }

    pub(crate) fn string_property(&self, local_id: usize, property_key: u16) -> Option<&[u8]> {
        let object = self.object(local_id)?;
        match self.property_kind(local_id, property_key)? {
            FieldKind::String => object.string_property(property_key),
            FieldKind::Bytes => object.bytes_property(property_key),
            _ => None,
        }
    }

    pub(crate) fn set_string_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: Vec<u8>,
    ) -> bool {
        let Some(kind) = self.property_kind(local_id, property_key) else {
            return false;
        };
        let value = match kind {
            FieldKind::String => FieldValue::String(StringValue {
                value: String::from_utf8(value.clone()).ok(),
                raw: value,
            }),
            FieldKind::Bytes => return false,
            _ => return false,
        };
        self.set_property_value(local_id, property_key, value)
    }

    fn set_property_value(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: FieldValue,
    ) -> bool {
        let Some(type_key) = self.object(local_id).map(InstanceObjectStorage::type_key) else {
            return false;
        };
        let Some((_owner, property)) = runtime_property_metadata_by_key(type_key, property_key)
        else {
            return false;
        };
        let Some(setter_kind) = core_registry_setter_field_kind_by_property_key(property_key)
        else {
            return false;
        };
        if !field_value_matches_kind(&value, setter_kind) {
            return false;
        }
        if !field_value_matches_kind(&value, property.runtime_type) {
            return false;
        }

        let Some(object) = self.object_mut(local_id) else {
            return false;
        };
        match value {
            FieldValue::Bool(value) => object.set_bool_property(property_key, value),
            FieldValue::Bytes(_) | FieldValue::Callback => false,
            FieldValue::Color(value) => object.set_color_property(property_key, value),
            FieldValue::Double(value) => object.set_double_property(property_key, value),
            FieldValue::String(value) => {
                object.set_string_property(property_key, InstanceString::from_string_value(&value))
            }
            FieldValue::Uint(value) => object.set_uint_property(property_key, value),
        }
    }
}

fn runtime_property_metadata_by_key(
    type_key: u16,
    property_key: u16,
) -> Option<(&'static str, &'static rive_schema::Property)> {
    let definition = definition_by_type_key(type_key)?;
    definition
        .property_by_key(property_key)
        .map(|property| (definition.name, property))
        .or_else(|| {
            definition.ancestors.iter().find_map(|ancestor| {
                let definition = definition_by_name(ancestor)?;
                definition
                    .property_by_key(property_key)
                    .map(|property| (*ancestor, property))
            })
        })
}

fn field_value_matches_kind(value: &FieldValue, kind: FieldKind) -> bool {
    matches!(
        (value, kind),
        (FieldValue::Bool(_), FieldKind::Bool)
            | (FieldValue::Bytes(_), FieldKind::Bytes)
            | (FieldValue::Callback, FieldKind::Callback)
            | (FieldValue::Color(_), FieldKind::Color)
            | (FieldValue::Double(_), FieldKind::Double)
            | (FieldValue::String(_), FieldKind::String)
            | (FieldValue::Uint(_), FieldKind::Uint)
    )
}
