use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, drawable::Drawable, nested_artboard::NestedArtboard,
};

pub trait NestedArtboardBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn artboard_id_changed(&mut self) {}
    fn data_bind_path_ids_changed(&mut self) {}
    fn is_paused_changed(&mut self) {}
    fn speed_changed(&mut self) {}
    fn quantize_changed(&mut self) {}
    fn is_stateful_changed(&mut self) {}
    fn decode_data_bind_path_ids(&mut self, value: &[u8]);
    fn copy_data_bind_path_ids(&mut self, object: &NestedArtboardBase);
}

pub struct NestedArtboardBase {
    pub base: Drawable,
    artboard_id: u32,
    is_paused: bool,
    speed: f32,
    quantize: f32,
    is_stateful: bool,
}

impl Default for NestedArtboardBase {
    fn default() -> Self {
        Self {
            base: Drawable::default(),
            artboard_id: u32::MAX,
            is_paused: false,
            speed: 1.0,
            quantize: -1.0,
            is_stateful: false,
        }
    }
}

impl NestedArtboardBase {
    pub const TYPE_KEY: u16 = 92;
    pub const ARTBOARD_ID_PROPERTY_KEY: u16 = 197;
    pub const DATA_BIND_PATH_IDS_PROPERTY_KEY: u16 = 582;
    pub const IS_PAUSED_PROPERTY_KEY: u16 = 895;
    pub const SPEED_PROPERTY_KEY: u16 = 907;
    pub const QUANTIZE_PROPERTY_KEY: u16 = 908;
    pub const IS_STATEFUL_PROPERTY_KEY: u16 = 1014;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn artboard_id(&self) -> u32 {
        self.artboard_id
    }
    pub fn set_artboard_id(
        &mut self,
        value: u32,
        callbacks: &mut impl NestedArtboardBaseCallbacks,
    ) {
        if self.artboard_id == value {
            return;
        }
        self.artboard_id = value;
        callbacks.artboard_id_changed();
        callbacks.notify_property_changed(Self::ARTBOARD_ID_PROPERTY_KEY);
    }
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }
    pub fn set_is_paused(&mut self, value: bool, callbacks: &mut impl NestedArtboardBaseCallbacks) {
        if self.is_paused == value {
            return;
        }
        self.is_paused = value;
        callbacks.is_paused_changed();
        callbacks.notify_property_changed(Self::IS_PAUSED_PROPERTY_KEY);
    }
    pub fn speed(&self) -> f32 {
        self.speed
    }
    pub fn set_speed(&mut self, value: f32, callbacks: &mut impl NestedArtboardBaseCallbacks) {
        if self.speed == value {
            return;
        }
        self.speed = value;
        callbacks.speed_changed();
        callbacks.notify_property_changed(Self::SPEED_PROPERTY_KEY);
    }
    pub fn quantize(&self) -> f32 {
        self.quantize
    }
    pub fn set_quantize(&mut self, value: f32, callbacks: &mut impl NestedArtboardBaseCallbacks) {
        if self.quantize == value {
            return;
        }
        self.quantize = value;
        callbacks.quantize_changed();
        callbacks.notify_property_changed(Self::QUANTIZE_PROPERTY_KEY);
    }
    pub fn is_stateful(&self) -> bool {
        self.is_stateful
    }
    pub fn set_is_stateful(
        &mut self,
        value: bool,
        callbacks: &mut impl NestedArtboardBaseCallbacks,
    ) {
        if self.is_stateful == value {
            return;
        }
        self.is_stateful = value;
        callbacks.is_stateful_changed();
        callbacks.notify_property_changed(Self::IS_STATEFUL_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl NestedArtboardBaseCallbacks) -> NestedArtboard {
        let mut cloned = NestedArtboard::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl NestedArtboardBaseCallbacks) {
        self.artboard_id = object.artboard_id;
        callbacks.copy_data_bind_path_ids(object);
        self.is_paused = object.is_paused;
        self.speed = object.speed;
        self.quantize = object.quantize;
        self.is_stateful = object.is_stateful;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl NestedArtboardBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ARTBOARD_ID_PROPERTY_KEY => {
                self.artboard_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::IS_PAUSED_PROPERTY_KEY => {
                self.is_paused = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::SPEED_PROPERTY_KEY => {
                self.speed = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::QUANTIZE_PROPERTY_KEY => {
                self.quantize = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::IS_STATEFUL_PROPERTY_KEY => {
                self.is_stateful = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::DATA_BIND_PATH_IDS_PROPERTY_KEY => {
                let value = crate::mechanical_port::source::core::field_types::core_bytes_type::CoreBytesType::deserialize(reader);
                callbacks.decode_data_bind_path_ids(value.as_slice());
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
