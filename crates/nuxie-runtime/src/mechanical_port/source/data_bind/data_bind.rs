use crate::mechanical_port::source::{
    core::{CoreArena, CoreHandle},
    data_bind::data_bind_container::DataBindContainerOwner,
    data_bind::data_values::data_type::DataType,
    data_bind_flags::DataBindFlags,
    file::RuntimeFileWeakHandle,
    generated::{
        animation::{
            state_transition_base::StateTransitionBase,
            transition_property_viewmodel_comparator_base::TransitionPropertyViewModelComparatorBase,
        },
        constraints::scrolling::scroll_constraint_base::ScrollConstraintBase,
        data_bind::{
            bindable_property_artboard_base::BindablePropertyArtboardBase,
            bindable_property_asset_base::BindablePropertyAssetBase,
            bindable_property_boolean_base::BindablePropertyBooleanBase,
            bindable_property_color_base::BindablePropertyColorBase,
            bindable_property_enum_base::BindablePropertyEnumBase,
            bindable_property_integer_base::BindablePropertyIntegerBase,
            bindable_property_list_base::BindablePropertyListBase,
            bindable_property_number_base::BindablePropertyNumberBase,
            bindable_property_string_base::BindablePropertyStringBase,
            bindable_property_trigger_base::BindablePropertyTriggerBase,
            bindable_property_viewmodel_base::BindablePropertyViewModelBase,
            data_bind_base::{DataBindBase, DataBindBaseCallbacks},
        },
        layout::layout_sizing_style_base::LayoutSizingStyleBase,
        node_base::NodeBase,
        shapes::{paint::color_channels_base::ColorChannelsBase, shape_base::ShapeBase},
        solo_base::SoloBase,
        viewmodel::viewmodel_instance_viewmodel_base::ViewModelInstanceViewModelBase,
    },
    status_code::StatusCode,
};
use std::{
    cell::{Cell, RefCell, RefMut},
    rc::Rc,
};

pub const DEPENDENTS: u32 =
    crate::mechanical_port::source::component_dirt::ComponentDirt::DEPENDENTS.0 as u32;
pub const BINDINGS: u32 =
    crate::mechanical_port::source::component_dirt::ComponentDirt::BINDINGS.0 as u32;
pub const BINDINGS_TARGET: u32 =
    crate::mechanical_port::source::component_dirt::ComponentDirt::BINDINGS_TARGET.0 as u32;
pub const TO_SOURCE: u32 = DataBindFlags::TO_SOURCE.0 as u32;
pub const TWO_WAY: u32 = DataBindFlags::TWO_WAY.0 as u32;
pub const DIRECTION: u32 = DataBindFlags::DIRECTION.0 as u32;
pub const SOURCE_TO_TARGET_FIRST: u32 = DataBindFlags::SOURCE_TO_TARGET_RUNS_FIRST.0 as u32;
pub const ONCE: u32 = DataBindFlags::ONCE.0 as u32;
pub const NAME_BASED: u32 = DataBindFlags::NAME_BASED.0 as u32;

const COLLAPSED: u8 = 1;
const IN_DIRTY: u8 = 2;
const IN_PERSISTING: u8 = 4;
const SUPPRESS_DIRT: u8 = 8;
const OBSERVING: u8 = 16;
const TARGET_ORIGIN: u8 = 32;
const INSTANCE_VALUE_BIND: u8 = 64;

pub trait BindScriptInput {
    fn scripted_object(&self) -> Option<CoreHandle>;
    fn set_scripted_object(&mut self, object: Option<CoreHandle>);
    fn data_bind(&self) -> Option<CoreHandle>;
    fn set_data_bind(&mut self, bind: Option<CoreHandle>, owns_data_bind: bool);
}

pub trait BindSource {
    fn data_type(&self) -> DataType;
}

pub trait BindConverter {
    fn output_type(&self) -> DataType;
    fn reset(&mut self);
    fn unbind(&mut self);
    fn update(&mut self);
    fn advance(&mut self, elapsed: f32) -> bool;
}

pub trait BindContextValue {
    fn invalidation_handle(&self) -> Rc<Cell<bool>>;
    fn apply(
        &mut self,
        target: Option<CoreHandle>,
        property_key: u32,
        is_main: bool,
        bind: CoreHandle,
    );
    fn refresh_target_value(&mut self, bind: CoreHandle);
    fn invalidate(&mut self);
    fn apply_to_source(
        &mut self,
        target: CoreHandle,
        property_key: u32,
        is_main: bool,
        bind: CoreHandle,
    );
}

#[derive(Clone)]
struct RuntimeBindContextValue {
    value: Rc<RefCell<Box<dyn BindContextValue>>>,
    valid: Rc<Cell<bool>>,
}

impl RuntimeBindContextValue {
    fn new(value: Box<dyn BindContextValue>) -> Self {
        let valid = value.invalidation_handle();
        Self {
            value: Rc::new(RefCell::new(value)),
            valid,
        }
    }
    fn borrow_mut(&self) -> RefMut<'_, Box<dyn BindContextValue>> {
        self.value.borrow_mut()
    }
    fn invalidate(&self) {
        self.valid.set(false);
    }
}

pub trait BindContainer {
    fn add_dirty_data_bind(&mut self, bind: CoreHandle);
    fn rebuild_data_bind(&mut self, bind: CoreHandle);
    fn relink_data_context(&mut self) {}
}

pub struct DataBind {
    pub base: DataBindBase,
    flags_byte: u8,
    dirt: u32,
    next_observer: Option<CoreHandle>,
    target: Option<CoreHandle>,
    source: Option<CoreHandle>,
    context_value: Option<RuntimeBindContextValue>,
    converter: Option<CoreHandle>,
    container: Option<DataBindContainerOwner>,
    file: RuntimeFileWeakHandle,
    changed_callback: Option<fn()>,
}

