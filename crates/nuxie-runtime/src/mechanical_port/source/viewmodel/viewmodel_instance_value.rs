use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::mechanical_port::source::{
    animation::state_machine_instance::RuntimeStateMachineLayerInstanceWeakHandle,
    component_dirt::ComponentDirt,
    core::CoreHandle,
    core_context::CoreContext,
    data_bind::data_bind_context::BoundSource,
    generated::viewmodel::viewmodel_instance_value_base::ViewModelInstanceValueBase,
    importers::{
        artboard_importer::ArtboardImporter, import_stack::ImportStack,
        viewmodel_instance_importer::ViewModelInstanceImporter,
    },
    status_code::StatusCode,
};

use super::{
    symbol_type::SymbolType, viewmodel_instance_value::ValueDependentHandle::Runtime,
    viewmodel_value_dependent::ViewModelValueDependent,
};

pub trait ViewModelInstanceValueDelegate {
    fn value_changed(&mut self);
}

pub type ViewModelInstanceValueDelegateHandle = Rc<RefCell<dyn ViewModelInstanceValueDelegate>>;
type ViewModelInstanceValueDelegateWeakHandle = Weak<RefCell<dyn ViewModelInstanceValueDelegate>>;

#[derive(Clone)]
pub enum ValueDependentHandle {
    Core(CoreHandle),
    Runtime(Weak<RefCell<dyn ViewModelValueDependent>>),
}

impl ValueDependentHandle {
    pub fn core(value: CoreHandle) -> Self {
        Self::Core(value)
    }

    pub fn runtime(value: &Rc<RefCell<dyn ViewModelValueDependent>>) -> Self {
        Runtime(Rc::downgrade(value))
    }

    fn same_identity(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Core(left), Self::Core(right)) => left == right,
            (Self::Runtime(left), Self::Runtime(right)) => Weak::ptr_eq(left, right),
            _ => false,
        }
    }

    fn add_dirt(&self, value: ComponentDirt) {
        match self {
            Self::Core(dependent) => {
                dependent.with_mut(|dependent| {
                    if let Some(dependent) = dependent.as_data_bind_mut() {
                        dependent.add_dirt(u32::from(value.0), true);
                    } else if let Some(formula) = dependent.as_any_mut().downcast_mut::<crate::mechanical_port::source::data_bind::converters::data_converter_formula::DataConverterFormula>() {
                        formula.add_dirt(u32::from(value.0), true);
                    }
                });
            }
            Self::Runtime(dependent) => {
                if let Some(dependent) = dependent.upgrade() {
                    dependent.borrow_mut().add_dirt(value, true);
                }
            }
        }
    }

    pub(crate) fn relink(&self) {
        let dependent = self.clone();
        if crate::view_model_cell::defer_transaction_notification(move || dependent.relink()) {
            return;
        }
        match self {
            Self::Core(dependent) => {
                crate::mechanical_port::source::data_bind::data_bind::DataBind::relink_handle(
                    dependent,
                );
            }
            Self::Runtime(dependent) => {
                if let Some(dependent) = dependent.upgrade() {
                    dependent.borrow_mut().relink_data_bind();
                }
            }
        }
    }

    fn is_alive(&self) -> bool {
        match self {
            Self::Core(value) => value.is_alive(),
            Self::Runtime(value) => value.strong_count() != 0,
        }
    }
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
    view_model_property: Option<CoreHandle>,
    change_flags: u8,
    delegates: Vec<ViewModelInstanceValueDelegateWeakHandle>,
    delegates_copy: Vec<ViewModelInstanceValueDelegateWeakHandle>,
    dependents: Vec<ValueDependentHandle>,
    view_model_instance: Option<CoreHandle>,
    used_layers: Vec<RuntimeStateMachineLayerInstanceWeakHandle>,
}

#[derive(Clone)]
pub(crate) struct HostValueState {
    value_changed: bool,
    used_layers: Vec<RuntimeStateMachineLayerInstanceWeakHandle>,
}

