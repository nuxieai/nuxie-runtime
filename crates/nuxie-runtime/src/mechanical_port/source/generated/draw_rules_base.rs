use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader,
    draw_rules::DrawRules,
};

pub trait DrawRulesBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn draw_target_id_changed(&mut self) {}
}

pub struct DrawRulesBase {
    pub base: ContainerComponent,
    draw_target_id: u32,
}

impl Default for DrawRulesBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
            draw_target_id: u32::MAX,
        }
    }
}

impl DrawRulesBase {
    pub const TYPE_KEY: u16 = 49;
    pub const DRAW_TARGET_ID_PROPERTY_KEY: u16 = 121;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn draw_target_id(&self) -> u32 {
        self.draw_target_id
    }
    pub fn set_draw_target_id(&mut self, value: u32, callbacks: &mut impl DrawRulesBaseCallbacks) {
        if self.draw_target_id == value {
            return;
        }
        self.draw_target_id = value;
        callbacks.draw_target_id_changed();
        callbacks.notify_property_changed(Self::DRAW_TARGET_ID_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl DrawRulesBaseCallbacks) -> DrawRules {
        let mut cloned = DrawRules::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl DrawRulesBaseCallbacks) {
        self.draw_target_id = object.draw_target_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DrawRulesBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::DRAW_TARGET_ID_PROPERTY_KEY => {
                self.draw_target_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
