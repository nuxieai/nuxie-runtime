use crate::mechanical_port::source::{
    data_bind::data_values::data_type::DataType,
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
};

pub const DEPENDENTS: u32 = 1;
pub const BINDINGS: u32 = 2;
pub const BINDINGS_TARGET: u32 = 4;
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
    fn add_property_observer(&mut self, bind: *mut DataBind);
    fn remove_property_observer(&mut self, bind: *mut DataBind);
    fn core_type(&self) -> u16 {
        0
    }
    fn is_component(&self) -> bool;
    fn is_collapsed(&self) -> bool;
    fn add_collapsable(&mut self, bind: *mut DataBind);
    fn should_reset_instances(&mut self, value: bool);
    fn script_input(&mut self) -> Option<&mut dyn BindScriptInput> {
        None
    }
    fn add_data_bind_to_converter(&mut self, _bind: *mut DataBind) -> bool {
        false
    }
    fn add_data_bind_to_formula_token(&mut self, _bind: *mut DataBind) -> bool {
        false
    }
    fn add_data_bind_to_parent_artboard(&mut self, _bind: *mut DataBind) -> bool {
        false
    }
}

pub trait BindScriptInput {
    fn scripted_object(&mut self) -> Option<&mut dyn BindScriptedObject>;
    fn set_data_bind(&mut self, bind: *mut DataBind, owns_data_bind: bool);
}

pub trait BindScriptedObject {
    fn has_component(&self) -> bool;
    fn add_data_bind_from_scripted_object(&mut self, bind: *mut DataBind) -> bool;
}

pub trait DataBindAddedContext {
    fn on_added_dirty_super(&mut self, bind: &mut DataBind) -> StatusCode;
}

pub trait DataBindImportStack {
    fn backboard_file(&mut self) -> Option<*mut ()>;
    fn add_data_converter_referencer(&mut self, bind: *mut DataBind);
    fn has_artboard_importer(&self) -> bool;
    fn add_artboard_data_bind(&mut self, bind: *mut DataBind);
    fn add_state_machine_data_bind(&mut self, bind: *mut DataBind) -> bool;
    fn import_super(&mut self, bind: &mut DataBind) -> StatusCode;
}

pub trait BindSource {
    fn add_dependent(&self, bind: *mut DataBind);
    fn remove_dependent(&self, bind: *mut DataBind);
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
    fn apply(
        &mut self,
        target: Option<*mut dyn BindTarget>,
        property_key: u32,
        is_main: bool,
        bind: *mut DataBind,
    );
    fn refresh_target_value(&mut self, bind: *mut DataBind);
    fn invalidate(&mut self);
    fn apply_to_source(
        &mut self,
        target: *mut dyn BindTarget,
        property_key: u32,
        is_main: bool,
        bind: *mut DataBind,
    );
}

pub trait ContextFactory {
    fn create(
        &mut self,
        data_type: DataType,
        bind: *mut DataBind,
    ) -> Option<Box<dyn BindContextValue>>;
}

pub trait BindContainer {
    fn add_dirty_data_bind(&mut self, bind: *mut DataBind);
    fn rebuild_data_bind(&mut self, bind: *mut DataBind);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatusCode {
    Ok,
    MissingObject,
}

pub struct DataBind {
    pub base: DataBindBase,
    flags_byte: u8,
    dirt: u32,
    next_observer: Option<*mut DataBind>,
    target: Option<*mut dyn BindTarget>,
    source: Option<*mut dyn BindSource>,
    context_value: Option<Box<dyn BindContextValue>>,
    converter: Option<*mut dyn BindConverter>,
    container: Option<*mut dyn BindContainer>,
    file: Option<*mut ()>,
    changed_callback: Option<fn()>,
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
            file: None,
            changed_callback: None,
        }
    }
}

impl DataBind {
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
        let Some(file) = import_stack.backboard_file() else {
            return StatusCode::MissingObject;
        };
        self.set_file(Some(file));
        let bind = self as *mut Self;
        import_stack.add_data_converter_referencer(bind);

        let Some(target_ptr) = self.target else {
            return import_stack.import_super(self);
        };
        let target = unsafe { &mut *target_ptr };
        self.initialize();
        if let Some(input) = target.script_input() {
            let mut owns_data_bind = true;
            if let Some(scripted_object) = input.scripted_object() {
                if scripted_object.has_component() {
                    if import_stack.has_artboard_importer() {
                        owns_data_bind = false;
                        import_stack.add_artboard_data_bind(bind);
                    }
                } else if scripted_object.add_data_bind_from_scripted_object(bind) {
                    owns_data_bind = false;
                }
            }
            input.set_data_bind(bind, owns_data_bind);
        } else if target.add_data_bind_to_converter(bind) {
        } else if target.add_data_bind_to_formula_token(bind) {
        } else if Self::state_machine_owned_type(target.core_type()) {
            if import_stack.add_state_machine_data_bind(bind) {
                return import_stack.import_super(self);
            }
        } else {
            if target.is_component() && target.add_data_bind_to_parent_artboard(bind) {
                return import_stack.import_super(self);
            }
            if import_stack.has_artboard_importer() {
                import_stack.add_artboard_data_bind(bind);
                return import_stack.import_super(self);
            }
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
        if let Some(converter) = self.converter {
            let output = unsafe { (&*converter).output_type() };
            if output != DataType::Input && output != DataType::None {
                return output;
            }
        }
        self.source_output_type()
    }

    pub fn source_output_type(&self) -> DataType {
        self.source
            .map_or(DataType::None, |source| unsafe { (&*source).data_type() })
    }

