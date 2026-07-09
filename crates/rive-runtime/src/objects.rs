#[cfg(test)]
use rive_binary::RuntimeObject;
use rive_binary::{FieldValue, RuntimeFile, StringValue};
use rive_schema::{
    FieldKind, core_registry_setter_field_kind_by_property_key, definition_by_name,
    definition_by_type_key, property_by_key_in_hierarchy,
};

mod generated_objects {
    include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"));
}

use generated_objects::InstanceObjectStorage;

#[derive(Debug, Clone)]
pub struct InstanceSlot {
    pub local_id: usize,
    pub source_global_id: u32,
    pub type_name: Option<&'static str>,
    pub name: Option<String>,
    pub component_index: Option<usize>,
}

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
        let object = self.object(local_id)?;
        let (_owner, property) = runtime_property_metadata_by_key(object.type_key(), property_key)?;
        if let Some(bitmask) = property.bitmask_passthrough {
            let (_owner, target) =
                runtime_property_metadata_by_name(object.type_key(), bitmask.target)?;
            let packed = object.uint_property(target.key.int).unwrap_or(0);
            return Some((packed & bitmask_field_mask(bitmask.bit, bitmask.width)) >> bitmask.bit);
        }
        object.uint_property(property_key)
    }

    pub(crate) fn double_property(&self, local_id: usize, property_key: u16) -> Option<f32> {
        self.object(local_id)
            .and_then(|object| object.double_property(property_key))
    }

    pub(crate) fn double_property_by_name(
        &self,
        local_id: usize,
        property_name: &str,
    ) -> Option<f32> {
        let object = self.object(local_id)?;
        let (_, property) = runtime_property_metadata_by_name(object.type_key(), property_name)?;
        object.double_property(property.key.int)
    }

    #[cfg(test)]
    pub(crate) fn set_double_property_by_name(
        &mut self,
        local_id: usize,
        property_name: &str,
        value: f32,
    ) -> bool {
        let Some(type_key) = self.object(local_id).map(InstanceObjectStorage::type_key) else {
            return false;
        };
        let Some((_, property)) = runtime_property_metadata_by_name(type_key, property_name) else {
            return false;
        };
        self.set_double_property(local_id, property.key.int, value)
    }

    pub(crate) fn set_double_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: f32,
    ) -> bool {
        self.set_property_value(local_id, property_key, FieldValue::Double(value))
    }

    pub(crate) fn set_generated_double_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: f32,
    ) -> bool {
        self.object_mut(local_id)
            .is_some_and(|object| object.set_double_property(property_key, value))
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

        if let (Some(bitmask), FieldValue::Uint(value)) = (property.bitmask_passthrough, &value) {
            let Some((_owner, target)) =
                runtime_property_metadata_by_name(type_key, bitmask.target)
            else {
                return false;
            };
            let Some(object) = self.object_mut(local_id) else {
                return false;
            };
            let mask = bitmask_field_mask(bitmask.bit, bitmask.width);
            let current = object.uint_property(target.key.int).unwrap_or(0);
            let shifted = value.checked_shl(bitmask.bit.into()).unwrap_or(0);
            let next = (current & !mask) | (shifted & mask);
            return object.set_uint_property(target.key.int, next);
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

fn bitmask_field_mask(bit: u8, width: u8) -> u64 {
    if bit >= 64 {
        return 0;
    }
    let width = width.min(64 - bit);
    let width_mask = if width >= 64 {
        u64::MAX
    } else {
        (1u64 << width) - 1
    };
    width_mask << bit
}

fn runtime_property_metadata_by_key(
    type_key: u16,
    property_key: u16,
) -> Option<(&'static str, &'static rive_schema::Property)> {
    property_by_key_in_hierarchy(type_key, property_key)
}

fn runtime_property_metadata_by_name(
    type_key: u16,
    property_name: &str,
) -> Option<(&'static str, &'static rive_schema::Property)> {
    let definition = definition_by_type_key(type_key)?;
    definition
        .properties
        .iter()
        .find(|property| property.name == property_name)
        .map(|property| (definition.name, property))
        .or_else(|| {
            definition.ancestors.iter().find_map(|ancestor| {
                let definition = definition_by_name(ancestor)?;
                definition
                    .properties
                    .iter()
                    .find(|property| property.name == property_name)
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
