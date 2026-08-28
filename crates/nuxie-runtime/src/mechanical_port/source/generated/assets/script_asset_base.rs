use crate::mechanical_port::source::{
    assets::{script_asset::ScriptAsset, text_asset::TextAsset},
    core::{
        binary_reader::BinaryReader,
        field_types::{core_bool_type::CoreBoolType, core_uint_type::CoreUintType},
    },
    generated::assets::text_asset_base::TextAssetBaseCallbacks,
};

pub trait ScriptAssetBaseCallbacks: TextAssetBaseCallbacks {
    fn generator_function_ref_changed(&mut self) {}
    fn is_module_changed(&mut self) {}
    fn serialized_implemented_methods_changed(&mut self) {}
}

pub struct ScriptAssetBase {
    pub base: TextAsset,
    generator_function_ref: u32,
    is_module: bool,
    serialized_implemented_methods: u32,
}

impl Default for ScriptAssetBase {
    fn default() -> Self {
        Self {
            base: TextAsset::default(),
            generator_function_ref: 0,
            is_module: false,
            serialized_implemented_methods: 2_097_151,
        }
    }
}

impl ScriptAssetBase {
    pub const TYPE_KEY: u16 = 529;
    pub const GENERATOR_FUNCTION_REF_PROPERTY_KEY: u16 = 893;
    pub const IS_MODULE_PROPERTY_KEY: u16 = 914;
    pub const SERIALIZED_IMPLEMENTED_METHODS_PROPERTY_KEY: u16 = 1022;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 971 | 103 | 99)
    }

    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }

    pub fn generator_function_ref(&self) -> u32 {
        self.generator_function_ref
    }

    pub fn set_generator_function_ref<C: ScriptAssetBaseCallbacks>(
        &mut self,
        value: u32,
        callbacks: &mut C,
    ) {
        if !self.set_generator_function_ref_value(value) {
            return;
        }
        callbacks.generator_function_ref_changed();
        callbacks.notify_property_changed(Self::GENERATOR_FUNCTION_REF_PROPERTY_KEY);
    }
    pub(crate) fn set_generator_function_ref_value(&mut self, value: u32) -> bool {
        if self.generator_function_ref == value {
            return false;
        }
        self.generator_function_ref = value;
        true
    }

    pub fn is_module(&self) -> bool {
        self.is_module
    }

    pub fn set_is_module<C: ScriptAssetBaseCallbacks>(&mut self, value: bool, callbacks: &mut C) {
        if !self.set_is_module_value(value) {
            return;
        }
        callbacks.is_module_changed();
        callbacks.notify_property_changed(Self::IS_MODULE_PROPERTY_KEY);
    }
    pub(crate) fn set_is_module_value(&mut self, value: bool) -> bool {
        if self.is_module == value {
            return false;
        }
        self.is_module = value;
        true
    }

    pub fn serialized_implemented_methods(&self) -> u32 {
        self.serialized_implemented_methods
    }

    pub fn set_serialized_implemented_methods<C: ScriptAssetBaseCallbacks>(
        &mut self,
        value: u32,
        callbacks: &mut C,
    ) {
        if !self.set_serialized_implemented_methods_value(value) {
            return;
        }
        callbacks.serialized_implemented_methods_changed();
        callbacks.notify_property_changed(Self::SERIALIZED_IMPLEMENTED_METHODS_PROPERTY_KEY);
    }
    pub(crate) fn set_serialized_implemented_methods_value(&mut self, value: u32) -> bool {
        if self.serialized_implemented_methods == value {
            return false;
        }
        self.serialized_implemented_methods = value;
        true
    }

    pub fn clone_into<C: ScriptAssetBaseCallbacks>(&self, callbacks: &mut C) -> ScriptAsset {
        let mut cloned = ScriptAsset::default();
        cloned.base.copy(self, callbacks);
        cloned
    }

    pub fn copy<C: ScriptAssetBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.generator_function_ref = object.generator_function_ref;
        self.is_module = object.is_module;
        self.serialized_implemented_methods = object.serialized_implemented_methods;
        self.base.base.copy(&object.base.base, callbacks);
    }

    pub fn deserialize<C: ScriptAssetBaseCallbacks>(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut C,
    ) -> bool {
        match property_key {
            Self::GENERATOR_FUNCTION_REF_PROPERTY_KEY => {
                self.generator_function_ref = CoreUintType::deserialize(reader);
                true
            }
            Self::IS_MODULE_PROPERTY_KEY => {
                self.is_module = CoreBoolType::deserialize(reader);
                true
            }
            Self::SERIALIZED_IMPLEMENTED_METHODS_PROPERTY_KEY => {
                self.serialized_implemented_methods = CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ScriptAssetBase {
    type Target = TextAsset;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ScriptAssetBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
