use crate::InstanceSlot;
use rive_binary::{
    BytesValue, FieldValue, RuntimeFile, RuntimeObject, RuntimeProperty, StringValue,
};
use rive_schema::{
    FieldKind, StoredFieldInitializer, core_registry_setter_field_kind_by_property_key,
    definition_by_name, definition_by_type_key,
};

#[derive(Debug, Clone)]
pub(crate) struct InstanceObjectArena {
    objects: Vec<Option<InstanceObject>>,
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
                .map(InstanceObject::from_runtime_object);
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
                .map(|object| object.as_ref().map(InstanceObject::from_runtime_object))
                .collect(),
        }
    }

    fn object(&self, local_id: usize) -> Option<&InstanceObject> {
        self.objects.get(local_id)?.as_ref()
    }

    fn object_mut(&mut self, local_id: usize) -> Option<&mut InstanceObject> {
        self.objects.get_mut(local_id)?.as_mut()
    }

    pub(crate) fn property_kind(&self, local_id: usize, property_key: u16) -> Option<FieldKind> {
        let object = self.object(local_id)?;
        object.field_kind(property_key)
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
        self.object(local_id)
            .and_then(|object| object.string_property(property_key))
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
            FieldKind::Bytes => FieldValue::Bytes(BytesValue::new(value)),
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
        let Some(type_key) = self.object(local_id).map(|object| object.type_key) else {
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
        let Some(value) = InstancePropertyValue::from_field_value(value) else {
            return false;
        };

        if let Some(existing) = object.property_mut_by_exact_key(property_key) {
            if existing.value == value {
                return false;
            }
            existing.value = value;
            return true;
        }

        object.properties.push(InstanceProperty {
            key: property_key,
            name: property.name,
            value,
        });
        true
    }
}

#[derive(Debug, Clone)]
struct InstanceObject {
    type_key: u16,
    type_name: &'static str,
    properties: Vec<InstanceProperty>,
}

impl InstanceObject {
    fn from_runtime_object(object: &RuntimeObject) -> Self {
        Self {
            type_key: object.type_key,
            type_name: object.type_name,
            properties: object
                .properties
                .iter()
                .map(InstanceProperty::from_runtime_property)
                .collect(),
        }
    }

    fn property_metadata(
        &self,
        property_key: u16,
    ) -> Option<(&'static str, &'static rive_schema::Property)> {
        runtime_property_metadata_by_key(self.type_key, property_key)
    }

    fn field_kind(&self, property_key: u16) -> Option<FieldKind> {
        self.property_metadata(property_key)
            .map(|(_, property)| property.runtime_type)
    }

    fn property(&self, property_key: u16) -> Option<&InstanceProperty> {
        let (_, property) = self.property_metadata(property_key)?;
        self.properties
            .iter()
            .rev()
            .find(|candidate| candidate.name == property.name)
    }

    fn property_mut_by_exact_key(&mut self, property_key: u16) -> Option<&mut InstanceProperty> {
        self.properties
            .iter_mut()
            .rev()
            .find(|property| property.key == property_key)
    }

    fn stored_field_initializer(
        &self,
        property: &'static rive_schema::Property,
    ) -> Option<StoredFieldInitializer> {
        // C++ Artboard::Artboard overrides the inherited LayoutComponent
        // default so artboards clip to their bounds unless serialized otherwise.
        if self.type_name == "Artboard" && property.name == "clip" {
            return Some(StoredFieldInitializer::Bool(true));
        }
        (*property).stored_field_initializer()
    }

    fn double_property(&self, property_key: u16) -> Option<f32> {
        if let Some(property) = self.property(property_key) {
            return property.value.as_double();
        }

        let (_, property) = self.property_metadata(property_key)?;
        match self.stored_field_initializer(property)? {
            StoredFieldInitializer::Double(value) => Some(value),
            _ => None,
        }
    }

    fn uint_property(&self, property_key: u16) -> Option<u64> {
        if let Some(property) = self.property(property_key) {
            return property.value.as_uint();
        }

        let (_, property) = self.property_metadata(property_key)?;
        match self.stored_field_initializer(property)? {
            StoredFieldInitializer::Uint(value) => Some(u64::from(value)),
            _ => None,
        }
    }

    fn bool_property(&self, property_key: u16) -> Option<bool> {
        if let Some(property) = self.property(property_key) {
            return property.value.as_bool();
        }

        let (_, property) = self.property_metadata(property_key)?;
        match self.stored_field_initializer(property)? {
            StoredFieldInitializer::Bool(value) => Some(value),
            _ => None,
        }
    }

    fn color_property(&self, property_key: u16) -> Option<u32> {
        if let Some(property) = self.property(property_key) {
            return property.value.as_color();
        }

        let (_, property) = self.property_metadata(property_key)?;
        match self.stored_field_initializer(property)? {
            StoredFieldInitializer::Color(value) => Some(value),
            _ => None,
        }
    }

    fn string_property(&self, property_key: u16) -> Option<&[u8]> {
        let (_, property) = self.property_metadata(property_key)?;
        match property.runtime_type {
            FieldKind::String => {
                if let Some(property) = self.property(property_key) {
                    return property.value.as_string_bytes();
                }
                match self.stored_field_initializer(property)? {
                    StoredFieldInitializer::String(value) => Some(value.as_bytes()),
                    _ => None,
                }
            }
            FieldKind::Bytes => self
                .property(property_key)
                .and_then(|property| property.value.as_bytes()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct InstanceProperty {
    key: u16,
    name: &'static str,
    value: InstancePropertyValue,
}

impl InstanceProperty {
    fn from_runtime_property(property: &RuntimeProperty) -> Self {
        Self {
            key: property.key,
            name: property.name,
            value: InstancePropertyValue::from_field_value(property.value.clone())
                .expect("runtime properties never store callback values"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum InstancePropertyValue {
    Bool(bool),
    Bytes(Vec<u8>),
    Color(u32),
    Double(f32),
    String { value: Option<String>, raw: Vec<u8> },
    Uint(u64),
}

impl InstancePropertyValue {
    fn from_field_value(value: FieldValue) -> Option<Self> {
        match value {
            FieldValue::Bool(value) => Some(Self::Bool(value)),
            FieldValue::Bytes(value) => Some(Self::Bytes(value.raw)),
            FieldValue::Callback => None,
            FieldValue::Color(value) => Some(Self::Color(value)),
            FieldValue::Double(value) => Some(Self::Double(value)),
            FieldValue::String(value) => Some(Self::String {
                value: value.value,
                raw: value.raw,
            }),
            FieldValue::Uint(value) => Some(Self::Uint(value)),
        }
    }

    fn as_double(&self) -> Option<f32> {
        match self {
            Self::Double(value) => Some(*value),
            _ => None,
        }
    }

    fn as_uint(&self) -> Option<u64> {
        match self {
            Self::Uint(value) => Some(*value),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    fn as_color(&self) -> Option<u32> {
        match self {
            Self::Color(value) => Some(*value),
            _ => None,
        }
    }

    fn as_string_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::String { raw, .. } => Some(raw.as_slice()),
            _ => None,
        }
    }

    fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(value) => Some(value.as_slice()),
            _ => None,
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
