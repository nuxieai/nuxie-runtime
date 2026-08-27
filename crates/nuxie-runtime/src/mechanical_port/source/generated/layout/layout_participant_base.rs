use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, layout::layout_node_style::LayoutNodeStyle,
    layout::layout_participant::LayoutParticipant,
};

pub struct LayoutParticipantBase {
    pub base: LayoutNodeStyle,
}

impl Default for LayoutParticipantBase {
    fn default() -> Self {
        Self {
            base: LayoutNodeStyle::default(),
        }
    }
}

impl LayoutParticipantBase {
    pub const TYPE_KEY: u16 = 1066;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 1057 | 1056 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> LayoutParticipant {
        let mut cloned = LayoutParticipant::default();
        cloned.base.copy(self);
        cloned
    }
}