impl std::ops::Deref for DataBind {
    type Target = DataBindBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for DataBind {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Default for DataBind {
    fn default() -> Self {
        Self {
            base: DataBindBase::default(),
            flags_byte: 0,
            dirt: 0,
            next_observer: None,
            target: None,
            source: None,
            context_value: None,
            converter: None,
            container: None,
            file: RuntimeFileWeakHandle::default(),
            changed_callback: None,
        }
    }
}

impl DataBind {
    /// DataBind may itself be an observed target. Complete its generated
    /// no-op changed callback before releasing the owner for notification.
    pub(crate) fn set_uint_handle(owner: &CoreHandle, property_key: u16, value: u32) -> bool {
        let changed = owner.with_mut(|owner| {
            let Some(bind) = owner.as_data_bind_mut() else {
                return false;
            };
            match property_key {
                DataBindBase::PROPERTY_KEY_PROPERTY_KEY => bind.base.set_property_key_value(value),
                DataBindBase::FLAGS_PROPERTY_KEY => bind.base.set_flags_value(value),
                DataBindBase::CONVERTER_ID_PROPERTY_KEY => bind.base.set_converter_id_value(value),
                _ => false,
            }
        });
        if changed == Some(true) {
            if let Some(observers) = owner.property_observers() {
                observers.notify(property_key);
            }
        }
        changed.is_some()
    }

    pub fn import_handle(
        owner: &CoreHandle,
        stack: &mut crate::mechanical_port::source::importers::import_stack::ImportStack,
    ) -> StatusCode {
        use crate::mechanical_port::source::{
            generated::{
                animation::state_machine_base::StateMachineBase, artboard_base::ArtboardBase,
                backboard_base::BackboardBase,
            },
            importers::{
                artboard_importer::ArtboardImporter, backboard_importer::BackboardImporter,
                state_machine_importer::StateMachineImporter,
            },
            scripted::{
                scripted_data_converter::ScriptedDataConverter, scripted_object::with_script_input,
            },
        };
        let Some(backboard) = stack.latest::<BackboardImporter>(BackboardBase::TYPE_KEY) else {
            return StatusCode::MissingObject;
        };
        let file = backboard
            .file()
            .expect("BackboardImporter retains its File");
        owner.with_mut(|owner| owner.as_data_bind_mut().unwrap().set_file(file));
        backboard.add_data_converter_referencer(owner.clone());
        if let Some(target) = owner
            .with(|owner| owner.as_data_bind().unwrap().target())
            .flatten()
        {
            Self::initialize_handle(owner);
            if let Some(scripted_object) =
                with_script_input(&target, |input| input.script_input().scripted_object())
            {
                let mut owns_data_bind = true;
                if let Some(scripted_object) = scripted_object {
                    let component = scripted_object
                        .with(|object| object.as_component().is_some())
                        .unwrap_or(false);
                    if component {
                        if let Some(artboard) =
                            stack.latest::<ArtboardImporter>(ArtboardBase::TYPE_KEY)
                        {
                            owns_data_bind = false;
                            artboard.add_data_bind(owner.clone());
                        }
                    } else if scripted_object
                        .with_downcast::<ScriptedDataConverter, _>(|_| ())
                        .is_some()
                    {
                        DataBindContainerOwner::Authored(scripted_object)
                            .add_data_bind(owner.clone());
                        owns_data_bind = false;
                    }
                }
                with_script_input(&target, |input| {
                    input
                        .script_input_mut()
                        .set_data_bind(Some(owner.clone()), owns_data_bind)
                });
            } else if target
                .with(|target| target.as_data_converter().is_some())
                .unwrap_or(false)
            {
                DataBindContainerOwner::Authored(target).add_data_bind(owner.clone());
            } else if target
                .with(|target| target.as_formula_token().is_some())
                .unwrap_or(false)
            {
                target.with_mut(|target| {
                    target
                        .as_formula_token_mut()
                        .unwrap()
                        .add_data_bind(owner.clone())
                });
            } else if Self::state_machine_owned_type(target.core_type().unwrap_or_default()) {
                if let Some(machine) =
                    stack.latest::<StateMachineImporter>(StateMachineBase::TYPE_KEY)
                {
                    machine.add_data_bind(owner.clone());
                }
            } else {
                let artboard = target
                    .with(|target| {
                        target
                            .as_component()
                            .and_then(|component| component.artboard_handle())
                    })
                    .flatten();
                if let Some(artboard) = artboard {
                    DataBindContainerOwner::Authored(artboard).add_data_bind(owner.clone());
                } else if let Some(artboard) =
                    stack.latest::<ArtboardImporter>(ArtboardBase::TYPE_KEY)
                {
                    artboard.add_data_bind(owner.clone());
                } else if target
                    .with(|target| target.as_view_model_instance_value().is_some())
                    .unwrap_or(false)
                {
                    use crate::mechanical_port::source::{
                        generated::viewmodel::viewmodel_instance_base::ViewModelInstanceBase,
                        importers::viewmodel_instance_importer::ViewModelInstanceImporter,
                        viewmodel::viewmodel_instance::ViewModelInstance,
                    };
                    if let Some(importer) =
                        stack.latest::<ViewModelInstanceImporter>(ViewModelInstanceBase::TYPE_KEY)
                    {
                        importer
                            .view_model_instance()
                            .with_downcast_mut::<ViewModelInstance, _>(|instance| {
                                instance.add_value_data_bind(owner.clone())
                            });
                    }
                }
            }
        }
        owner
            .with_mut(|owner| owner.core_mut().import(stack))
            .unwrap_or(StatusCode::MissingObject)
    }

    pub fn relink_handle(owner: &CoreHandle) {
        let container = owner
            .with(|owner| owner.as_data_bind().and_then(|bind| bind.container.clone()))
            .flatten();
        if let Some(container) = container {
            container.rebuild_data_bind(owner.clone());
        }
    }

    pub fn bind_handle(owner: &CoreHandle) {
        owner.with_mut(|owner| owner.as_data_bind_mut().unwrap().context_value = None);
        let context = super::context::context_value::create_context_value(owner);
        owner.with_mut(|owner| {
            owner.as_data_bind_mut().unwrap().context_value =
                context.map(RuntimeBindContextValue::new)
        });
        if let Some(converter) = owner
            .with(|owner| owner.as_data_bind().unwrap().converter())
            .flatten()
        {
            converter.with_mut(|converter| {
                converter
                    .as_data_converter_capability_mut()
                    .unwrap()
                    .reset()
            });
        }
        let dirt = owner.with_mut(|owner| {
            let bind = owner.as_data_bind_mut().unwrap();
            bind.unsubscribe_target();
            bind.subscribe_target();
            bind.reconcile_dirt()
        });
        if let Some(dirt) = dirt {
            Self::add_dirt_handle(owner, dirt, true);
        }
    }

    pub fn update_data_bind_handle(owner: &CoreHandle, apply_target_to_source: bool) {
        #[cfg(any(test, feature = "testing"))]
        super::data_bind_container::SM_DATA_BIND_UPDATES
            .with(|count| count.set(count.get().wrapping_add(1)));
        let dirt = owner
            .with(|owner| owner.as_data_bind().unwrap().dirt())
            .expect("live DataBind");
        if dirt & DEPENDENTS == DEPENDENTS {
            let converter = owner
                .with(|owner| owner.as_data_bind().unwrap().converter())
                .flatten();
            if let Some(converter) = converter {
                super::converters::data_converter::DataConverter::update_handle(&converter);
            }
        }
        let wants = apply_target_to_source
            && owner
                .with(|owner| {
                    owner.as_data_bind().unwrap().in_persisting_list()
                        || dirt & BINDINGS_TARGET == BINDINGS_TARGET
                })
                .unwrap_or(false);
        if wants
            && !owner
                .with(|owner| owner.as_data_bind().unwrap().source_to_target_runs_first())
                .unwrap_or(false)
        {
            Self::update_source_binding_handle(owner, false);
        }
        if dirt != 0 {
            owner.with_mut(|owner| owner.as_data_bind_mut().unwrap().set_dirt(0));
            Self::update_handle(owner, dirt);
        }
        if wants
            && owner
                .with(|owner| owner.as_data_bind().unwrap().source_to_target_runs_first())
                .unwrap_or(false)
        {
            Self::update_source_binding_handle(owner, false);
        }
    }

    pub fn update_handle(owner: &CoreHandle, dirt: u32) {
        let state = owner
            .with_mut(|owner| {
                let bind = owner.as_data_bind_mut().unwrap();
                if bind.source.is_none()
                    || bind.context_value.is_none()
                    || dirt & BINDINGS != BINDINGS
                    || !bind.to_target()
                {
                    return None;
                }
                bind.set_flag(SUPPRESS_DIRT, true);
                Some((
                    bind.context_value.clone().unwrap(),
                    bind.target(),
                    bind.property_key(),
                    bind.base.flags() & DIRECTION == 0,
                ))
            })
            .flatten();
        let Some((context, target, key, is_main)) = state else {
            return;
        };
        context
            .borrow_mut()
            .apply(target, key, is_main, owner.clone());
        context.borrow_mut().refresh_target_value(owner.clone());
        owner.with_mut(|owner| {
            owner
                .as_data_bind_mut()
                .unwrap()
                .set_flag(SUPPRESS_DIRT, false)
        });
    }

    pub fn update_source_binding_handle(owner: &CoreHandle, invalidate: bool) {
        let state = owner
            .with(|owner| {
                let bind = owner.as_data_bind().unwrap();
                if !bind.to_source() {
                    return None;
                }
                Some((
                    bind.target()?,
                    bind.context_value.clone()?,
                    bind.property_key(),
                    bind.is_main_to_source(),
                ))
            })
            .flatten();
        let Some((target, context, key, is_main)) = state else {
            return;
        };
        if invalidate {
            context.invalidate();
        }
        context
            .borrow_mut()
            .apply_to_source(target, key, is_main, owner.clone());
    }

    pub fn advance_handle(owner: &CoreHandle, elapsed: f32) -> bool {
        let converter = owner
            .with(|owner| {
                let bind = owner.as_data_bind().unwrap();
                if bind.source.is_some() && !bind.has_flag(COLLAPSED) {
                    bind.converter()
                } else {
                    None
                }
            })
            .flatten();
        converter.is_some_and(|converter| {
            converter
                .with_mut(|converter| {
                    converter
                        .as_data_converter_capability_mut()
                        .unwrap()
                        .advance(elapsed)
                })
                .unwrap_or(false)
        })
    }

    pub fn unbind_handle(owner: &CoreHandle) {
        let source = owner
            .with(|owner| {
                let bind = owner.as_data_bind().unwrap();
                (!bind.binds_once()).then(|| bind.source()).flatten()
            })
            .flatten();
        if let Some(source) = source {
            source.with_mut(|source| source.as_view_model_instance_value_mut().unwrap().remove_dependent(&crate::mechanical_port::source::viewmodel::viewmodel_instance_value::ValueDependentHandle::core(owner.clone())));
        }
        owner.with_mut(|owner| {
            let bind = owner.as_data_bind_mut().unwrap();
            bind.source = None;
            bind.unsubscribe_target();
        });
        if let Some(converter) = owner
            .with(|owner| owner.as_data_bind().unwrap().converter())
            .flatten()
        {
            super::converters::data_converter::DataConverter::unbind_handle(&converter);
        }
        owner.with_mut(|owner| owner.as_data_bind_mut().unwrap().context_value = None);
    }

    fn handle(&self) -> Option<CoreHandle> {
        self.base.base.handle()
    }

    pub fn new(bind_flags: u32, property_key: u32, _display_property_key: u32) -> Self {
        let mut data_bind = Self::default();
        let mut callbacks = DataBindInitializationCallbacks;
        data_bind.base.set_flags(bind_flags, &mut callbacks);
        data_bind
            .base
            .set_property_key(property_key, &mut callbacks);
        data_bind
    }

    fn has_flag(&self, flag: u8) -> bool {
        self.flags_byte & flag != 0
    }

    fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.flags_byte |= flag;
        } else {
            self.flags_byte &= !flag;
        }
    }

