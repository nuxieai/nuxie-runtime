use crate::mechanical_port::source::{
    core::CoreHandle,
    data_bind::data_bind_container::DataBindContainerOwner,
    data_bind::data_values::data_type::DataType,
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
        shapes::shape_base::ShapeBase,
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
pub const TO_SOURCE: u32 = 1;
pub const TWO_WAY: u32 = 2;
pub const DIRECTION: u32 = 3;
pub const SOURCE_TO_TARGET_FIRST: u32 = 4;
pub const ONCE: u32 = 8;
pub const NAME_BASED: u32 = 16;

const COLLAPSED: u8 = 1;
const IN_DIRTY: u8 = 2;
const IN_PERSISTING: u8 = 4;
const SUPPRESS_DIRT: u8 = 8;
const OBSERVING: u8 = 16;
const TARGET_ORIGIN: u8 = 32;

pub trait BindTarget {
    fn add_property_observer(&mut self, bind: CoreHandle);
    fn remove_property_observer(&mut self, bind: &CoreHandle);
    fn core_type(&self) -> u16 {
        0
    }
    fn is_component(&self) -> bool;
    fn is_collapsed(&self) -> bool;
    fn add_collapsable(&mut self, bind: CoreHandle);
    fn should_reset_instances(&mut self, value: bool);
    fn script_input(&mut self) -> Option<&mut dyn BindScriptInput> {
        None
    }
    fn add_data_bind_to_converter(&mut self, _bind: CoreHandle) -> bool {
        false
    }
    fn add_data_bind_to_formula_token(&mut self, _bind: CoreHandle) -> bool {
        false
    }
    fn add_data_bind_to_parent_artboard(&mut self, _bind: CoreHandle) -> bool {
        false
    }
}

pub trait BindScriptInput {
    fn scripted_object(&self) -> Option<CoreHandle>;
    fn set_scripted_object(&mut self, object: Option<CoreHandle>);
    fn data_bind(&self) -> Option<CoreHandle>;
    fn set_data_bind(&mut self, bind: Option<CoreHandle>, owns_data_bind: bool);
}

pub trait BindScriptedObject {
    fn has_component(&self) -> bool;
    fn add_data_bind_from_scripted_object(&mut self, bind: CoreHandle) -> bool;
}

pub trait DataBindAddedContext {
    fn on_added_dirty_super(&mut self, bind: &mut DataBind) -> StatusCode;
}

pub trait DataBindImportStack {
    fn backboard_file(&mut self) -> Option<RuntimeFileWeakHandle>;
    fn add_data_converter_referencer(&mut self, bind: CoreHandle);
    fn has_artboard_importer(&self) -> bool;
    fn add_artboard_data_bind(&mut self, bind: CoreHandle);
    fn add_state_machine_data_bind(&mut self, bind: CoreHandle) -> bool;
    fn import_super(&mut self, bind: &mut DataBind) -> StatusCode;
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

pub trait ContextFactory {
    fn create(
        &mut self,
        data_type: DataType,
        bind: CoreHandle,
    ) -> Option<Box<dyn BindContextValue>>;
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
        let observing = owner
            .with(|owner| {
                let bind = owner.as_data_bind().unwrap();
                bind.has_flag(OBSERVING).then(|| bind.target()).flatten()
            })
            .flatten();
        if let Some(target) = observing {
            target.with_mut(|target| target.core_mut().remove_property_observer(owner));
            owner.with_mut(|owner| owner.as_data_bind_mut().unwrap().set_flag(OBSERVING, false));
        }
        let subscribe = owner
            .with(|owner| {
                let bind = owner.as_data_bind().unwrap();
                (bind.to_source() && bind.target_supports_push())
                    .then(|| bind.target())
                    .flatten()
            })
            .flatten();
        if let Some(target) = subscribe {
            target.with_mut(|target| target.core_mut().add_property_observer(owner.clone()));
            owner.with_mut(|owner| owner.as_data_bind_mut().unwrap().set_flag(OBSERVING, true));
        }
        owner.with_mut(|owner| {
            let bind = owner.as_data_bind_mut().unwrap();
            bind.add_dirt(bind.reconcile_dirt(), true);
        });
    }

    pub fn update_data_bind_handle(owner: &CoreHandle, apply_target_to_source: bool) {
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
        let observing = owner
            .with_mut(|owner| {
                let bind = owner.as_data_bind_mut().unwrap();
                bind.source = None;
                bind.has_flag(OBSERVING).then(|| bind.target()).flatten()
            })
            .flatten();
        if let Some(target) = observing {
            target.with_mut(|target| target.core_mut().remove_property_observer(owner));
            owner.with_mut(|owner| owner.as_data_bind_mut().unwrap().set_flag(OBSERVING, false));
        }
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

    pub fn on_added_dirty(&mut self, context: &mut dyn DataBindAddedContext) -> StatusCode {
        let code = context.on_added_dirty_super(self);
        if code != StatusCode::Ok {
            return code;
        }
        StatusCode::Ok
    }

    pub fn import(&mut self, import_stack: &mut dyn DataBindImportStack) -> StatusCode {
        let Some(bind) = self.handle() else {
            return StatusCode::MissingObject;
        };
        let Some(file) = import_stack.backboard_file() else {
            return StatusCode::MissingObject;
        };
        self.set_file(file);
        import_stack.add_data_converter_referencer(bind.clone());

        let Some(target_handle) = self.target.clone() else {
            return import_stack.import_super(self);
        };
        self.initialize();
        let imported = target_handle.with_mut(|target| {
            let Some(target) = target.as_bind_target_mut() else {
                return false;
            };
            if let Some(input) = target.script_input() {
                let mut owns_data_bind = true;
                if let Some(scripted_object) = input.scripted_object() {
                    let has_component = scripted_object
                        .with_mut(|scripted_object| {
                            scripted_object
                                .as_bind_scripted_object_mut()
                                .is_some_and(|scripted_object| scripted_object.has_component())
                        })
                        .unwrap_or(false);
                    if has_component {
                        if import_stack.has_artboard_importer() {
                            owns_data_bind = false;
                            import_stack.add_artboard_data_bind(bind.clone());
                        }
                    } else if scripted_object
                        .with_mut(|scripted_object| {
                            scripted_object.as_bind_scripted_object_mut().is_some_and(
                                |scripted_object| {
                                    scripted_object.add_data_bind_from_scripted_object(bind.clone())
                                },
                            )
                        })
                        .unwrap_or(false)
                    {
                        owns_data_bind = false;
                    }
                }
                input.set_data_bind(Some(bind.clone()), owns_data_bind);
            } else if target.add_data_bind_to_converter(bind.clone()) {
            } else if target.add_data_bind_to_formula_token(bind.clone()) {
            } else if Self::state_machine_owned_type(target.core_type()) {
                return import_stack.add_state_machine_data_bind(bind.clone());
            } else {
                if target.is_component() && target.add_data_bind_to_parent_artboard(bind.clone()) {
                    return true;
                }
                if import_stack.has_artboard_importer() {
                    import_stack.add_artboard_data_bind(bind.clone());
                    return true;
                }
            }
            false
        });
        if imported == Some(true) {
            return import_stack.import_super(self);
        }
        import_stack.import_super(self)
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
                if let Some(target) = target.as_bind_target_mut() {
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

    pub fn bind(&mut self, factory: &mut dyn ContextFactory) {
        let Some(bind) = self.handle() else {
            return;
        };
        self.context_value = None;
        self.context_value = factory
            .create(self.output_type(), bind.clone())
            .map(RuntimeBindContextValue::new);
        if let Some(converter) = self.converter.as_ref() {
            converter.with_mut(|converter| {
                if let Some(converter) = converter.as_data_converter_capability_mut() {
                    converter.reset();
                }
            });
        }
        if self.has_flag(OBSERVING) {
            if let Some(target) = self.target.as_ref() {
                target.with_mut(|target| {
                    if let Some(target) = target.as_bind_target_mut() {
                        target.remove_property_observer(&bind);
                    }
                });
                self.set_flag(OBSERVING, false);
            }
        }
        if self.to_source() && self.target_supports_push() {
            if let Some(target) = self.target.as_ref() {
                target.with_mut(|target| {
                    if let Some(target) = target.as_bind_target_mut() {
                        target.add_property_observer(bind.clone());
                    }
                });
                self.set_flag(OBSERVING, true);
            }
        }
        self.add_dirt(self.reconcile_dirt(), true);
    }

    pub fn set_target(&mut self, value: Option<CoreHandle>) {
        if self.target == value {
            return;
        }
        let bind = self.handle();
        if self.has_flag(OBSERVING) {
            if let (Some(target), Some(bind)) = (self.target.as_ref(), bind.as_ref()) {
                target.with_mut(|target| {
                    if let Some(target) = target.as_bind_target_mut() {
                        target.remove_property_observer(bind);
                    }
                });
                self.set_flag(OBSERVING, false);
            }
        }
        self.target = value;
        if self.to_source() && self.target_supports_push() {
            if let (Some(target), Some(bind)) = (self.target.as_ref(), bind) {
                target.with_mut(|target| {
                    if let Some(target) = target.as_bind_target_mut() {
                        target.add_property_observer(bind.clone());
                    }
                });
                self.set_flag(OBSERVING, true);
            }
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
        let bind = self.handle();
        self.clear_source();
        if self.has_flag(OBSERVING) {
            if let (Some(target), Some(bind)) = (self.target.as_ref(), bind.as_ref()) {
                target.with_mut(|target| {
                    if let Some(target) = target.as_bind_target_mut() {
                        target.remove_property_observer(bind);
                    }
                });
                self.set_flag(OBSERVING, false);
            }
        }
        if let Some(converter) = self.converter.as_ref() {
            converter.with_mut(|converter| {
                if let Some(converter) = converter.as_data_converter_capability_mut() {
                    converter.unbind();
                }
            });
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
                target.with(|target| {
                    target
                        .as_bind_target()
                        .map(|target| target.is_component() && target.is_collapsed())
                })
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
        if self.has_flag(SUPPRESS_DIRT) || self.dirt & value == value {
            return;
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
        if let Some(callback) = self.changed_callback {
            callback();
        }
        if self.dirt & DEPENDENTS != 0
            && let Some(context) = self.context_value.as_mut()
        {
            context.invalidate();
        }
        if !self.has_flag(COLLAPSED)
            && let Some(container) = self.container.clone()
        {
            container.add_dirty_data_bind_borrowed(self);
        }
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
        if self.has_flag(COLLAPSED) == collapsed
            || self.base.property_key()
                == u32::from(LayoutSizingStyleBase::DISPLAY_VALUE_PROPERTY_KEY)
            || !self.target_supports_push()
        {
            return;
        }
        self.set_flag(COLLAPSED, collapsed);
        if !collapsed
            && self.dirt != 0
            && let Some(container) = self.container.clone()
        {
            container.add_dirty_data_bind_borrowed(self);
        }
    }

    pub fn initialize(&mut self) {
        if let Some(target) = self.target.as_ref()
            && let Some(bind) = self.handle()
        {
            target.with_mut(|target| {
                if let Some(target) = target.as_bind_target_mut()
                    && target.is_component()
                {
                    target.add_collapsable(bind);
                }
            });
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
