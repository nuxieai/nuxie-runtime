use crate::mechanical_port::source::{
    core::{binary_reader::BinaryReader, field_types::core_bytes_type::CoreBytesType},
    custom_property::CustomProperty,
    script_input_viewmodel_property::ScriptInputViewModelProperty,
};

pub trait ScriptInputViewModelPropertyBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn decode_data_bind_path_ids(&mut self, value: &[u8]);
    fn data_bind_path_ids_changed(&mut self) {}
}

pub struct ScriptInputViewModelPropertyBase {
    pub base: CustomProperty,
}
impl Default for ScriptInputViewModelPropertyBase {
    fn default() -> Self {
        Self {
            base: CustomProperty::default(),
        }
    }
}
impl ScriptInputViewModelPropertyBase {
    pub const TYPE_KEY: u16 = 612;
    pub const DATA_BIND_PATH_IDS_PROPERTY_KEY: u16 = 866;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 167 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn copy<C: ScriptInputViewModelPropertyBaseCallbacks>(&mut self, object: &Self, c: &mut C) {
        self.base.base.copy(&object.base.base, c);
    }
    pub fn deserialize<C: ScriptInputViewModelPropertyBaseCallbacks>(
        &mut self,
        key: u16,
        reader: &mut BinaryReader<'_>,
        c: &mut C,
    ) -> bool {
        match key {
            Self::DATA_BIND_PATH_IDS_PROPERTY_KEY => {
                c.decode_data_bind_path_ids(CoreBytesType::deserialize(reader).as_slice());
                true
            }
            _ => self.base.base.deserialize(key, reader, c),
        }
    }
    pub fn clone_into<C: ScriptInputViewModelPropertyBaseCallbacks>(
        &self,
        c: &mut C,
    ) -> ScriptInputViewModelProperty {
        let mut cloned = ScriptInputViewModelProperty::default();
        cloned.base.copy(self, c);
        cloned
    }
}

impl std::ops::Deref for ScriptInputViewModelPropertyBase {
    type Target = CustomProperty;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ScriptInputViewModelPropertyBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