    fn state_machine_owned_type(type_key: u16) -> bool {
        matches!(
            type_key,
            BindablePropertyNumberBase::TYPE_KEY
                | BindablePropertyStringBase::TYPE_KEY
                | BindablePropertyBooleanBase::TYPE_KEY
                | BindablePropertyEnumBase::TYPE_KEY
                | BindablePropertyArtboardBase::TYPE_KEY
                | BindablePropertyColorBase::TYPE_KEY
                | BindablePropertyTriggerBase::TYPE_KEY
                | BindablePropertyIntegerBase::TYPE_KEY
                | BindablePropertyAssetBase::TYPE_KEY
                | BindablePropertyViewModelBase::TYPE_KEY
                | BindablePropertyListBase::TYPE_KEY
                | TransitionPropertyViewModelComparatorBase::TYPE_KEY
                | StateTransitionBase::TYPE_KEY
        )
    }

    pub fn output_type(&self) -> DataType {
        if let Some(output) = self.converter.as_ref().and_then(|converter| {
            converter
                .with(|converter| {
                    converter
                        .as_data_converter_capability()
                        .map(|converter| converter.output_type())
                })
                .flatten()
        }) {
            if output != DataType::Input && output != DataType::None {
                return output;
            }
        }
        self.source_output_type()
    }

    pub fn source_output_type(&self) -> DataType {
        self.source
            .as_ref()
            .and_then(|source| {
                source
                    .with(|source| source.as_bind_source().map(BindSource::data_type))
                    .flatten()
            })
            .unwrap_or(DataType::None)
    }

    pub fn set_source(&mut self, value: CoreHandle) {
        let Some(bind) = self.handle() else {
            return;
        };
        if !self.binds_once() {
            value.with_mut(|source| {
                if let Some(source) = source.as_view_model_instance_value_mut() {
                    source.add_dependent(
                        crate::mechanical_port::source::viewmodel::viewmodel_instance_value::ValueDependentHandle::core(bind.clone()),
                    );
                }
            });
        }
        let is_number = value
            .with(|source| source.as_bind_source().map(BindSource::data_type))
            .flatten()
            == Some(DataType::Number);
        self.source = Some(value);
        if let Some(target) = self.target.as_ref() {
            target.with_mut(|target| {
                if let Some(target) = target.as_artboard_component_list_mut() {
                    target.should_reset_instances(is_number);
                }
            });
        }
    }

    pub fn clear_source(&mut self) {
        if let Some(source) = self.source.take()
            && !self.binds_once()
            && let Some(bind) = self.handle()
        {
            source.with_mut(|source| {
                if let Some(source) = source.as_view_model_instance_value_mut() {
                    source.remove_dependent(
                        &crate::mechanical_port::source::viewmodel::viewmodel_instance_value::ValueDependentHandle::core(bind),
                    );
                }
            });
        }
    }

    pub fn set_target(&mut self, value: Option<CoreHandle>) {
        if self.target == value {
            return;
        }
        self.unsubscribe_target();
        self.target = value;
        if self.property_key() == u32::from(crate::source::generated::layout_component_base::LayoutComponentBase::CLIP_PROPERTY_KEY) {
            if let Some(target) = &self.target {
                target.with_mut(|target| {
                    if let Some(layout) = target.as_layout_component_mut() {
                        layout.mark_clip_may_be_dynamic();
                    }
                });
            }
        }
        self.subscribe_target();
    }

    fn subscribe_target(&mut self) {
        if !self.to_source() || !self.target_supports_push() || self.handle().is_none() {
            return;
        }
        let Some(target) = self.target.clone() else {
            return;
        };
        let observers = if self.handle().as_ref() == Some(&target) {
            Some(self.base.base.ensure_property_observers())
        } else {
            target.ensure_property_observers()
        };
        if let Some(observers) = observers {
            observers.add(self);
            self.set_flag(OBSERVING, true);
        }
    }

    pub(crate) fn unsubscribe_target(&mut self) {
        if !self.has_flag(OBSERVING) {
            return;
        }
        if let Some(target) = &self.target {
            if let Some(observers) = target.property_observers() {
                // The active bind may already be retired or mutably borrowed.
                // Its own next link is read directly, never through its handle.
                observers.remove(self);
            }
            self.set_flag(OBSERVING, false);
        }
    }

    pub fn configure_target(&mut self, target: CoreHandle, property_key: u32) {
        self.set_target(Some(target));
        let mut callbacks = DataBindInitializationCallbacks;
        self.base.set_property_key(property_key, &mut callbacks);
    }

    pub fn on_target_destroyed(&mut self) {
        self.next_observer = None;
        self.target = None;
        self.set_flag(OBSERVING, false);
    }

    pub fn unbind(&mut self) {
        self.clear_source();
        self.unsubscribe_target();
        if let Some(converter) = self.converter.as_ref() {
            super::converters::data_converter::DataConverter::unbind_handle(converter);
        }
        self.context_value = None;
    }

    pub fn target_supports_push(&self) -> bool {
        let Some(target) = self.target.as_ref() else {
            return false;
        };
        let key = self.base.property_key() as u16;
        if matches!(
            key,
            SoloBase::ACTIVE_COMPONENT_ID_PROPERTY_KEY
                | NodeBase::COMPUTED_LOCAL_X_PROPERTY_KEY
                | NodeBase::COMPUTED_LOCAL_Y_PROPERTY_KEY
                | NodeBase::COMPUTED_WORLD_X_PROPERTY_KEY
                | NodeBase::COMPUTED_WORLD_Y_PROPERTY_KEY
                | NodeBase::COMPUTED_ROOT_X_PROPERTY_KEY
                | NodeBase::COMPUTED_ROOT_Y_PROPERTY_KEY
                | NodeBase::COMPUTED_WIDTH_PROPERTY_KEY
                | NodeBase::COMPUTED_HEIGHT_PROPERTY_KEY
                | ShapeBase::LENGTH_PROPERTY_KEY
                | ScrollConstraintBase::SCROLL_INDEX_PROPERTY_KEY
                | ScrollConstraintBase::SCROLL_PERCENT_X_PROPERTY_KEY
                | ScrollConstraintBase::SCROLL_PERCENT_Y_PROPERTY_KEY
                | ScrollConstraintBase::VELOCITY_X_PROPERTY_KEY
                | ScrollConstraintBase::VELOCITY_Y_PROPERTY_KEY
                | ScrollConstraintBase::SCROLL_ACTIVE_PROPERTY_KEY
                | ScrollConstraintBase::COMPUTED_CONTENT_WIDTH_PROPERTY_KEY
                | ScrollConstraintBase::COMPUTED_CONTENT_HEIGHT_PROPERTY_KEY
                | ColorChannelsBase::COLOR_ALPHA_PROPERTY_KEY
                | ColorChannelsBase::COLOR_RED_PROPERTY_KEY
                | ColorChannelsBase::COLOR_BLUE_PROPERTY_KEY
                | ColorChannelsBase::COLOR_GREEN_PROPERTY_KEY
        ) {
            return false;
        }
        !matches!(
            target.core_type().unwrap_or_default(),
            BindablePropertyAssetBase::TYPE_KEY
                | BindablePropertyViewModelBase::TYPE_KEY
                | ViewModelInstanceViewModelBase::TYPE_KEY
        )
    }

