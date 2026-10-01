use crate::mechanical_port::source::{
    core::CoreHandle,
    core_context::CoreContext,
    generated::{animation::state_machine_base::StateMachineBase, artboard_base::ArtboardBase},
    importers::{artboard_importer::ArtboardImporter, import_stack::ImportStack},
    status_code::StatusCode,
};
use std::collections::HashSet;

fn is_tracked_comparator(comparator: Option<CoreHandle>, untracked: &HashSet<CoreHandle>) -> bool {
    use crate::mechanical_port::source::generated::{animation::*, data_bind::*};
    let Some(comparator) = comparator else {
        return true;
    };
    if untracked.contains(&comparator) {
        return false;
    }
    match comparator.core_type() {
        Some(transition_property_viewmodel_comparator_base::TransitionPropertyViewModelComparatorBase::TYPE_KEY) => {
            let Some(bindable) = comparator.with(|c| c.transition_comparator_bindable_property()).flatten() else { return false };
            if untracked.contains(&bindable) { return false; }
            matches!(bindable.core_type(),
                Some(bindable_property_number_base::BindablePropertyNumberBase::TYPE_KEY
                | bindable_property_integer_base::BindablePropertyIntegerBase::TYPE_KEY
                | bindable_property_boolean_base::BindablePropertyBooleanBase::TYPE_KEY
                | bindable_property_string_base::BindablePropertyStringBase::TYPE_KEY
                | bindable_property_color_base::BindablePropertyColorBase::TYPE_KEY
                | bindable_property_enum_base::BindablePropertyEnumBase::TYPE_KEY
                | bindable_property_trigger_base::BindablePropertyTriggerBase::TYPE_KEY))
        }
        Some(transition_value_number_comparator_base::TransitionValueNumberComparatorBase::TYPE_KEY
        | transition_value_boolean_comparator_base::TransitionValueBooleanComparatorBase::TYPE_KEY
        | transition_value_string_comparator_base::TransitionValueStringComparatorBase::TYPE_KEY
        | transition_value_color_comparator_base::TransitionValueColorComparatorBase::TYPE_KEY
        | transition_value_enum_comparator_base::TransitionValueEnumComparatorBase::TYPE_KEY
        | transition_value_trigger_comparator_base::TransitionValueTriggerComparatorBase::TYPE_KEY
        | transition_value_asset_comparator_base::TransitionValueAssetComparatorBase::TYPE_KEY
        | transition_value_artboard_comparator_base::TransitionValueArtboardComparatorBase::TYPE_KEY
        | transition_self_comparator_base::TransitionSelfComparatorBase::TYPE_KEY) => true,
        _ => false,
    }
}

fn is_tracked_condition(condition: Option<CoreHandle>, untracked: &HashSet<CoreHandle>) -> bool {
    use crate::mechanical_port::source::{
        animation::transition_viewmodel_condition::TransitionViewModelCondition,
        generated::animation::transition_viewmodel_condition_base::TransitionViewModelConditionBase,
    };
    let Some(condition) = condition else {
        return false;
    };
    if condition.core_type() != Some(TransitionViewModelConditionBase::TYPE_KEY) {
        return false;
    }
    condition
        .with_downcast::<TransitionViewModelCondition, _>(|condition| {
            is_tracked_comparator(condition.left_comparator(), untracked)
                && is_tracked_comparator(condition.right_comparator(), untracked)
        })
        .unwrap_or(false)
}

fn classify_state(state: &CoreHandle, untracked: &HashSet<CoreHandle>) -> (bool, bool) {
    let (flags, transitions) = state
        .with(|state| {
            (
                state.layer_state_flags().expect("LayerState flags"),
                (0..state
                    .layer_state_transition_count()
                    .expect("LayerState transitions"))
                    .filter_map(|i| state.layer_state_transition(i))
                    .collect::<Vec<_>>(),
            )
        })
        .expect("live layer state");
    let mut safe = flags & 1 == 0;
    let mut ignore_time = true;
    for transition in transitions {
        transition.with(|transition| {
            let transition = transition.as_state_transition().expect("StateTransition");
            if transition.is_disabled() {
                return;
            }
            if transition.enable_exit_time() {
                ignore_time = false;
            }
            for i in 0..transition.condition_count() {
                if !is_tracked_condition(transition.condition(i), untracked) {
                    safe = false;
                }
            }
        });
    }
    (safe, ignore_time)
}

