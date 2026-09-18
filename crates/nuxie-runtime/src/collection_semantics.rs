//! Logical collection metadata on the existing upstream semantic owner.
use crate::source::{
    component::Component,
    component_dirt::ComponentDirt,
    core::{
        Core, CoreObject, CoreType, binary_reader::BinaryReader,
        field_types::core_callback_type::CallbackData,
    },
    core_context::CoreContext,
    generated::{
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject},
        semantic::semantic_data_base::SemanticDataBase,
    },
    importers::import_stack::ImportStack,
    semantic::{semantic_data::SemanticData, semantic_role::SemanticRole},
    status_code::StatusCode,
};
use std::any::Any;

pub struct SemanticCollectionData {
    semantic: SemanticData,
    item_count: u32,
    item_position: u32,
}
impl Default for SemanticCollectionData {
    fn default() -> Self {
        Self {
            semantic: SemanticData::default(),
            item_count: u32::MAX,
            item_position: u32::MAX,
        }
    }
}
impl SemanticCollectionData {
    pub const TYPE_KEY: u16 = 60002;
    pub fn item_count(&self) -> Option<u32> {
        (self.item_count != u32::MAX).then_some(self.item_count)
    }
    pub fn item_position(&self) -> Option<u32> {
        (self.item_position != u32::MAX).then_some(self.item_position)
    }
    fn subtype(key: u16) -> bool {
        key == Self::TYPE_KEY || SemanticDataBase::is_type_of(key)
    }
    fn valid_role(&self) -> bool {
        match self.semantic.base.role() {
            role if role == SemanticRole::List as u32 => self.item_position == u32::MAX,
            role if role == SemanticRole::ListItem as u32 => self.item_count == u32::MAX,
            _ => false,
        }
    }
}
impl CoreType for SemanticCollectionData {
    const TYPE_KEY: u16 = Self::TYPE_KEY;
}
impl CoreRegistryObject for SemanticCollectionData {
    fn as_registry_any(&self) -> &dyn Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn set_uint(&mut self, field: CoreField, value: u32) {
        self.semantic.set_uint(field, value);
    }
    fn get_uint(&mut self, field: CoreField) -> u32 {
        self.semantic.get_uint(field)
    }
    fn set_string(&mut self, field: CoreField, value: String) {
        self.semantic.set_string(field, value);
    }
    fn get_string(&mut self, field: CoreField) -> String {
        self.semantic.get_string(field)
    }
    fn set_color(&mut self, field: CoreField, value: i32) {
        self.semantic.set_color(field, value);
    }
    fn get_color(&mut self, field: CoreField) -> i32 {
        self.semantic.get_color(field)
    }
    fn set_bool(&mut self, field: CoreField, value: bool) {
        self.semantic.set_bool(field, value);
    }
    fn get_bool(&mut self, field: CoreField) -> bool {
        self.semantic.get_bool(field)
    }
    fn set_double(&mut self, field: CoreField, value: f32) {
        self.semantic.set_double(field, value);
    }
    fn get_double(&mut self, field: CoreField) -> f32 {
        self.semantic.get_double(field)
    }
    fn set_int(&mut self, field: CoreField, value: i32) {
        self.semantic.set_int(field, value);
    }
    fn get_int(&mut self, field: CoreField) -> i32 {
        self.semantic.get_int(field)
    }
    fn set_callback(&mut self, field: CoreField, value: CallbackData<'_>) {
        self.semantic.set_callback(field, value);
    }
}
impl CoreObject for SemanticCollectionData {
    fn core(&self) -> &Core {
        CoreObject::core(&self.semantic)
    }
    fn core_mut(&mut self) -> &mut Core {
        CoreObject::core_mut(&mut self.semantic)
    }
    fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        Self::subtype
    }
    fn clone_boxed(&self) -> Option<Box<dyn CoreObject>> {
        let mut cloned = Self::default();
        let mut base = std::mem::take(&mut cloned.semantic.base);
        base.copy(&self.semantic.base, &mut cloned.semantic);
        cloned.semantic.base = base;
        cloned.item_count = self.item_count;
        cloned.item_position = self.item_position;
        Some(Box::new(cloned))
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        match key {
            60016 => self.item_count = reader.read_var_uint_as::<u32>(),
            60017 => self.item_position = reader.read_var_uint_as::<u32>(),
            _ => return self.semantic.deserialize(key, reader),
        }
        true
    }
}
impl CoreCapabilities for SemanticCollectionData {
    fn as_component(&self) -> Option<&Component> {
        self.semantic.as_component()
    }
    fn as_component_mut(&mut self) -> Option<&mut Component> {
        self.semantic.as_component_mut()
    }
    fn as_semantic_data(&self) -> Option<&SemanticData> {
        Some(&self.semantic)
    }
    fn as_semantic_data_mut(&mut self) -> Option<&mut SemanticData> {
        Some(&mut self.semantic)
    }
    fn component_update(&mut self, dirt: ComponentDirt) -> bool {
        self.semantic.component_update(dirt)
    }
    fn component_build_dependencies(&mut self) -> bool {
        self.semantic.component_build_dependencies()
    }
    fn component_collapse_post(&mut self, value: bool) -> bool {
        self.semantic.component_collapse_post(value)
    }
    fn component_collapse_after_container(&mut self, value: bool) -> bool {
        self.semantic.component_collapse_after_container(value)
    }
    fn lifecycle_validate(&mut self, context: &mut dyn CoreContext) -> Option<bool> {
        Some(self.valid_role() && self.semantic.lifecycle_validate(context).unwrap_or(false))
    }
    fn lifecycle_on_added_dirty(&mut self, context: &mut dyn CoreContext) -> Option<StatusCode> {
        self.semantic.lifecycle_on_added_dirty(context)
    }
    fn lifecycle_import(&mut self, stack: &mut ImportStack) -> Option<StatusCode> {
        if !self.valid_role() {
            return Some(StatusCode::InvalidObject);
        }
        self.semantic.lifecycle_import(stack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{core::CoreArena, semantic::semantic_listener::SemanticListener};
    use std::{cell::Cell, rc::Rc};

    fn decoded(role: u8, key: u16, bytes: &[u8]) -> Box<dyn CoreObject> {
        let mut object = crate::scene_objects::make_core(60002).unwrap();
        assert!(object.deserialize(982, &mut BinaryReader::new(&[role])));
        let mut reader = BinaryReader::new(bytes);
        assert!(object.deserialize(key, &mut reader));
        assert!(!reader.has_error());
        object
    }

    #[test]
    fn factory_preserves_unknown_empty_and_first_item_distinctions() {
        let default = SemanticCollectionData::default();
        assert_eq!(default.item_count(), None);
        assert_eq!(default.item_position(), None);
        for (role, key) in [(10, 60016), (11, 60017)] {
            let object = decoded(role, key, &[0]);
            let data = object
                .as_registry_any()
                .downcast_ref::<SemanticCollectionData>()
                .unwrap();
            assert!(data.valid_role());
            assert_eq!(data.item_count(), (role == 10).then_some(0));
            assert_eq!(data.item_position(), (role == 11).then_some(0));
            assert_eq!(
                object.as_semantic_data().unwrap().base.role(),
                u32::from(role)
            );
            assert!(CoreObject::is_type_of(object.as_ref(), 668));
        }
    }

    #[test]
    fn role_validation_rejects_metadata_on_the_wrong_owner() {
        for (role, key) in [(1, 60016), (10, 60017), (11, 60016)] {
            let object = decoded(role, key, &[0]);
            assert!(
                !object
                    .as_registry_any()
                    .downcast_ref::<SemanticCollectionData>()
                    .unwrap()
                    .valid_role()
            );
        }
    }

    #[test]
    fn occurrence_clone_preserves_metadata_and_upstream_label() {
        let mut object = decoded(10, 60016, &[10]);
        object.set_string(CoreField::SemanticDataLabel, "Plans".into());
        let cloned = object.clone_boxed().unwrap();
        assert_eq!(cloned.core_type(), 60002);
        assert_eq!(
            cloned
                .as_registry_any()
                .downcast_ref::<SemanticCollectionData>()
                .unwrap()
                .item_count(),
            Some(10)
        );
        assert_eq!(cloned.as_semantic_data().unwrap().base.label(), "Plans");
        object.set_string(CoreField::SemanticDataLabel, "Changed".into());
        assert_eq!(cloned.as_semantic_data().unwrap().base.label(), "Plans");
    }

    #[derive(Debug)]
    struct Listener(Cell<usize>);
    impl SemanticListener for Listener {
        fn supports_semantic_action(&self, action: u8) -> bool {
            action == 0
        }
        fn on_semantic_tap(&self) {
            self.0.set(self.0.get() + 1);
        }
        fn on_semantic_increase(&self) {}
        fn on_semantic_decrease(&self) {}
    }

    #[test]
    fn semantic_capability_keeps_listeners_and_focus_on_the_upstream_owner() {
        let arena = CoreArena::default();
        let handle = arena.insert_boxed(decoded(11, 60017, &[2]));
        let listener = Rc::new(Listener(Cell::new(0)));
        handle
            .with_semantic_data_mut(|data| {
                data.add_semantic_listener(listener.clone());
                data.semantic_node();
                data.set_focused_state(true);
            })
            .unwrap();
        handle
            .with_semantic_data(|data| {
                assert_ne!(
                    data.existing_semantic_node().unwrap().borrow().state_flags & (1 << 7),
                    0
                );
                assert!(data.supports_semantic_action(0));
                data.fire_semantic_tap();
            })
            .unwrap();
        assert_eq!(listener.0.get(), 1);
    }
}
