use super::{
    text_style::TextStyleOwner, text_style_global::TextStyleGlobal,
    text_style_reference::TextStyleReferenceWeak,
};
use crate::source::{
    advance_flags::AdvanceFlags,
    advancing_component::QuietState,
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    core::{CoreArena, CoreHandle},
    data_bind::data_context::{DataContext, RuntimeDataContextHandle},
    factory::RuntimeFactoryHandle,
    generated::{core_registry::CoreRegistry, viewmodel::*},
    viewmodel::viewmodel_instance::ViewModelInstance,
};
use std::rc::Rc;

#[derive(Clone, PartialEq)]
enum InputValue {
    Number(f32),
    Boolean(bool),
    String(String),
    Color(i32),
    Index(u32),
}
struct InputLink {
    from: CoreHandle,
    to: CoreHandle,
    last: InputValue,
}
fn read_input(h: &CoreHandle) -> Option<InputValue> {
    let key = h.with(|v| v.core_type())?;
    Some(match key {
        viewmodel_instance_number_base::ViewModelInstanceNumberBase::TYPE_KEY=>InputValue::Number(CoreRegistry::get_double_handle(h,575)?),
        viewmodel_instance_boolean_base::ViewModelInstanceBooleanBase::TYPE_KEY=>InputValue::Boolean(CoreRegistry::get_bool_handle(h,593)?),
        viewmodel_instance_string_base::ViewModelInstanceStringBase::TYPE_KEY=>InputValue::String(CoreRegistry::get_string_handle(h,561)?),
        viewmodel_instance_color_base::ViewModelInstanceColorBase::TYPE_KEY=>InputValue::Color(CoreRegistry::get_color_handle(h,555)?),
        viewmodel_instance_enum_base::ViewModelInstanceEnumBase::TYPE_KEY=>InputValue::Index(CoreRegistry::get_uint_handle(h,560)?),
        viewmodel_instance_trigger_base::ViewModelInstanceTriggerBase::TYPE_KEY=>InputValue::Index(CoreRegistry::get_uint_handle(h,687)?),
        viewmodel_instance_symbol_list_index_base::ViewModelInstanceSymbolListIndexBase::TYPE_KEY=>InputValue::Index(CoreRegistry::get_uint_handle(h,814)?),_=>return None,
    })
}
fn copy_value(to: &CoreHandle, value: &InputValue) {
    match value {
        InputValue::Number(v) => {
            CoreRegistry::set_double_handle(to, 575, *v);
        }
        InputValue::Boolean(v) => {
            CoreRegistry::set_bool_handle(to, 593, *v);
        }
        InputValue::String(v) => {
            CoreRegistry::set_string_handle(to, 561, v.clone());
        }
        InputValue::Color(v) => {
            CoreRegistry::set_color_handle(to, 555, *v);
        }
        InputValue::Index(v) => {
            let key = to
                .with(|o| match o.core_type() {
                    viewmodel_instance_enum_base::ViewModelInstanceEnumBase::TYPE_KEY => 560,
                    viewmodel_instance_trigger_base::ViewModelInstanceTriggerBase::TYPE_KEY => 687,
                    _ => 814,
                })
                .unwrap();
            CoreRegistry::set_uint_handle(to, key, *v);
        }
    }
}
fn default_value(value: &InputValue) -> InputValue {
    match value {
        InputValue::Number(_) => InputValue::Number(0.0),
        InputValue::Boolean(_) => InputValue::Boolean(false),
        InputValue::String(_) => InputValue::String(String::new()),
        InputValue::Color(_) => InputValue::Color(0),
        InputValue::Index(_) => InputValue::Index(0),
    }
}
pub(crate) struct ShapeNotifier(pub Option<CoreHandle>);
impl TextStyleOwner for ShapeNotifier {
    fn style_shape_changed(&self) {
        if let Some(text) = &self.0 {
            text.with_mut(|t| t.text_interface_mark_shape_dirty());
        }
    }
}

