//! Nuxie-owned keyed row artboards: one more artboard a keyed list's rows
//! may be made from.
//!
//! A keyed list (an `ArtboardComponentList` whose record sets
//! `rowArtboardId`) makes its rows from that artboard. `KeyedRowArtboard`
//! (type 60003) is a child of the list and declares one more, by its index
//! in the file's artboards, as `rowArtboardId` names one. A host mounting a
//! row names which declared artboard it is made from. Upstream chooses each
//! row's artboard per item through `ArtboardListMapRule`, keyed by the
//! item's view model; a keyed list has no view models, so its rows choose
//! from the set the file declares. Under any other parent the record is
//! refused at import.
use crate::source::{
    artboard_component_list::ArtboardComponentList,
    component::Component,
    core::{
        Core, CoreObject, CoreType, PropertySetterCompletion,
        binary_reader::BinaryReader,
        field_types::{core_callback_type::CallbackData, core_uint_type::CoreUintType},
    },
    core_context::CoreContext,
    generated::{
        artboard_component_list_base::ArtboardComponentListBase,
        component_base::ComponentBase,
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject},
    },
    importers::import_stack::ImportStack,
    status_code::StatusCode,
};
use std::any::Any;

/// The `artboardId` property: the declared artboard's index in the file's
/// artboards.
pub const ARTBOARD_ID_PROPERTY_KEY: u16 = 60025;

pub struct KeyedRowArtboard {
    component: Component,
    artboard_id: u32,
}

impl Default for KeyedRowArtboard {
    fn default() -> Self {
        Self {
            component: Component::default(),
            artboard_id: u32::MAX,
        }
    }
}

impl KeyedRowArtboard {
    pub const TYPE_KEY: u16 = 60003;

    fn subtype(key: u16) -> bool {
        key == Self::TYPE_KEY || ComponentBase::is_type_of(key)
    }

    /// The declared artboard, as an index into the file's artboards.
    pub fn artboard_id(&self) -> u32 {
        self.artboard_id
    }

    fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.component.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(parent) = self.component.parent_handle() else {
            return StatusCode::MissingObject;
        };
        let artboard_id = self.artboard_id;
        parent
            .with_downcast_mut::<ArtboardComponentList, _>(|list| {
                list.declare_row_artboard(artboard_id)
            })
            .map_or(StatusCode::InvalidObject, |_| StatusCode::Ok)
    }
}

impl CoreType for KeyedRowArtboard {
    const TYPE_KEY: u16 = KeyedRowArtboard::TYPE_KEY;
}

impl CoreObject for KeyedRowArtboard {
    fn core(&self) -> &Core {
        &self.component.base.base
    }
    fn core_mut(&mut self) -> &mut Core {
        &mut self.component.base.base
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
        let mut cloned = Self {
            artboard_id: self.artboard_id,
            ..Self::default()
        };
        cloned
            .component
            .base
            .copy(&self.component.base, &mut Component::default());
        Some(Box::new(cloned))
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        if key == ARTBOARD_ID_PROPERTY_KEY {
            self.artboard_id = CoreUintType::deserialize(reader);
            return true;
        }
        let mut base = std::mem::take(&mut self.component.base);
        let handled = base.deserialize(key, reader, &mut self.component);
        self.component.base = base;
        handled
    }
}

impl CoreRegistryObject for KeyedRowArtboard {
    fn as_registry_any(&self) -> &dyn Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn set_uint_with_completion(
        &mut self,
        field: CoreField,
        value: u32,
        completion: &mut PropertySetterCompletion,
    ) {
        if field == CoreField::ComponentParentId && self.component.base.set_parent_id_value(value) {
            completion.record(
                CoreObject::core(self),
                ComponentBase::PARENT_ID_PROPERTY_KEY,
            );
        }
    }
    fn set_string_with_completion(
        &mut self,
        field: CoreField,
        value: String,
        completion: &mut PropertySetterCompletion,
    ) {
        if field == CoreField::ComponentName && self.component.base.set_name_value(value) {
            completion.record(CoreObject::core(self), ComponentBase::NAME_PROPERTY_KEY);
        }
    }
    fn set_color_with_completion(
        &mut self,
        _: CoreField,
        _: i32,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_bool_with_completion(
        &mut self,
        _: CoreField,
        _: bool,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_double_with_completion(
        &mut self,
        _: CoreField,
        _: f32,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_int_with_completion(&mut self, _: CoreField, _: i32, _: &mut PropertySetterCompletion) {}
    fn set_callback_with_completion(
        &mut self,
        _: CoreField,
        _: CallbackData<'_>,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn get_uint(&mut self, field: CoreField) -> u32 {
        match field {
            CoreField::ComponentParentId => self.component.base.parent_id(),
            _ => 0,
        }
    }
    fn get_string(&mut self, field: CoreField) -> String {
        match field {
            CoreField::ComponentName => self.component.base.name().to_owned(),
            _ => String::new(),
        }
    }
    fn get_color(&mut self, _: CoreField) -> i32 {
        0
    }
    fn get_bool(&mut self, _: CoreField) -> bool {
        false
    }
    fn get_double(&mut self, _: CoreField) -> f32 {
        0.0
    }
    fn get_int(&mut self, _: CoreField) -> i32 {
        0
    }
}

impl CoreCapabilities for KeyedRowArtboard {
    fn component_on_dirty(&mut self, _: crate::source::component_dirt::ComponentDirt) -> bool {
        true
    }
    fn component_on_dirty_is_noop(&self) -> bool {
        true
    }
    fn lifecycle_validate(&mut self, context: &mut dyn CoreContext) -> Option<bool> {
        Some(
            self.component.validate(context)
                && context
                    .resolve(self.component.base.parent_id())
                    .is_some_and(|parent| parent.is_type_of(ArtboardComponentListBase::TYPE_KEY)),
        )
    }
    fn lifecycle_on_added_dirty(&mut self, context: &mut dyn CoreContext) -> Option<StatusCode> {
        Some(self.on_added_dirty(context))
    }
    fn lifecycle_import(&mut self, stack: &mut ImportStack) -> Option<StatusCode> {
        Some(self.component.import(stack))
    }
    fn as_component(&self) -> Option<&Component> {
        Some(&self.component)
    }
    fn as_component_mut(&mut self) -> Option<&mut Component> {
        Some(&mut self.component)
    }
}
