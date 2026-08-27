use crate::mechanical_port::source::{
    component::Component,
    core::{
        binary_reader::BinaryReader,
        field_types::{core_string_type::CoreStringType, core_uint_type::CoreUintType},
    },
    semantic::semantic_data::SemanticData,
};

pub trait SemanticDataBaseCallbacks {
    fn role_changed(&mut self) {}
    fn label_changed(&mut self) {}
    fn value_changed(&mut self) {}
    fn hint_changed(&mut self) {}
    fn heading_level_changed(&mut self) {}
    fn trait_flags_changed(&mut self) {}
    fn state_flags_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}

pub struct SemanticDataBase {
    pub base: Component,
    role: u32,
    label: String,
    value: String,
    hint: String,
    heading_level: u32,
    trait_flags: u32,
    state_flags: u32,
}

impl Default for SemanticDataBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            role: 0,
            label: String::new(),
            value: String::new(),
            hint: String::new(),
            heading_level: 0,
            trait_flags: 0,
            state_flags: 0,
        }
    }
}

impl SemanticDataBase {
    pub const TYPE_KEY: u16 = 668;
    pub const ROLE_PROPERTY_KEY: u16 = 982;
    pub const LABEL_PROPERTY_KEY: u16 = 983;
    pub const VALUE_PROPERTY_KEY: u16 = 984;
    pub const HINT_PROPERTY_KEY: u16 = 985;
    pub const HEADING_LEVEL_PROPERTY_KEY: u16 = 986;
    pub const TRAIT_FLAGS_PROPERTY_KEY: u16 = 987;
    pub const STATE_FLAGS_PROPERTY_KEY: u16 = 988;
    pub const IS_EXPANDABLE_PROPERTY_KEY: u16 = 989;
    pub const IS_EXPANDABLE_BITMASK: u32 = 1 << 0;
    pub const IS_SELECTABLE_PROPERTY_KEY: u16 = 990;
    pub const IS_SELECTABLE_BITMASK: u32 = 1 << 1;
    pub const IS_CHECKABLE_PROPERTY_KEY: u16 = 991;
    pub const IS_CHECKABLE_BITMASK: u32 = 1 << 2;
    pub const IS_TOGGLEABLE_PROPERTY_KEY: u16 = 992;
    pub const IS_TOGGLEABLE_BITMASK: u32 = 1 << 3;
    pub const IS_REQUIRABLE_PROPERTY_KEY: u16 = 993;
    pub const IS_REQUIRABLE_BITMASK: u32 = 1 << 4;
    pub const IS_ENABLABLE_PROPERTY_KEY: u16 = 994;
    pub const IS_ENABLABLE_BITMASK: u32 = 1 << 5;
    pub const IS_FOCUSABLE_PROPERTY_KEY: u16 = 995;
    pub const IS_FOCUSABLE_BITMASK: u32 = 1 << 6;
    pub const IS_EXPANDED_PROPERTY_KEY: u16 = 996;
    pub const IS_EXPANDED_BITMASK: u32 = 1 << 0;
    pub const IS_SELECTED_PROPERTY_KEY: u16 = 997;
    pub const IS_SELECTED_BITMASK: u32 = 1 << 1;
    pub const IS_CHECKED_PROPERTY_KEY: u16 = 998;
    pub const IS_CHECKED_BITMASK: u32 = 1 << 2;
    pub const IS_MIXED_PROPERTY_KEY: u16 = 999;
    pub const IS_MIXED_BITMASK: u32 = 1 << 3;
    pub const IS_TOGGLED_PROPERTY_KEY: u16 = 1000;
    pub const IS_TOGGLED_BITMASK: u32 = 1 << 4;
    pub const IS_REQUIRED_PROPERTY_KEY: u16 = 1001;
    pub const IS_REQUIRED_BITMASK: u32 = 1 << 5;
    pub const IS_DISABLED_PROPERTY_KEY: u16 = 1002;
    pub const IS_DISABLED_BITMASK: u32 = 1 << 6;
    pub const IS_FOCUSED_PROPERTY_KEY: u16 = 1003;
    pub const IS_FOCUSED_BITMASK: u32 = 1 << 7;
    pub const IS_HIDDEN_PROPERTY_KEY: u16 = 1004;
    pub const IS_HIDDEN_BITMASK: u32 = 1 << 8;
    pub const IS_LIVE_REGION_PROPERTY_KEY: u16 = 1005;
    pub const IS_LIVE_REGION_BITMASK: u32 = 1 << 9;
    pub const IS_READ_ONLY_PROPERTY_KEY: u16 = 1006;
    pub const IS_READ_ONLY_BITMASK: u32 = 1 << 10;
    pub const IS_MODAL_PROPERTY_KEY: u16 = 1007;
    pub const IS_MODAL_BITMASK: u32 = 1 << 11;
    pub const IS_OBSCURED_PROPERTY_KEY: u16 = 1008;
    pub const IS_OBSCURED_BITMASK: u32 = 1 << 12;
    pub const IS_MULTILINE_PROPERTY_KEY: u16 = 1009;
    pub const IS_MULTILINE_BITMASK: u32 = 1 << 13;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn role(&self) -> u32 {
        self.role
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn value(&self) -> &str {
        &self.value
    }
    pub fn hint(&self) -> &str {
        &self.hint
    }
    pub fn heading_level(&self) -> u32 {
        self.heading_level
    }
    pub fn trait_flags(&self) -> u32 {
        self.trait_flags
    }
    pub fn state_flags(&self) -> u32 {
        self.state_flags
    }

    pub fn set_role<C: SemanticDataBaseCallbacks>(&mut self, value: u32, c: &mut C) {
        if self.role == value {
            return;
        }
        self.role = value;
        c.role_changed();
        c.notify_property_changed(Self::ROLE_PROPERTY_KEY);
    }
    pub fn set_label<C: SemanticDataBaseCallbacks>(&mut self, value: String, c: &mut C) {
        if self.label == value {
            return;
        }
        self.label = value;
        c.label_changed();
        c.notify_property_changed(Self::LABEL_PROPERTY_KEY);
    }
    pub fn set_value<C: SemanticDataBaseCallbacks>(&mut self, value: String, c: &mut C) {
        if self.value == value {
            return;
        }
        self.value = value;
        c.value_changed();
        c.notify_property_changed(Self::VALUE_PROPERTY_KEY);
    }
    pub fn set_hint<C: SemanticDataBaseCallbacks>(&mut self, value: String, c: &mut C) {
        if self.hint == value {
            return;
        }
        self.hint = value;
        c.hint_changed();
        c.notify_property_changed(Self::HINT_PROPERTY_KEY);
    }
    pub fn set_heading_level<C: SemanticDataBaseCallbacks>(&mut self, value: u32, c: &mut C) {
        if self.heading_level == value {
            return;
        }
        self.heading_level = value;
        c.heading_level_changed();
        c.notify_property_changed(Self::HEADING_LEVEL_PROPERTY_KEY);
    }
    pub fn set_trait_flags<C: SemanticDataBaseCallbacks>(&mut self, value: u32, c: &mut C) {
        if self.trait_flags == value {
            return;
        }
        self.trait_flags = value;
        c.trait_flags_changed();
        c.notify_property_changed(Self::TRAIT_FLAGS_PROPERTY_KEY);
    }
    pub fn set_state_flags<C: SemanticDataBaseCallbacks>(&mut self, value: u32, c: &mut C) {
        if self.state_flags == value {
            return;
        }
        self.state_flags = value;
        c.state_flags_changed();
        c.notify_property_changed(Self::STATE_FLAGS_PROPERTY_KEY);
    }

    pub fn clone_into<C: SemanticDataBaseCallbacks>(&self, c: &mut C) -> SemanticData {
        let mut cloned = SemanticData::default();
        cloned.base.copy(self, c);
        cloned
    }
    pub fn copy<C: SemanticDataBaseCallbacks>(&mut self, object: &Self, c: &mut C) {
        self.role = object.role;
        self.label.clone_from(&object.label);
        self.value.clone_from(&object.value);
        self.hint.clone_from(&object.hint);
        self.heading_level = object.heading_level;
        self.trait_flags = object.trait_flags;
        self.state_flags = object.state_flags;
        self.base.copy(&object.base, c);
    }
    pub fn deserialize<C: SemanticDataBaseCallbacks>(
        &mut self,
        key: u16,
        reader: &mut BinaryReader<'_>,
        c: &mut C,
    ) -> bool {
        match key {
            Self::ROLE_PROPERTY_KEY => {
                self.role = CoreUintType::deserialize(reader);
                true
            }
            Self::LABEL_PROPERTY_KEY => {
                self.label = CoreStringType::deserialize(reader);
                true
            }
            Self::VALUE_PROPERTY_KEY => {
                self.value = CoreStringType::deserialize(reader);
                true
            }
            Self::HINT_PROPERTY_KEY => {
                self.hint = CoreStringType::deserialize(reader);
                true
            }
            Self::HEADING_LEVEL_PROPERTY_KEY => {
                self.heading_level = CoreUintType::deserialize(reader);
                true
            }
            Self::TRAIT_FLAGS_PROPERTY_KEY => {
                self.trait_flags = CoreUintType::deserialize(reader);
                true
            }
            Self::STATE_FLAGS_PROPERTY_KEY => {
                self.state_flags = CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(key, reader, c),
        }
    }
}
