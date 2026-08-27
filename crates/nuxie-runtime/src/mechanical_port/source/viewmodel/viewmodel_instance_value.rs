use std::ptr::NonNull;

use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core_context::CoreContext,
    generated::viewmodel::viewmodel_instance_value_base::ViewModelInstanceValueBase,
    importers::{
        artboard_importer::ArtboardImporter, import_stack::ImportStack,
        viewmodel_instance_importer::ViewModelInstanceImporter,
    },
    refcnt::RiveRc,
    status_code::StatusCode,
};

use super::{
    symbol_type::SymbolType, viewmodel_instance::ViewModelInstance,
    viewmodel_property::ViewModelProperty, viewmodel_value_dependent::ViewModelValueDependent,
};

pub trait ViewModelInstanceValueDelegate {
    fn value_changed(&mut self);
}

#[repr(u8)]
#[derive(Clone, Copy, Default)]
enum ValueFlags {
    #[default]
    None = 0,
    ValueChanged = 1 << 1,
    DelegatesChanged = 1 << 2,
    Delegating = 1 << 3,
}

pub struct ViewModelInstanceValue {
    pub base: ViewModelInstanceValueBase,
    view_model_property: Option<NonNull<ViewModelProperty>>,
    change_flags: u8,
    delegates: Vec<NonNull<dyn ViewModelInstanceValueDelegate>>,
    delegates_copy: Vec<NonNull<dyn ViewModelInstanceValueDelegate>>,
    dependents: Vec<NonNull<dyn ViewModelValueDependent>>,
    view_model_instance: Option<NonNull<ViewModelInstance>>,
}

impl ViewModelInstanceValue {
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let result = self.base.on_added_dirty(context);
        if result != StatusCode::Ok {
            return result;
        }
        if let Some(mut parent) = self.base.parent_as_view_model_instance() {
            unsafe { parent.as_mut() }.add_value(NonNull::from(&mut *self));
        }
        StatusCode::Ok
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = import_stack.latest::<ViewModelInstanceImporter>(
            crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_base::ViewModelInstanceBase::TYPE_KEY,
        ) else {
            return StatusCode::MissingObject;
        };
        importer.add_value(NonNull::from(&mut *self));
        if import_stack
            .latest::<ArtboardImporter>(
                crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
            )
            .is_some()
        {
            self.base.import(import_stack)
        } else {
            self.base.core_import(import_stack)
        }
    }

    pub fn has_changed(&self) -> bool {
        self.has_flag(ValueFlags::ValueChanged)
    }

    fn register_symbol(&mut self) {
        let (Some(property), Some(mut instance)) =
            (self.view_model_property, self.view_model_instance)
        else {
            return;
        };
        let property = unsafe { property.as_ref() };
        if property.is_symbol_list_index() {
            unsafe { instance.as_mut() }
                .set_property_symbol(SymbolType::ItemIndex, NonNull::from(&mut *self));
        } else if let Some(symbol) = SymbolType::from_i32(property.base.symbol_type_value()) {
            if symbol != SymbolType::None {
                unsafe { instance.as_mut() }.set_property_symbol(symbol, NonNull::from(&mut *self));
            }
        }
    }

    pub fn set_view_model_property(&mut self, value: NonNull<ViewModelProperty>) {
        self.view_model_property = Some(value);
        self.register_symbol();
    }

    pub fn view_model_property(&self) -> Option<NonNull<ViewModelProperty>> {
        self.view_model_property
    }

    pub fn add_dependent(&mut self, value: NonNull<dyn ViewModelValueDependent>) {
        self.dependents.push(value);
    }

    pub fn remove_dependent(&mut self, value: NonNull<dyn ViewModelValueDependent>) {
        self.dependents
            .retain(|candidate| !std::ptr::addr_eq(candidate.as_ptr(), value.as_ptr()));
    }

    pub fn add_dirt(&mut self, value: ComponentDirt) {
        for dependent in &mut self.dependents {
            unsafe { dependent.as_mut() }.add_dirt(value, true);
        }
    }

    pub fn set_root(&mut self, _value: RiveRc<ViewModelInstance>) {}

    pub fn name(&self) -> &str {
        self.view_model_property
            .map(|property| unsafe { property.as_ref() }.const_name())
            .unwrap_or("")
    }

    pub fn advanced(&mut self) {
        self.base.used_layers_mut().clear();
        self.clear_flag(ValueFlags::ValueChanged);
    }

    pub fn set_view_model_instance(&mut self, value: NonNull<ViewModelInstance>) {
        self.view_model_instance = Some(value);
        self.register_symbol();
    }

    pub fn view_model_instance(&self) -> Option<NonNull<ViewModelInstance>> {
        self.view_model_instance
    }

    pub fn add_delegate(&mut self, delegate: NonNull<dyn ViewModelInstanceValueDelegate>) {
        self.delegates.push(delegate);
        self.set_flag(ValueFlags::DelegatesChanged);
    }

    pub fn remove_delegate(&mut self, delegate: NonNull<dyn ViewModelInstanceValueDelegate>) {
        let old_len = self.delegates.len();
        self.delegates
            .retain(|candidate| !std::ptr::addr_eq(candidate.as_ptr(), delegate.as_ptr()));
        if self.delegates.len() != old_len {
            self.set_flag(ValueFlags::DelegatesChanged);
        }
    }

    fn suppress_delegation(&mut self) -> bool {
        if self.has_flag(ValueFlags::Delegating) {
            return false;
        }
        self.set_flag(ValueFlags::Delegating);
        true
    }

    fn restore_delegation(&mut self) {
        self.clear_flag(ValueFlags::Delegating);
    }

    pub fn on_value_changed(&mut self) {
        self.set_flag(ValueFlags::ValueChanged);
        if self.delegates.is_empty() {
            return;
        }
        if self.has_flag(ValueFlags::DelegatesChanged) {
            self.delegates_copy.clone_from(&self.delegates);
            self.clear_flag(ValueFlags::DelegatesChanged);
        }
        if self.has_flag(ValueFlags::Delegating) {
            return;
        }
        self.set_flag(ValueFlags::Delegating);
        for delegate in &mut self.delegates_copy {
            unsafe { delegate.as_mut() }.value_changed();
        }
        self.clear_flag(ValueFlags::Delegating);
    }

    pub fn dependents(&self) -> &[NonNull<dyn ViewModelValueDependent>] {
        &self.dependents
    }

    fn has_flag(&self, flag: ValueFlags) -> bool {
        self.change_flags & flag as u8 != 0
    }

    fn set_flag(&mut self, flag: ValueFlags) {
        self.change_flags |= flag as u8;
    }

    fn clear_flag(&mut self, flag: ValueFlags) {
        self.change_flags &= !(flag as u8);
    }
}

pub struct SuppressDelegation {
    value: NonNull<ViewModelInstanceValue>,
    suppressed: bool,
}

impl SuppressDelegation {
    pub fn new(mut value: NonNull<ViewModelInstanceValue>) -> Self {
        let suppressed = unsafe { value.as_mut() }.suppress_delegation();
        Self { value, suppressed }
    }
}

impl Drop for SuppressDelegation {
    fn drop(&mut self) {
        if self.suppressed {
            unsafe { self.value.as_mut() }.restore_delegation();
        }
    }
}