    pub fn can_skip(&self) -> bool {
        self.target
            .as_ref()
            .and_then(|target| {
                target.with(|target| target.as_component().map(|target| target.is_collapsed()))
            })
            .flatten()
            .unwrap_or(false)
            && self.base.property_key()
                != u32::from(LayoutSizingStyleBase::DISPLAY_VALUE_PROPERTY_KEY)
    }

    pub fn update(&mut self, value: u32) {
        if self.source.is_some()
            && self.context_value.is_some()
            && value & BINDINGS == BINDINGS
            && self.to_target()
        {
            self.set_flag(SUPPRESS_DIRT, true);
            let is_main = self.base.flags() & DIRECTION == 0;
            let target = self.target.clone();
            let Some(bind) = self.handle() else {
                self.set_flag(SUPPRESS_DIRT, false);
                return;
            };
            let mut context = self.context_value.as_ref().unwrap().borrow_mut();
            context.apply(target, self.base.property_key(), is_main, bind.clone());
            context.refresh_target_value(bind);
            drop(context);
            self.set_flag(SUPPRESS_DIRT, false);
        }
    }

    pub fn update_dependents(&mut self) {
        if let Some(converter) = self.converter.as_ref() {
            converter.with_mut(|converter| {
                if let Some(converter) = converter.as_data_converter_capability_mut() {
                    converter.update();
                }
            });
        }
    }

    pub fn update_source_binding(&mut self, invalidate: bool) {
        if self.to_source() {
            let Some(bind) = self.handle() else {
                return;
            };
            let is_main = self.is_main_to_source();
            if let (Some(target), Some(context)) =
                (self.target.clone(), self.context_value.as_mut())
            {
                let mut context = context.borrow_mut();
                if invalidate {
                    context.invalidate();
                }
                context.apply_to_source(target, self.base.property_key(), is_main, bind);
            }
        }
    }

    pub fn is_main_to_source(&self) -> bool {
        self.base.flags() & DIRECTION == TO_SOURCE
    }

    pub fn source_to_target_runs_first(&self) -> bool {
        self.base.flags() & SOURCE_TO_TARGET_FIRST == SOURCE_TO_TARGET_FIRST
    }

    pub fn reconcile_dirt(&self) -> u32 {
        (if self.to_target() { BINDINGS } else { 0 })
            | (if self.to_source() { BINDINGS_TARGET } else { 0 })
    }

    pub fn add_dirt(&mut self, value: u32, _recurse: bool) {
        let Some(callback) = self.begin_add_dirt(value) else {
            return;
        };
        if let Some(callback) = callback {
            callback();
        }
        if let Some(container) = self.finish_add_dirt() {
            container.add_dirty_data_bind_borrowed(self);
        }
    }

    pub(crate) fn add_dirt_handle(owner: &CoreHandle, value: u32, _recurse: bool) {
        let callback = owner
            .with_mut(|owner| owner.as_data_bind_mut()?.begin_add_dirt(value))
            .flatten();
        let Some(callback) = callback else {
            return;
        };
        if let Some(callback) = callback {
            callback();
        }
        // Read live state after the callback, matching addDirt's source order.
        // Generation checks reject a retired/replaced observer.
        let container = owner
            .with_mut(|owner| owner.as_data_bind_mut()?.finish_add_dirt())
            .flatten();
        if let Some(container) = container {
            // The converter's parent notification can inspect this child.
            // No child borrow spans that callback or the subsequent enqueue.
            container.add_dirty_data_bind(owner.clone());
        }
    }

    fn begin_add_dirt(&mut self, value: u32) -> Option<Option<fn()>> {
        if self.has_flag(SUPPRESS_DIRT) || self.dirt & value == value {
            return None;
        }
        let source = value & BINDINGS != 0;
        let target = value & BINDINGS_TARGET != 0;
        if source && target {
            self.set_flag(TARGET_ORIGIN, !self.source_to_target_runs_first());
        } else if target {
            self.set_flag(TARGET_ORIGIN, true);
        } else if source {
            self.set_flag(TARGET_ORIGIN, false);
        }
        self.dirt |= value;
        Some(self.changed_callback)
    }

    fn finish_add_dirt(&mut self) -> Option<DataBindContainerOwner> {
        if self.dirt & DEPENDENTS != 0
            && let Some(context) = self.context_value.as_mut()
        {
            context.invalidate();
        }
        (!self.has_flag(COLLAPSED))
            .then(|| self.container.clone())
            .flatten()
    }

    pub fn relink_data_bind(&mut self) {
        if let Some(container) = self.container.as_ref()
            && let Some(bind) = self.handle()
        {
            container.rebuild_data_bind(bind);
        }
    }

    pub fn binds_once(&self) -> bool {
        self.base.flags() & ONCE != 0
    }

    pub fn to_source(&self) -> bool {
        self.base.flags() & (TWO_WAY | TO_SOURCE) != 0
    }

    pub fn to_target(&self) -> bool {
        self.base.flags() & TWO_WAY != 0 || self.base.flags() & TO_SOURCE == 0
    }

    pub fn is_name_based(&self) -> bool {
        self.base.flags() & NAME_BASED != 0
    }

    pub fn advance(&mut self, elapsed: f32) -> bool {
        if self.source.is_some()
            && !self.has_flag(COLLAPSED)
            && let Some(converter) = self.converter.as_ref()
        {
            return converter
                .with_mut(|converter| {
                    converter
                        .as_data_converter_capability_mut()
                        .is_some_and(|converter| converter.advance(elapsed))
                })
                .unwrap_or(false);
        }
        false
    }

    pub fn collapse(&mut self, collapsed: bool) {
        if let Some(container) = self.collapse_state(collapsed) {
            container.add_dirty_data_bind_borrowed(self);
        }
    }

    pub(crate) fn collapse_handle(owner: &CoreHandle, collapsed: bool) {
        let container = owner
            .with_mut(|owner| owner.as_data_bind_mut()?.collapse_state(collapsed))
            .flatten();
        if let Some(container) = container {
            container.add_dirty_data_bind(owner.clone());
        }
    }

    fn collapse_state(&mut self, collapsed: bool) -> Option<DataBindContainerOwner> {
        if self.has_flag(COLLAPSED) == collapsed
            || self.base.property_key()
                == u32::from(LayoutSizingStyleBase::DISPLAY_VALUE_PROPERTY_KEY)
            || !self.target_supports_push()
        {
            return None;
        }
        self.set_flag(COLLAPSED, collapsed);
        (!collapsed && self.dirt != 0)
            .then(|| self.container.clone())
            .flatten()
    }

    pub fn clone_with_target_handle(
        source: &CoreHandle,
        target: Option<CoreHandle>,
    ) -> Option<CoreHandle> {
        Self::clone_with_target_into(source, target, &source.retain_arena()?)
    }

    pub(crate) fn clone_with_target_into(
        source: &CoreHandle,
        target: Option<CoreHandle>,
        arena: &CoreArena,
    ) -> Option<CoreHandle> {
        let cloned = source.clone_occurrence_into(arena)?;
        let (file, converter) = source
            .with(|source| {
                let bind = source.as_data_bind()?;
                Some((bind.file(), bind.converter()))
            })
            .flatten()?;
        cloned.with_mut(|cloned| {
            let bind = cloned.as_data_bind_mut().unwrap();
            bind.set_target(target);
            bind.set_file(file);
        });
        Self::initialize_handle(&cloned);
        if let Some(converter) = converter {
            let converter = converter.clone_occurrence_into(arena)?;
            cloned.with_mut(|cloned| {
                cloned
                    .as_data_bind_mut()
                    .unwrap()
                    .set_converter(Some(converter))
            });
        }
        Some(cloned)
    }