#[derive(Default)]
pub struct StateMachine {
    pub base: StateMachineBase,
    layers: Vec<CoreHandle>,
    inputs: Vec<Option<CoreHandle>>,
    listeners: Vec<CoreHandle>,
    data_binds: Vec<CoreHandle>,
    scripted_objects: Vec<CoreHandle>,
}
impl StateMachine {
    pub fn set_name(&mut self, value: String) {
        use crate::mechanical_port::source::generated::animation::animation_base::{
            AnimationBase, AnimationBaseCallbacks,
        };
        if self.base.set_name_value(value) {
            AnimationBaseCallbacks::name_changed(self);
            AnimationBaseCallbacks::notify_property_changed(self, AnimationBase::NAME_PROPERTY_KEY);
        }
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        for input in self.inputs.iter().filter_map(Clone::clone) {
            let code = input
                .with_mut(|input| input.on_added_dirty(context))
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        for layer in self.layers.iter().cloned() {
            let code = layer
                .with_downcast_mut::<
                    crate::mechanical_port::source::animation::state_machine_layer::StateMachineLayer,
                    _,
                >(|layer| layer.on_added_dirty(context))
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        for listener in self.listeners.iter().cloned() {
            let code = listener
                .with_mut(|listener| listener.on_added_dirty(context))
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        for input in self.inputs.iter().filter_map(Clone::clone) {
            let code = input
                .with_mut(|input| input.on_added_clean(context))
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        for layer in self.layers.iter().cloned() {
            let code = layer
                .with_downcast_mut::<
                    crate::mechanical_port::source::animation::state_machine_layer::StateMachineLayer,
                    _,
                >(|layer| layer.on_added_clean(context))
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        for listener in self.listeners.iter().cloned() {
            let code = listener
                .with_mut(|listener| listener.on_added_clean(context))
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        let mut untracked = HashSet::new();
        for data_bind in &self.data_binds {
            data_bind.with(|data_bind| {
                let data_bind = data_bind.as_data_bind().expect("StateMachine DataBind");
                if let Some(target) = data_bind.target() {
                    if data_bind.flags() & 4 != 0 || !target.is_type_of(crate::mechanical_port::source::generated::data_bind::bindable_property_base::BindablePropertyBase::TYPE_KEY) {
                        untracked.insert(target);
                    }
                }
            });
        }
        for layer in &self.layers {
            let states = layer.with_downcast::<crate::mechanical_port::source::animation::state_machine_layer::StateMachineLayer, _>(|layer| {
                (0..layer.state_count()).filter_map(|i| layer.state(i)).collect::<Vec<_>>()
            }).expect("StateMachineLayer");
            for state in states {
                let (safe, ignore_time) = classify_state(&state, &untracked);
                state.with_mut(|state| state.set_layer_state_settle_flags(safe, ignore_time));
            }
        }
        StatusCode::Ok
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = stack.latest::<ArtboardImporter>(ArtboardBase::TYPE_KEY) else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_state_machine(this);
        self.base.base.import(stack)
    }
    pub(crate) fn add_layer(&mut self, value: CoreHandle) {
        self.layers.push(value);
    }
    pub(crate) fn add_input(&mut self, value: Option<CoreHandle>) {
        self.inputs.push(value);
    }
    pub(crate) fn add_listener(&mut self, value: CoreHandle) {
        self.listeners.push(value);
    }
    pub(crate) fn add_data_bind(&mut self, value: CoreHandle) {
        self.data_binds.push(value);
    }
    pub(crate) fn add_scripted_object(&mut self, value: CoreHandle) {
        self.scripted_objects.push(value);
    }
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }
    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }
    pub fn data_bind_count(&self) -> usize {
        self.data_binds.len()
    }
    pub fn scripted_objects(&self) -> Vec<CoreHandle> {
        self.scripted_objects.clone()
    }
    pub fn input(&self, index: usize) -> Option<CoreHandle> {
        self.inputs.get(index).and_then(Clone::clone)
    }
    pub fn input_named(&self, name: &str) -> Option<CoreHandle> {
        self.inputs.iter().filter_map(Clone::clone).find(|input| {
            input
                .with(|input| input.state_machine_input_name().as_deref() == Some(name))
                .unwrap_or(false)
        })
    }
    pub fn layer(&self, index: usize) -> Option<CoreHandle> {
        self.layers.get(index).cloned()
    }
    pub fn layer_named(&self, name: &str) -> Option<CoreHandle> {
        self.layers
            .iter()
            .find(|layer| {
                layer
                    .with(|layer| layer.state_machine_component_name().as_deref() == Some(name))
                    .unwrap_or(false)
            })
            .cloned()
    }
    pub fn listener(&self, index: usize) -> Option<CoreHandle> {
        self.listeners.get(index).cloned()
    }
    pub fn data_bind(&self, index: usize) -> Option<CoreHandle> {
        self.data_binds.get(index).cloned()
    }
}

impl std::ops::Deref for StateMachine {
    type Target = StateMachineBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for StateMachine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
