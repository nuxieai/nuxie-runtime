use crate::mechanical_port::source::{
    animation::listener_types::listener_input_type::ListenerInputType,
    animation::listener_types::listener_input_type_semantic::ListenerInputTypeSemantic,
    core::binary_reader::BinaryReader,
};

pub struct ListenerInputTypeSemanticBase {
    pub base: ListenerInputType,
}

impl Default for ListenerInputTypeSemanticBase {
    fn default() -> Self {
        Self {
            base: ListenerInputType::default(),
        }
    }
}

impl ListenerInputTypeSemanticBase {
    pub const TYPE_KEY: u16 = 669;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 658)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ListenerInputTypeSemantic {
        let mut cloned = ListenerInputTypeSemantic::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for ListenerInputTypeSemanticBase {
    type Target = ListenerInputType;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ListenerInputTypeSemanticBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