    pub fn is_instance_value_bind(&self) -> bool {
        self.has_flag(INSTANCE_VALUE_BIND)
    }
    pub fn mark_instance_value_bind(&mut self) {
        self.set_flag(INSTANCE_VALUE_BIND, true);
    }

    pub fn initialize(&mut self) {
        if let Some(target) = self.target.as_ref()
            && let Some(bind) = self.handle()
        {
            let collapsed = target
                .with_mut(|target| {
                    target
                        .as_component_mut()
                        .and_then(|target| target.register_collapsable(bind))
                })
                .flatten();
            if let Some(collapsed) = collapsed {
                self.collapse(collapsed);
            }
        }
    }

    pub(crate) fn initialize_handle(owner: &CoreHandle) {
        let target = owner
            .with(|owner| owner.as_data_bind().and_then(DataBind::target))
            .flatten();
        if let Some(target) = target {
            let collapsed = target
                .with_mut(|target| {
                    target
                        .as_component_mut()
                        .and_then(|target| target.register_collapsable(owner.clone()))
                })
                .flatten();
            if let Some(collapsed) = collapsed {
                Self::collapse_handle(owner, collapsed);
            }
        }
    }

    pub fn dirt(&self) -> u32 {
        self.dirt
    }

    pub fn set_dirt(&mut self, value: u32) {
        self.dirt = value;
    }

    pub fn target_origin(&self) -> bool {
        self.has_flag(TARGET_ORIGIN)
    }

    pub fn property_key(&self) -> u32 {
        self.base.property_key()
    }

    pub fn target(&self) -> Option<CoreHandle> {
        self.target.clone()
    }

    pub fn source(&self) -> Option<CoreHandle> {
        self.source.clone()
    }

    pub fn converter(&self) -> Option<CoreHandle> {
        self.converter.clone()
    }

    pub fn suppress_dirt(&mut self, value: bool) {
        self.set_flag(SUPPRESS_DIRT, value);
    }

    pub fn in_dirty_list(&self) -> bool {
        self.has_flag(IN_DIRTY)
    }

    pub fn set_in_dirty_list(&mut self, value: bool) {
        self.set_flag(IN_DIRTY, value);
    }

    pub fn in_persisting_list(&self) -> bool {
        self.has_flag(IN_PERSISTING)
    }

    pub fn set_in_persisting_list(&mut self, value: bool) {
        self.set_flag(IN_PERSISTING, value);
    }

    pub fn container(&self) -> Option<DataBindContainerOwner> {
        self.container.clone()
    }

    pub fn set_container(&mut self, value: Option<DataBindContainerOwner>) {
        self.container = value;
    }

    #[cfg(feature = "tools")]
    pub fn set_changed_callback(&mut self, callback: fn()) {
        self.changed_callback = Some(callback);
    }

    pub fn set_converter(&mut self, value: Option<CoreHandle>) {
        self.converter = value;
    }

    pub fn set_file(&mut self, value: RuntimeFileWeakHandle) {
        self.file = value;
    }

    pub fn file(&self) -> RuntimeFileWeakHandle {
        self.file.clone()
    }

    pub fn set_next_observer(&mut self, value: Option<CoreHandle>) {
        self.next_observer = value;
    }

    pub fn next_observer(&self) -> Option<CoreHandle> {
        self.next_observer.clone()
    }

    pub fn next_observer_ref(&mut self) -> &mut Option<CoreHandle> {
        &mut self.next_observer
    }
}

impl DataBindBaseCallbacks for DataBind {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }
}

struct DataBindInitializationCallbacks;

impl DataBindBaseCallbacks for DataBindInitializationCallbacks {
    fn notify_property_changed(&mut self, _property_key: u16) {}
}

impl Drop for DataBind {
    fn drop(&mut self) {
        self.unbind();
        // Each DataBind owns the converter cloned for it by the importer.
        // Retire that arena occurrence, never reconstruct a Box from a pointer.
        if let Some(converter) = self.converter.take() {
            converter.remove_occurrence();
        }
    }
}

#[cfg(test)]
mod observer_tests {
    use super::*;
    use crate::source::{generated::core_registry::CoreRegistry, node::Node};

    thread_local! {
        static EVENTS: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
        static ACTION: RefCell<Option<Box<dyn FnOnce()>>> = const { RefCell::new(None) };
    }
    fn first_changed() {
        EVENTS.with(|events| events.borrow_mut().push(1));
        let action = ACTION.with(|action| action.borrow_mut().take());
        if let Some(action) = action {
            action();
        }
    }
    fn second_changed() {
        EVENTS.with(|events| events.borrow_mut().push(2));
    }
    fn third_changed() {
        EVENTS.with(|events| events.borrow_mut().push(3));
    }
    fn fourth_changed() {
        EVENTS.with(|events| events.borrow_mut().push(4));
    }

    fn scene() -> (CoreArena, CoreHandle, [CoreHandle; 3]) {
        EVENTS.with(|events| events.borrow_mut().clear());
        ACTION.with(|action| *action.borrow_mut() = None);
        let arena = CoreArena::default();
        let target = arena.insert(Node::default());
        let binds = [first_changed as fn(), second_changed, third_changed].map(|callback| {
            let mut bind = DataBind::new(TO_SOURCE, NodeBase::X_PROPERTY_KEY.into(), 0);
            bind.changed_callback = Some(callback);
            arena.insert(bind)
        });
        // Registration prepends, so register 3,2,1 to produce 1 -> 2 -> 3.
        for bind in binds.iter().rev() {
            bind.with_mut(|owner| {
                owner
                    .as_data_bind_mut()
                    .unwrap()
                    .set_target(Some(target.clone()))
            });
        }
        (arena, target, binds)
    }
    fn notify(target: &CoreHandle, value: f32) {
        assert!(CoreRegistry::set_double_handle(
            target,
            NodeBase::X_PROPERTY_KEY.into(),
            value
        ));
    }
    fn events() -> Vec<u8> {
        EVENTS.with(|events| events.borrow().clone())
    }

    #[test]
    fn property_observers_preserve_prepend_order_and_skip_unchanged_values() {
        let (_arena, target, _binds) = scene();
        notify(&target, 1.0);
        assert_eq!(events(), [1, 2, 3]);
        notify(&target, 1.0);
        assert_eq!(events(), [1, 2, 3]);
    }

