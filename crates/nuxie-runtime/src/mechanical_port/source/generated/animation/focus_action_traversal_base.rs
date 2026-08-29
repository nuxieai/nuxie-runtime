use crate::mechanical_port::source::{
    animation::focus_action::FocusAction, animation::focus_action_traversal::FocusActionTraversal,
    core::binary_reader::BinaryReader,
};

pub trait FocusActionTraversalBaseCallbacks: crate::mechanical_port::source::generated::animation::listener_action_base::ListenerActionBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn traversal_kind_changed(&mut self) {}
}

pub struct FocusActionTraversalBase {
    pub base: FocusAction,
    traversal_kind: u32,
}

impl Default for FocusActionTraversalBase {
    fn default() -> Self {
        Self {
            base: FocusAction::default(),
            traversal_kind: 0,
        }
    }
}

impl FocusActionTraversalBase {
    pub const TYPE_KEY: u16 = 672;
    pub const TRAVERSAL_KIND_PROPERTY_KEY: u16 = 1011;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 671 | 125)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn traversal_kind(&self) -> u32 {
        self.traversal_kind
    }
    pub fn set_traversal_kind(
        &mut self,
        value: u32,
        callbacks: &mut impl FocusActionTraversalBaseCallbacks,
    ) {
        if !self.set_traversal_kind_value(value) {
            return;
        }
        callbacks.traversal_kind_changed();
        FocusActionTraversalBaseCallbacks::notify_property_changed(
            callbacks,
            Self::TRAVERSAL_KIND_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_traversal_kind_value(&mut self, value: u32) -> bool {
        if self.traversal_kind == value {
            return false;
        }
        self.traversal_kind = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl FocusActionTraversalBaseCallbacks,
    ) -> FocusActionTraversal {
        let mut cloned = FocusActionTraversal::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl FocusActionTraversalBaseCallbacks) {
        self.traversal_kind = object.traversal_kind;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl FocusActionTraversalBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::TRAVERSAL_KIND_PROPERTY_KEY => {
                self.traversal_kind = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for FocusActionTraversalBase {
    type Target = FocusAction;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FocusActionTraversalBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