impl std::ops::Deref for ViewModelInstanceValue {
    type Target = ViewModelInstanceValueBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ViewModelInstanceValue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Default for ViewModelInstanceValue {
    fn default() -> Self {
        Self {
            base: ViewModelInstanceValueBase::default(),
            view_model_property: None,
            change_flags: 0,
            delegates: Vec::new(),
            delegates_copy: Vec::new(),
            dependents: Vec::new(),
            view_model_instance: None,
            used_layers: Vec::new(),
        }
    }
}

impl ViewModelInstanceValue {
    fn handle(&self) -> Option<CoreHandle> {
        self.base.base.base.base.handle()
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let result = self.base.on_added_dirty(context);
        if result != StatusCode::Ok {
            return result;
        }
        StatusCode::Ok
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(value) = self.handle() else {
            return StatusCode::MissingObject;
        };
        let Some(importer) = import_stack.latest::<ViewModelInstanceImporter>(
            crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_base::ViewModelInstanceBase::TYPE_KEY,
        ) else {
            return StatusCode::MissingObject;
        };
        importer.add_value(value);
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
        let (Some(property), Some(instance), Some(value)) = (
            self.view_model_property.as_ref(),
            self.view_model_instance.as_ref(),
            self.handle(),
        ) else {
            return;
        };
        let (is_list_index, symbol) = property
            .with(|property| {
                let property = property.as_view_model_property()?;
                Some((
                    property.is_symbol_list_index(),
                    SymbolType::from_i32(property.base.symbol_type_value()),
                ))
            })
            .flatten()
            .unwrap_or((false, None));
        let symbol = if is_list_index {
            Some(SymbolType::ItemIndex)
        } else {
            symbol.filter(|symbol| *symbol != SymbolType::None)
        };
        if let Some(symbol) = symbol {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    instance.set_property_symbol(symbol, value);
                }
            });
        }
    }

    pub fn set_view_model_property(&mut self, value: CoreHandle) {
        self.view_model_property = Some(value);
        self.register_symbol();
    }

    pub fn view_model_property(&self) -> Option<CoreHandle> {
        self.view_model_property.clone()
    }

    pub fn add_dependent(&mut self, value: ValueDependentHandle) {
        if !self
            .dependents
            .iter()
            .any(|candidate| candidate.same_identity(&value))
        {
            self.dependents.push(value);
        }
    }

    pub fn remove_dependent(&mut self, value: &ValueDependentHandle) {
        self.dependents
            .retain(|candidate| !candidate.same_identity(value));
    }

    pub fn add_dirt(&mut self, value: ComponentDirt) {
        self.dependents.retain(ValueDependentHandle::is_alive);
        let dependents = self.dependents.clone();
        if crate::view_model_cell::defer_transaction_notification(move || {
            for dependent in dependents {
                dependent.add_dirt(value);
            }
        }) {
            return;
        }
        for dependent in &self.dependents {
            dependent.add_dirt(value);
        }
    }

    pub fn relink_dependents(&mut self) {
        self.dependents.retain(ValueDependentHandle::is_alive);
        let dependents = self.dependents.clone();
        if crate::view_model_cell::defer_transaction_notification(move || {
            for dependent in dependents {
                dependent.relink();
            }
        }) {
            return;
        }
        for dependent in &self.dependents {
            dependent.relink();
        }
    }

    pub fn set_root(&mut self, _value: CoreHandle) {}

    pub fn name(&self) -> String {
        self.view_model_property
            .as_ref()
            .and_then(|property| {
                property.with(|property| {
                    property
                        .as_view_model_property()
                        .map(|property| property.const_name().to_owned())
                })
            })
            .flatten()
            .unwrap_or_default()
    }

    pub fn advanced(&mut self) {
        self.used_layers.clear();
        self.clear_flag(ValueFlags::ValueChanged);
    }

    pub fn is_used_in_layer(&self, layer: &RuntimeStateMachineLayerInstanceWeakHandle) -> bool {
        self.used_layers.iter().any(|used| used.ptr_eq(layer))
    }

    pub fn use_in_layer(&mut self, layer: RuntimeStateMachineLayerInstanceWeakHandle) {
        if !self.is_used_in_layer(&layer) {
            self.used_layers.push(layer);
        }
    }

    pub fn set_view_model_instance(&mut self, value: CoreHandle) {
        self.view_model_instance = Some(value);
        self.register_symbol();
    }

    pub fn view_model_instance(&self) -> Option<CoreHandle> {
        self.view_model_instance.clone()
    }

    pub fn add_delegate(&mut self, delegate: &ViewModelInstanceValueDelegateHandle) {
        self.delegates.push(Rc::downgrade(delegate));
        self.set_flag(ValueFlags::DelegatesChanged);
    }

    pub fn remove_delegate(&mut self, delegate: &ViewModelInstanceValueDelegateHandle) {
        let identity = Rc::downgrade(delegate);
        let old_len = self.delegates.len();
        self.delegates
            .retain(|candidate| !Weak::ptr_eq(candidate, &identity));
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
        self.delegates
            .retain(|delegate| delegate.strong_count() != 0);
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
        let delegates = self.delegates_copy.clone();
        let owner = self.handle();
        if crate::view_model_cell::defer_transaction_notification(move || {
            let _suppress = owner.map(SuppressDelegation::new);
            for delegate in delegates {
                if let Some(delegate) = delegate.upgrade() {
                    delegate.borrow_mut().value_changed();
                }
            }
        }) {
            return;
        }
        self.set_flag(ValueFlags::Delegating);
        for delegate in &self.delegates_copy {
            if let Some(delegate) = delegate.upgrade() {
                delegate.borrow_mut().value_changed();
            }
        }
        self.clear_flag(ValueFlags::Delegating);
    }

    pub fn dependents(&self) -> Vec<ValueDependentHandle> {
        self.dependents.clone()
    }

    pub(crate) fn host_snapshot(&self) -> HostValueState {
        HostValueState {
            value_changed: self.has_flag(ValueFlags::ValueChanged),
            used_layers: self.used_layers.clone(),
        }
    }

    pub(crate) fn restore_host_snapshot(&mut self, state: HostValueState) {
        if state.value_changed {
            self.set_flag(ValueFlags::ValueChanged);
        } else {
            self.clear_flag(ValueFlags::ValueChanged);
        }
        self.used_layers = state.used_layers;
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

impl BoundSource for ViewModelInstanceValue {}

macro_rules! impl_bind_source {
    ($owner:path, $data_type:expr) => {
        impl crate::mechanical_port::source::data_bind::data_bind::BindSource for $owner {
            fn data_type(
                &self,
            ) -> crate::mechanical_port::source::data_bind::data_values::data_type::DataType {
                $data_type
            }
        }
    };
}

use crate::mechanical_port::source::data_bind::data_values::data_type::DataType;

impl_bind_source!(
    super::viewmodel_instance_artboard::ViewModelInstanceArtboard,
    DataType::Artboard
);
impl_bind_source!(
    super::viewmodel_instance_asset_blob::ViewModelInstanceAssetBlob,
    DataType::AssetBlob
);
impl_bind_source!(
    super::viewmodel_instance_asset_font::ViewModelInstanceAssetFont,
    DataType::AssetFont
);
impl_bind_source!(
    super::viewmodel_instance_asset_image::ViewModelInstanceAssetImage,
    DataType::AssetImage
);
impl_bind_source!(
    super::viewmodel_instance_boolean::ViewModelInstanceBoolean,
    DataType::Boolean
);
impl_bind_source!(
    super::viewmodel_instance_color::ViewModelInstanceColor,
    DataType::Color
);
impl_bind_source!(
    super::viewmodel_instance_enum::ViewModelInstanceEnum,
    DataType::Enum
);
impl_bind_source!(
    super::viewmodel_instance_list::ViewModelInstanceList,
    DataType::List
);
impl_bind_source!(
    super::viewmodel_instance_number::ViewModelInstanceNumber,
    DataType::Number
);
impl_bind_source!(
    super::viewmodel_instance_string::ViewModelInstanceString,
    DataType::String
);
impl_bind_source!(
    super::viewmodel_instance_symbol_list_index::ViewModelInstanceSymbolListIndex,
    DataType::SymbolListIndex
);
impl_bind_source!(
    super::viewmodel_instance_trigger::ViewModelInstanceTrigger,
    DataType::Trigger
);
impl_bind_source!(
    super::viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
    DataType::ViewModel
);

pub struct SuppressDelegation {
    value: CoreHandle,
    suppressed: bool,
}

impl SuppressDelegation {
    pub fn new(value: CoreHandle) -> Self {
        let suppressed = value
            .with_mut(|value| {
                value
                    .as_view_model_instance_value_mut()
                    .is_some_and(ViewModelInstanceValue::suppress_delegation)
            })
            .unwrap_or(false);
        Self { value, suppressed }
    }
}

impl Drop for SuppressDelegation {
    fn drop(&mut self) {
        if self.suppressed {
            self.value.with_mut(|value| {
                if let Some(value) = value.as_view_model_instance_value_mut() {
                    value.restore_delegation();
                }
            });
        }
    }
}