    #[test]
    fn duplicate_observer_registration_preserves_chain_in_both_build_modes() {
        let (_arena, target, binds) = scene();
        let observers = target.property_observers().unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            binds[1].with_mut(|owner| observers.add(owner.as_data_bind_mut().unwrap()));
        }));
        // Core::addPropertyObserver asserts in debug, but still returns without
        // inserting the duplicate when C++ is built with NDEBUG.
        assert_eq!(result.is_err(), cfg!(debug_assertions));
        // Check the links before notifying so a future cycle fails boundedly.
        for index in 0..binds.len() {
            assert_eq!(
                binds[index]
                    .with(|owner| owner.as_data_bind().unwrap().next_observer())
                    .flatten(),
                binds.get(index + 1).cloned()
            );
        }
        notify(&target, 1.0);
        assert_eq!(events(), [1, 2, 3]);
    }

    #[test]
    fn target_observer_callback_reads_published_node_property() {
        let (_arena, target, _binds) = scene();
        let callback_target = target.clone();
        ACTION.with(|action| {
            *action.borrow_mut() = Some(Box::new(move || {
                assert_eq!(
                    callback_target.with_downcast::<Node, _>(|node| node.base.x()),
                    Some(4.0)
                );
            }));
        });
        // A read of the published property mutates neither C++ observer links
        // nor the dependency vector traversed by xChanged().
        notify(&target, 4.0);
        assert_eq!(events(), [1, 2, 3]);
    }

    fn observe_read(
        arena: &CoreArena,
        target: &CoreHandle,
        key: u16,
        read: impl FnOnce() + 'static,
    ) -> CoreHandle {
        EVENTS.with(|events| events.borrow_mut().clear());
        ACTION.with(|action| *action.borrow_mut() = Some(Box::new(read)));
        let mut bind = DataBind::new(TO_SOURCE, u32::from(key), 0);
        bind.changed_callback = Some(first_changed);
        let bind = arena.insert(bind);
        bind.with_mut(|owner| {
            owner
                .as_data_bind_mut()
                .unwrap()
                .set_target(Some(target.clone()))
        });
        bind
    }

    #[test]
    fn every_scalar_registry_family_releases_target_before_final_notification() {
        use crate::source::animation::{
            keyed_object::KeyedObject, keyframe_bool::KeyFrameBool, keyframe_color::KeyFrameColor,
            keyframe_int::KeyFrameInt, keyframe_string::KeyFrameString, keyframe_uint::KeyFrameUint,
        };
        macro_rules! check {
            ($owner:ty, $key:expr, $set:ident, $get:ident, $value:expr) => {{
                let arena = CoreArena::default();
                let target = arena.insert(<$owner>::default());
                let read_target = target.clone();
                let value = $value;
                let expected = value.clone();
                let bind = observe_read(&arena, &target, $key, move || {
                    assert_eq!(
                        CoreRegistry::$get(&read_target, i32::from($key)),
                        Some(expected)
                    );
                });
                assert!(CoreRegistry::$set(&target, i32::from($key), value.clone()));
                assert_eq!(events(), [1]);
                // Remove the dirt latch so an erroneous unchanged notification
                // cannot hide behind DataBind's already-dirty short circuit.
                bind.with_mut(|owner| owner.as_data_bind_mut().unwrap().dirt = 0);
                assert!(CoreRegistry::$set(&target, i32::from($key), value));
                assert_eq!(events(), [1]);
            }};
        }
        check!(KeyFrameBool, 181u16, set_bool_handle, get_bool_handle, true);
        check!(
            KeyFrameString,
            280u16,
            set_string_handle,
            get_string_handle,
            String::from("published")
        );
        check!(
            KeyFrameColor,
            88u16,
            set_color_handle,
            get_color_handle,
            0x12345678i32
        );
        check!(
            KeyFrameUint,
            631u16,
            set_uint_handle,
            get_uint_handle,
            17u32
        );
        check!(KeyFrameInt, 1068u16, set_int_handle, get_int_handle, -17i32);
        check!(KeyedObject, 51u16, set_id_handle, get_id_handle, 17u32);
    }

    #[test]
    fn registry_aliases_notify_actual_underlying_property_after_release() {
        let (_arena, target, _binds) = scene();
        let read_target = target.clone();
        ACTION.with(|action| {
            *action.borrow_mut() = Some(Box::new(move || {
                assert_eq!(CoreRegistry::get_double_handle(&read_target, 13), Some(8.0));
            }))
        });
        assert!(CoreRegistry::set_double_handle(&target, 9, 8.0));
        assert_eq!(events(), [1, 2, 3]);

        let arena = CoreArena::default();
        let color = arena.insert(crate::source::shapes::paint::solid_color::SolidColor::default());
        CoreRegistry::set_color_handle(&color, 37, 0);
        let read_color = color.clone();
        let _bind = observe_read(&arena, &color, 37, move || {
            assert_eq!(
                CoreRegistry::get_color_handle(&read_color, 37),
                Some(0x00550000)
            );
        });
        assert!(CoreRegistry::set_uint_handle(&color, 118, 0x55));
        assert_eq!(events(), [1]);
    }

    #[test]
    fn animated_bool_and_string_notification_releases_target() {
        use crate::source::{
            animation::{keyframe_bool::KeyFrameBool, keyframe_string::KeyFrameString},
            generated::core_registry::CoreCapabilities,
        };
        let arena = CoreArena::default();
        let target = arena.insert(KeyFrameBool::default());
        let read_target = target.clone();
        let _bind = observe_read(&arena, &target, 181, move || {
            assert_eq!(CoreRegistry::get_bool_handle(&read_target, 181), Some(true));
        });
        let mut frame = KeyFrameBool::default();
        frame.base.set_value_value(true);
        assert!(frame.keyframe_apply(target, 181, 1.0, None));
        assert_eq!(events(), [1]);

        let target = arena.insert(KeyFrameString::default());
        let read_target = target.clone();
        let _bind = observe_read(&arena, &target, 280, move || {
            assert_eq!(
                CoreRegistry::get_string_handle(&read_target, 280),
                Some(String::from("animated"))
            );
        });
        let mut frame = KeyFrameString::default();
        frame.base.set_value_value(String::from("animated"));
        let next = arena.insert(KeyFrameString::default());
        assert!(frame.keyframe_interpolate(target, 280, 0.5, next, 1.0, None));
        assert_eq!(events(), [1]);
    }

    #[test]
    fn callback_registry_trigger_releases_target_before_property_notification() {
        use crate::source::{
            core::field_types::core_callback_type::CallbackData,
            generated::viewmodel::viewmodel_instance_trigger_base::ViewModelInstanceTriggerBase,
            viewmodel::viewmodel_instance_trigger::ViewModelInstanceTrigger,
        };
        let arena = CoreArena::default();
        let target = arena.insert(ViewModelInstanceTrigger::default());
        let read_target = target.clone();
        let _bind = observe_read(
            &arena,
            &target,
            ViewModelInstanceTriggerBase::PROPERTY_VALUE_PROPERTY_KEY,
            move || {
                assert_eq!(
                    CoreRegistry::get_uint_handle(
                        &read_target,
                        i32::from(ViewModelInstanceTriggerBase::PROPERTY_VALUE_PROPERTY_KEY)
                    ),
                    Some(1)
                );
            },
        );
        assert!(CoreRegistry::set_callback_handle(
            &target,
            1016,
            CallbackData::new(None, 0.0)
        ));
        assert_eq!(events(), [1]);
    }

    #[test]
    fn computed_scroll_index_notifies_x_before_publishing_y() {
        use crate::source::{
            constraints::scrolling::scroll_constraint::ScrollConstraint,
            generated::constraints::scrolling::scroll_constraint_base::ScrollConstraintBase,
        };
        let arena = CoreArena::default();
        // Imported authored offsets can precede initial derived-offset update.
        let mut scroll = ScrollConstraint::default();
        scroll.base.set_direction_value_value(2);
        scroll.base.set_scroll_offset_x_value(10.0);
        scroll.base.set_scroll_offset_y_value(20.0);
        let target = arena.insert(scroll);
        let _y = observe_read(
            &arena,
            &target,
            ScrollConstraintBase::SCROLL_OFFSET_Y_PROPERTY_KEY,
            || {},
        );
        let read_target = target.clone();
        let _x = observe_read(
            &arena,
            &target,
            ScrollConstraintBase::SCROLL_OFFSET_X_PROPERTY_KEY,
            move || {
                assert_eq!(
                    CoreRegistry::get_double_handle(
                        &read_target,
                        i32::from(ScrollConstraintBase::SCROLL_OFFSET_X_PROPERTY_KEY)
                    ),
                    Some(0.0)
                );
                assert_eq!(
                    CoreRegistry::get_double_handle(
                        &read_target,
                        i32::from(ScrollConstraintBase::SCROLL_OFFSET_Y_PROPERTY_KEY)
                    ),
                    Some(20.0)
                );
                ACTION.with(|action| {
                    *action.borrow_mut() = Some(Box::new(move || {
                        assert_eq!(
                            CoreRegistry::get_double_handle(
                                &read_target,
                                i32::from(ScrollConstraintBase::SCROLL_OFFSET_Y_PROPERTY_KEY)
                            ),
                            Some(0.0)
                        );
                    }))
                });
            },
        );
        // The upstream NaN index branch resolves to zero before consulting layout.
        assert!(CoreRegistry::set_double_handle(
            &target,
            i32::from(ScrollConstraintBase::SCROLL_INDEX_PROPERTY_KEY),
            f32::NAN
        ));
        assert_eq!(events(), [1, 1]);
    }

    #[test]
    fn removing_current_observer_in_callback_ends_live_traversal() {
        let (_arena, target, binds) = scene();
        let current = binds[0].clone();
        ACTION.with(|action| {
            *action.borrow_mut() = Some(Box::new(move || DataBind::unbind_handle(&current)))
        });
        notify(&target, 1.0);
        assert_eq!(events(), [1]);
        assert!(
            binds[0]
                .with(|owner| owner.as_data_bind().unwrap().next_observer())
                .flatten()
                .is_none()
        );
        assert_eq!(
            binds[1].with(|owner| owner.as_data_bind().unwrap().dirt()),
            Some(0)
        );
    }

    #[test]
    fn removing_successor_in_callback_splices_live_traversal() {
        let (_arena, target, binds) = scene();
        let successor = binds[1].clone();
        ACTION.with(|action| {
            *action.borrow_mut() = Some(Box::new(move || DataBind::unbind_handle(&successor)))
        });
        notify(&target, 1.0);
        assert_eq!(events(), [1, 3]);
        assert_eq!(
            binds[1].with(|owner| owner.as_data_bind().unwrap().dirt()),
            Some(0)
        );
    }

    #[test]
    fn prepending_in_callback_does_not_visit_new_head_until_next_notification() {
        let (arena, target, binds) = scene();
        let mut new_bind = DataBind::new(TO_SOURCE, NodeBase::X_PROPERTY_KEY.into(), 0);
        new_bind.changed_callback = Some(fourth_changed);
        let added = arena.insert(new_bind);
        let (added_action, target_action) = (added.clone(), target.clone());
        ACTION.with(|action| {
            *action.borrow_mut() = Some(Box::new(move || {
                added_action.with_mut(|owner| {
                    owner
                        .as_data_bind_mut()
                        .unwrap()
                        .set_target(Some(target_action))
                });
            }))
        });
        notify(&target, 1.0);
        assert_eq!(events(), [1, 2, 3]);
        for bind in binds.iter().chain(std::iter::once(&added)) {
            bind.with_mut(|owner| owner.as_data_bind_mut().unwrap().set_dirt(0));
        }
        EVENTS.with(|events| events.borrow_mut().clear());
        notify(&target, 2.0);
        assert_eq!(events(), [4, 1, 2, 3]);
    }

    #[test]
    fn destroying_target_detaches_every_observer_and_rebinding_remains_valid() {
        let (arena, target, binds) = scene();
        let removed = arena.remove(&target).unwrap();
        for bind in &binds {
            bind.with(|owner| {
                let bind = owner.as_data_bind().unwrap();
                assert!(bind.target().is_none());
                assert!(bind.next_observer().is_none());
                assert!(!bind.has_flag(OBSERVING));
            });
        }
        let replacement = arena.insert(Node::default());
        assert_eq!(replacement.identity_key().1, target.identity_key().1);
        assert!(target.property_observers().is_none());
        binds[0].with_mut(|owner| {
            owner
                .as_data_bind_mut()
                .unwrap()
                .set_target(Some(replacement.clone()))
        });
        drop(removed); // Old Core destruction must not detach the replacement.
        notify(&replacement, 1.0);
        assert_eq!(events(), [1]);
    }

    #[test]
    fn retiring_observer_splices_before_its_box_is_dropped_and_slot_is_reused() {
        let (arena, target, binds) = scene();
        let removed = arena.remove(&binds[1]).unwrap();
        let replacement = arena.insert(Node::default());
        assert_eq!(replacement.identity_key().1, binds[1].identity_key().1);
        notify(&target, 1.0);
        assert_eq!(events(), [1, 3]);
        drop(removed);
        assert!(replacement.is_alive());
    }

    #[test]
    fn core_clone_starts_empty_and_assignment_preserves_destination_observers() {
        let (arena, target, _binds) = scene();
        let cloned = target.clone_occurrence().unwrap();
        assert!(cloned.property_observers().is_none());
        let source = arena.insert(Node::default());
        target.with_mut(|destination| {
            source.with(|source| destination.core_mut().clone_from(source.core()))
        });
        notify(&target, 1.0);
        assert_eq!(events(), [1, 2, 3]);
        drop(arena.remove(&cloned).unwrap());
        assert!(target.property_observers().is_some());
    }

    #[test]
    fn self_target_registry_setter_releases_bind_before_changed_callback() {
        EVENTS.with(|events| events.borrow_mut().clear());
        let arena = CoreArena::default();
        let mut bind = DataBind::new(TO_SOURCE, DataBindBase::CONVERTER_ID_PROPERTY_KEY.into(), 0);
        bind.changed_callback = Some(first_changed);
        let bind = arena.insert(bind);
        bind.with_mut(|owner| {
            owner
                .as_data_bind_mut()
                .unwrap()
                .set_target(Some(bind.clone()))
        });
        let callback_bind = bind.clone();
        ACTION.with(|action| {
            *action.borrow_mut() = Some(Box::new(move || DataBind::unbind_handle(&callback_bind)))
        });
        assert!(CoreRegistry::set_id_handle(
            &bind,
            DataBindBase::CONVERTER_ID_PROPERTY_KEY.into(),
            3
        ));
        assert_eq!(events(), [1]);
        bind.with(|owner| {
            let bind = owner.as_data_bind().unwrap();
            assert!(!bind.has_flag(OBSERVING));
            assert!(bind.next_observer().is_none());
        });
    }

    #[test]
    fn value_bind_insertion_notifies_self_observed_flags_through_released_owner() {
        use crate::source::viewmodel::viewmodel_instance::ViewModelInstance;
        EVENTS.with(|events| events.borrow_mut().clear());
        let arena = CoreArena::default();
        let mut bind = DataBind::new(TWO_WAY, DataBindBase::FLAGS_PROPERTY_KEY.into(), 0);
        bind.changed_callback = Some(first_changed);
        let bind = arena.insert(bind);
        bind.with_mut(|owner| {
            owner
                .as_data_bind_mut()
                .unwrap()
                .set_target(Some(bind.clone()))
        });
        let callback_bind = bind.clone();
        ACTION.with(|action| {
            *action.borrow_mut() = Some(Box::new(move || {
                callback_bind.with(|owner| {
                    let bind = owner.as_data_bind().unwrap();
                    assert!(bind.source_to_target_runs_first());
                    assert!(bind.has_flag(OBSERVING));
                });
                DataBind::unbind_handle(&callback_bind);
            }))
        });
        let mut instance = ViewModelInstance::default();
        assert!(instance.value_data_binds().is_empty());
        instance.add_value_data_bind(bind.clone());
        assert_eq!(events(), [1]);
        assert_eq!(instance.value_data_binds(), &[bind.clone()]);
        bind.with(|owner| {
            let bind = owner.as_data_bind().unwrap();
            assert!(!bind.has_flag(OBSERVING));
            assert!(bind.next_observer().is_none());
        });
    }

    fn context_scene() -> (
        CoreArena,
        CoreHandle,
        CoreHandle,
        super::super::data_context::RuntimeDataContextHandle,
    ) {
        use crate::source::{
            data_bind::{
                data_bind_context::DataBindContext,
                data_context::{DataContext, RuntimeDataContextHandle},
            },
            viewmodel::{
                viewmodel_instance::ViewModelInstance,
                viewmodel_instance_number::ViewModelInstanceNumber,
            },
        };
        EVENTS.with(|events| events.borrow_mut().clear());
        ACTION.with(|action| *action.borrow_mut() = None);
        let arena = CoreArena::default();
        let mut number = ViewModelInstanceNumber::default();
        number.base.base.base.set_view_model_property_id_value(1);
        let source = arena.insert(number);
        let instance = arena.insert(ViewModelInstance::default());
        instance.with_mut(|owner| {
            owner
                .as_view_model_instance_mut()
                .unwrap()
                .add_value(source.clone())
        });
        let target = arena.insert(Node::default());
        let mut bind = DataBindContext::default();
        bind.base.base = DataBind::new(TWO_WAY, NodeBase::X_PROPERTY_KEY.into(), 0);
        bind.decode_source_path_ids(&[0, 1]);
        let bind = arena.insert(bind);
        bind.with_mut(|owner| owner.as_data_bind_mut().unwrap().set_target(Some(target)));
        let context = RuntimeDataContextHandle::new(DataContext::new(Some(instance)));
        DataBindContext::bind_from_context_handle(&bind, Some(context.clone()));
        bind.with_mut(|owner| {
            let bind = owner.as_data_bind_mut().unwrap();
            assert!(bind.source().as_ref() == Some(&source));
            bind.set_dirt(0);
            bind.changed_callback = Some(first_changed);
        });
        (arena, source, bind, context)
    }

    #[test]
    fn unchanged_source_reconcile_callback_can_unbind_and_retire_bind() {
        use crate::source::data_bind::data_bind_context::DataBindContext;
        let (arena, _source, bind, context) = context_scene();
        let callback_bind = bind.clone();
        ACTION.with(|action| {
            *action.borrow_mut() = Some(Box::new(move || {
                DataBind::unbind_handle(&callback_bind);
                assert!(callback_bind.remove_occurrence());
            }))
        });
        DataBindContext::bind_from_context_handle(&bind, Some(context));
        assert_eq!(events(), [1]);
        assert!(!arena.contains(&bind));
    }

    #[test]
    fn host_transaction_source_notification_releases_bind_for_unbind_and_retirement() {
        use crate::source::viewmodel::viewmodel_instance_number::ViewModelInstanceNumber;
        use crate::view_model_cell::{
            RuntimeHostMutationNotifications, RuntimeHostTransactionPublication,
            RuntimeTransactionKind,
        };
        let (arena, source, bind, _context) = context_scene();
        let callback_bind = bind.clone();
        ACTION.with(|action| {
            *action.borrow_mut() = Some(Box::new(move || {
                DataBind::unbind_handle(&callback_bind);
                assert!(callback_bind.remove_occurrence());
            }))
        });
        let publication =
            RuntimeHostTransactionPublication::begin(RuntimeTransactionKind::HostMutation).unwrap();
        let notifications = RuntimeHostMutationNotifications::begin().unwrap();
        source.with_downcast_mut::<ViewModelInstanceNumber, _>(|source| source.set_value(4.0));
        assert!(events().is_empty());
        notifications.commit();
        drop(publication);
        assert_eq!(events(), [1]);
        assert!(!arena.contains(&bind));
    }

    #[test]
    fn converter_parent_callback_can_read_and_unbind_live_child_before_enqueue() {
        use crate::source::data_bind::converters::data_converter_formula::DataConverterFormula;
        EVENTS.with(|events| events.borrow_mut().clear());
        let arena = CoreArena::default();
        let converter = arena.insert(DataConverterFormula::default());
        let parent = arena.insert(DataBind::default());
        let child = arena.insert(DataBind::default());
        let container = converter.data_bind_container().unwrap();
        container.set_parent_data_bind(Some(parent.clone()));
        container.add_data_bind(child.clone());
        parent.with_mut(|owner| {
            let bind = owner.as_data_bind_mut().unwrap();
            bind.set_dirt(0);
            bind.changed_callback = Some(first_changed);
        });
        child.with_mut(|owner| owner.as_data_bind_mut().unwrap().set_dirt(0));
        let callback_child = child.clone();
        ACTION.with(|action| {
            *action.borrow_mut() = Some(Box::new(move || {
                callback_child
                    .with(|owner| {
                        let child = owner.as_data_bind().unwrap();
                        assert_eq!(child.dirt() & BINDINGS, BINDINGS);
                        assert!(
                            !child.in_dirty_list(),
                            "upstream notifies parent before inserting child"
                        );
                    })
                    .expect("callback child remains live");
                DataBind::unbind_handle(&callback_child);
            }))
        });
        DataBind::add_dirt_handle(&child, BINDINGS, false);
        assert_eq!(events(), [1]);
        assert!(child.is_alive());
        assert_eq!(
            child.with(|owner| owner.as_data_bind().unwrap().in_dirty_list()),
            Some(true)
        );
    }

    #[test]
    fn component_uncollapse_releases_child_before_converter_parent_callback() {
        use crate::source::{
            component::{ComponentDirt, ComponentOccurrenceHandle},
            data_bind::converters::data_converter_formula::DataConverterFormula,
        };
        EVENTS.with(|events| events.borrow_mut().clear());
        let arena = CoreArena::default();
        let component = arena.insert(Node::default());
        component.with_mut(|owner| {
            owner
                .as_component_mut()
                .unwrap()
                .set_dirt(ComponentDirt::NONE)
        });
        let converter = arena.insert(DataConverterFormula::default());
        let parent = arena.insert(DataBind::default());
        let child = arena.insert(DataBind::new(0, NodeBase::X_PROPERTY_KEY.into(), 0));
        child.with_mut(|owner| {
            owner
                .as_data_bind_mut()
                .unwrap()
                .set_target(Some(component.clone()))
        });
        DataBind::initialize_handle(&child);
        let container = converter.data_bind_container().unwrap();
        container.set_parent_data_bind(Some(parent.clone()));
        container.add_data_bind(child.clone());
        assert!(ComponentOccurrenceHandle::Authored(component.clone()).collapse(true));
        DataBind::add_dirt_handle(&child, BINDINGS, false);
        assert_eq!(
            child.with(|owner| owner.as_data_bind().unwrap().in_dirty_list()),
            Some(false)
        );
        parent.with_mut(|owner| {
            let bind = owner.as_data_bind_mut().unwrap();
            bind.set_dirt(0);
            bind.changed_callback = Some(first_changed);
        });
        let callback_child = child.clone();
        ACTION.with(|action| {
            *action.borrow_mut() = Some(Box::new(move || {
                callback_child
                    .with(|owner| {
                        let child = owner.as_data_bind().unwrap();
                        assert!(!child.has_flag(COLLAPSED));
                        assert!(!child.in_dirty_list());
                    })
                    .expect("uncollapsed child remains readable");
                DataBind::unbind_handle(&callback_child);
            }))
        });
        assert!(ComponentOccurrenceHandle::Authored(component).collapse(false));
        assert_eq!(events(), [1]);
        assert_eq!(
            child.with(|owner| owner.as_data_bind().unwrap().in_dirty_list()),
            Some(true)
        );
    }

    #[test]
    fn number_occurrence_notification_callback_reads_published_source() {
        use crate::source::generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase;
        use crate::source::viewmodel::viewmodel_instance_number::ViewModelInstanceNumber;
        let (_arena, source, _bind, _context) = context_scene();
        let callback_source = source.clone();
        ACTION.with(|action| {
            *action.borrow_mut() = Some(Box::new(move || {
                assert_eq!(
                    callback_source
                        .with_downcast::<ViewModelInstanceNumber, _>(|source| source.value()),
                    Some(4.0)
                );
            }))
        });
        // Defined upstream: merely read the already-published value. Never
        // mutate the live dependency vector being traversed by the source.
        assert!(CoreRegistry::set_double_handle(
            &source,
            ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
            4.0
        ));
        assert_eq!(events(), [1]);
    }

    #[cfg(feature = "tools")]
    #[test]
    fn number_tools_callback_and_value_are_read_after_dependencies() {
        use crate::source::viewmodel::viewmodel_instance_number::ViewModelInstanceNumber;
        fn changed(number: &mut ViewModelInstanceNumber, value: f32) {
            assert_eq!(number.value(), 7.0);
            assert_eq!(value, 7.0);
            EVENTS.with(|events| events.borrow_mut().push(7));
        }
        let (_arena, source, _bind, _context) = context_scene();
        let callback_source = source.clone();
        ACTION.with(|action| {
            *action.borrow_mut() = Some(Box::new(move || {
                callback_source.with_downcast_mut::<ViewModelInstanceNumber, _>(|number| {
                    number.on_changed(Some(changed));
                });
                // Re-enter the value setter without changing the dependency list.
                assert!(ViewModelInstanceNumber::set_value_handle(&callback_source, 7.0));
            }));
        });
        assert!(ViewModelInstanceNumber::set_value_handle(&source, 4.0));
        // Inner write, then the outer write's live post-dependency callback.
        assert_eq!(events(), [1, 7, 7]);
    }
}