    pub fn set_source(&mut self, value: *mut dyn BindSource) {
        if !self.binds_once() {
            unsafe {
                (&*value).add_dependent(self as *mut Self);
            }
        }
        self.source = Some(value);
        if let Some(target) = self.target {
            unsafe {
                (&mut *target).should_reset_instances((&*value).data_type() == DataType::Number);
            }
        }
    }

    pub fn clear_source(&mut self) {
        if let Some(source) = self.source.take()
            && !self.binds_once()
        {
            unsafe {
                (&*source).remove_dependent(self as *mut Self);
            }
        }
    }

    pub fn bind(&mut self, factory: &mut dyn ContextFactory) {
        self.context_value = None;
        self.context_value = factory.create(self.output_type(), self as *mut Self);
        if let Some(converter) = self.converter {
            unsafe {
                (&mut *converter).reset();
            }
        }
        if self.has_flag(OBSERVING) {
            if let Some(target) = self.target {
                unsafe {
                    (&mut *target).remove_property_observer(self as *mut Self);
                }
                self.set_flag(OBSERVING, false);
            }
        }
        if self.to_source() && self.target_supports_push() {
            if let Some(target) = self.target {
                unsafe {
                    (&mut *target).add_property_observer(self as *mut Self);
                }
                self.set_flag(OBSERVING, true);
            }
        }
        self.add_dirt(self.reconcile_dirt(), true);
    }

    pub fn set_target(&mut self, value: Option<*mut dyn BindTarget>) {
        if same_ptr(self.target, value) {
            return;
        }
        if self.has_flag(OBSERVING) {
            if let Some(target) = self.target {
                unsafe {
                    (&mut *target).remove_property_observer(self as *mut Self);
                }
                self.set_flag(OBSERVING, false);
            }
        }
        self.target = value;
        if self.to_source() && self.target_supports_push() {
            if let Some(target) = self.target {
                unsafe {
                    (&mut *target).add_property_observer(self as *mut Self);
                }
                self.set_flag(OBSERVING, true);
            }
        }
    }

    pub fn on_target_destroyed(&mut self) {
        self.next_observer = None;
        self.target = None;
        self.set_flag(OBSERVING, false);
    }

    pub fn unbind(&mut self) {
        self.clear_source();
        if self.has_flag(OBSERVING) {
            if let Some(target) = self.target {
                unsafe {
                    (&mut *target).remove_property_observer(self as *mut Self);
                }
                self.set_flag(OBSERVING, false);
            }
        }
        if let Some(converter) = self.converter {
            unsafe {
                (&mut *converter).unbind();
            }
        }
        self.context_value = None;
    }

    pub fn target_supports_push(&self) -> bool {
        let Some(target) = self.target else {
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
            unsafe { (&*target).core_type() },
            BindablePropertyAssetBase::TYPE_KEY
                | BindablePropertyViewModelBase::TYPE_KEY
                | ViewModelInstanceViewModelBase::TYPE_KEY
        )
    }

    pub fn can_skip(&self) -> bool {
        self.target
            .is_some_and(|target| unsafe { (&*target).is_component() && (&*target).is_collapsed() })
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
            let is_main = self.bind_flags & DIRECTION == 0;
            let target = self.target;
            let bind = self as *mut Self;
            let context = self.context_value.as_mut().unwrap();
            context.apply(target, self.base.property_key(), is_main, bind);
            context.refresh_target_value(bind);
            self.set_flag(SUPPRESS_DIRT, false);
        }
    }

    pub fn update_dependents(&mut self) {
        if let Some(converter) = self.converter {
            unsafe {
                (&mut *converter).update();
            }
        }
    }

    pub fn update_source_binding(&mut self, invalidate: bool) {
        if self.to_source() {
            let bind = self as *mut Self;
            let is_main = self.is_main_to_source();
            if let (Some(target), Some(context)) = (self.target, self.context_value.as_mut()) {
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
            && let Some(container) = self.container
        {
            unsafe {
                (&mut *container).add_dirty_data_bind(self as *mut Self);
            }
        }
    }

    pub fn relink_data_bind(&mut self) {
        if let Some(container) = self.container {
            unsafe {
                (&mut *container).rebuild_data_bind(self as *mut Self);
            }
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
            && let Some(converter) = self.converter
        {
            return unsafe { (&mut *converter).advance(elapsed) };
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
            && let Some(container) = self.container
        {
            unsafe {
                (&mut *container).add_dirty_data_bind(self as *mut Self);
            }
        }
    }

    pub fn initialize(&mut self) {
        if let Some(target) = self.target
            && unsafe { (&*target).is_component() }
        {
            unsafe {
                (&mut *target).add_collapsable(self as *mut Self);
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

    pub fn target(&self) -> Option<*mut dyn BindTarget> {
        self.target
    }

    pub fn source(&self) -> Option<*mut dyn BindSource> {
        self.source
    }

    pub fn converter(&self) -> Option<*mut dyn BindConverter> {
        self.converter
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

    pub fn set_container(&mut self, value: Option<*mut dyn BindContainer>) {
        self.container = value;
    }

    pub fn set_converter(&mut self, value: Option<*mut dyn BindConverter>) {
        self.converter = value;
    }

    pub fn set_file(&mut self, value: Option<*mut ()>) {
        self.file = value;
    }

    pub fn set_next_observer(&mut self, value: Option<*mut DataBind>) {
        self.next_observer = value;
    }

    pub fn next_observer(&self) -> Option<*mut DataBind> {
        self.next_observer
    }

    pub fn next_observer_ref(&mut self) -> &mut Option<*mut DataBind> {
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
        if let Some(converter) = self.converter.take() {
            unsafe {
                drop(Box::from_raw(converter));
            }
        }
    }
}

fn same_ptr<T: ?Sized>(a: Option<*mut T>, b: Option<*mut T>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => core::ptr::addr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}