pub struct TextStyleInstance {
    inputs: Vec<InputLink>,
    values_source: Option<CoreHandle>,
    values_lease: Option<CoreArena>,
    state_machine: Option<RuntimeStateMachineInstanceHandle>,
    instance: Option<RuntimeArtboardInstanceHandle>,
    view_model_instance: Option<CoreHandle>,
    view_model_lease: Option<CoreArena>,
    list_item: Option<CoreHandle>,
    parent: Option<RuntimeDataContextHandle>,
    paint: Option<CoreHandle>,
    text: Option<CoreHandle>,
    reads_outside_own_view_model: bool,
    is_bound: bool,
    _owner: Rc<dyn TextStyleOwner>,
}
impl TextStyleInstance {
    pub fn new(
        source: &CoreHandle,
        factory: RuntimeFactoryHandle,
        view_model_instance: Option<CoreHandle>,
        reads_outside_own_view_model: bool,
        text: Option<CoreHandle>,
        served_text: Option<CoreHandle>,
        served_style: Option<TextStyleReferenceWeak>,
    ) -> Self {
        let instance = Artboard::instance_from_handle_with_factory(source, Some(factory))
            .expect("live style source");
        let paint = TextStyleGlobal::first_text_style_paint(Some(&instance.core_handle()));
        let owner: Rc<dyn TextStyleOwner> = Rc::new(ShapeNotifier(text.clone()));
        if let Some(p) = &paint {
            p.with_mut(|p| {
                p.as_text_style_paint_mut()
                    .unwrap()
                    .set_owner(Some(Rc::downgrade(&owner)))
            });
        }
        let state_machine = instance.default_state_machine();
        if let (Some(sm), Some(text), Some(style)) = (&state_machine, served_text, served_style) {
            sm.with_instance_mut(|sm| sm.serve_text_style_weak(&text, &style));
        }
        let view_model_lease = view_model_instance
            .as_ref()
            .and_then(CoreHandle::retain_arena);
        Self {
            inputs: Vec::new(),
            values_source: None,
            values_lease: None,
            state_machine,
            instance: Some(instance),
            view_model_instance,
            view_model_lease,
            list_item: None,
            parent: None,
            paint,
            text,
            reads_outside_own_view_model,
            is_bound: false,
            _owner: owner,
        }
    }
    pub fn artboard_instance(&self) -> RuntimeArtboardInstanceHandle {
        self.instance.as_ref().unwrap().clone()
    }
    pub fn paint(&self) -> Option<CoreHandle> {
        self.paint.clone()
    }
    pub fn view_model_instance(&self) -> Option<CoreHandle> {
        self.view_model_instance.clone()
    }
    pub fn state_machine(&self) -> Option<RuntimeStateMachineInstanceHandle> {
        self.state_machine.clone()
    }
    pub fn bind(&mut self, mut parent: Option<RuntimeDataContextHandle>, force: bool) {
        let same = match (&self.parent, &parent) {
            (None, None) => true,
            (Some(a), Some(b)) => a.ptr_eq(b),
            _ => false,
        };
        if self.is_bound && !force && same && !self.reads_outside_own_view_model {
            return;
        }
        self.is_bound = true;
        self.parent = parent.clone();
        if let Some(item) = &self.list_item {
            let mut context = DataContext::new(Some(item.clone()));
            context.set_parent(parent);
            parent = Some(RuntimeDataContextHandle::new(context));
        }
        let instance = self.artboard_instance();
        if let Some(vm) = &self.view_model_instance {
            instance.bind_view_model_instances(vec![vm.clone()], parent);
        } else if let Some(parent) = parent {
            instance.internal_data_context(parent);
        }
        if let Some(sm) = &self.state_machine {
            sm.with_instance_mut(|sm| {
                if let Some(context) = instance.data_context() {
                    sm.set_data_context(context)
                } else {
                    sm.clear_data_context()
                }
            });
        }
    }
    pub fn relink(&mut self) {
        if self.reads_outside_own_view_model {
            self.bind(self.parent.clone(), true);
        }
    }
    pub fn list_item(&mut self, value: Option<CoreHandle>) {
        if value == self.list_item {
            return;
        }
        self.list_item = value;
        if self.is_bound {
            self.bind(self.parent.clone(), true);
            self.flush();
        }
    }
    pub fn flush(&mut self) -> bool {
        let instance = self.artboard_instance();
        instance.update_data_binds(false);
        if !instance.with_artboard(|a| a.has_component_dirt()) {
            return false;
        }
        Artboard::update_pass_handle(&instance.core_handle(), false);
        self._owner.style_shape_changed();
        true
    }
    pub fn advance(&mut self, seconds: f32, flags: AdvanceFlags) -> bool {
        self.sync_inputs();
        let mut keep = false;
        if flags.contains(AdvanceFlags::ADVANCE_NESTED) {
            if let Some(sm) = &self.state_machine {
                keep |= sm.with_instance_mut(|sm| {
                    sm.advance(seconds, flags.contains(AdvanceFlags::NEW_FRAME))
                });
            }
        }
        keep |= self
            .artboard_instance()
            .advance_internal(seconds, flags & !AdvanceFlags::IS_ROOT);
        self.flush();
        keep
    }
    pub fn quiet_state(&self) -> QuietState {
        if self.state_machine.is_some() || self.reads_outside_own_view_model {
            QuietState::Never
        } else if self
            .artboard_instance()
            .with_artboard(|a| a.has_component_dirt())
        {
            QuietState::Busy
        } else {
            QuietState::Quiet
        }
    }
    pub fn copy_values_from(&mut self, source: Option<CoreHandle>) {
        self.inputs.clear();
        self.values_lease = source.as_ref().and_then(CoreHandle::retain_arena);
        self.values_source = source;
        let (Some(source), Some(target)) = (&self.values_source, &self.view_model_instance) else {
            return;
        };
        let froms = source
            .with_downcast::<ViewModelInstance, _>(|v| v.property_values().to_vec())
            .unwrap();
        let targets = target
            .with_downcast::<ViewModelInstance, _>(|v| v.property_values().to_vec())
            .unwrap();
        for from in froms {
            let Some(value) = read_input(&from) else {
                continue;
            };
            let (kind, id) = from
                .with(|v| {
                    (
                        v.core_type(),
                        v.as_view_model_instance_value()
                            .unwrap()
                            .view_model_property_id(),
                    )
                })
                .unwrap();
            for to in &targets {
                let matches = to
                    .with(|v| {
                        v.core_type() == kind
                            && v.as_view_model_instance_value()
                                .unwrap()
                                .view_model_property_id()
                                == id
                    })
                    .unwrap();
                if !matches {
                    continue;
                }
                copy_value(to, &value);
                let property = from
                    .with(|v| {
                        v.as_view_model_instance_value()
                            .unwrap()
                            .view_model_property()
                    })
                    .flatten()
                    .or_else(|| {
                        to.with(|v| {
                            v.as_view_model_instance_value()
                                .unwrap()
                                .view_model_property()
                        })
                        .flatten()
                    });
                if property.is_some_and(|p| {
                    p.with(|p| p.as_view_model_property().unwrap().is_input())
                        .unwrap()
                }) {
                    self.inputs.push(InputLink {
                        from: from.clone(),
                        to: to.clone(),
                        last: default_value(&value),
                    });
                }
                break;
            }
        }
        self.sync_inputs();
    }
    pub fn sync_inputs(&mut self) -> bool {
        let mut changed = false;
        for link in &mut self.inputs {
            if let Some(value) = read_input(&link.from) {
                let differs = value != link.last;
                link.last = value.clone();
                if differs {
                    copy_value(&link.to, &value);
                    changed = true;
                }
            }
        }
        changed
    }
}
impl Drop for TextStyleInstance {
    fn drop(&mut self) {
        self.inputs.clear();
        self.values_source = None;
        self.values_lease = None;
        self.state_machine = None;
        self.instance = None;
        self.view_model_instance = None;
        self.view_model_lease = None;
        self.list_item = None;
        self.parent = None;
    }
}
