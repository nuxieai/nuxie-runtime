//! Retire animation definitions owned by a replaced source artboard.
//!
//! Core handles are arena references, not C++ owning pointers. Follow only the
//! owning collections/destructor bodies here; referenced animations, target
//! states, interpolators and view models remain owned by their original owner.

use super::{
    blend_animation_direct::BlendAnimationDirect, blend_state_1d_viewmodel::BlendState1DViewModel,
    keyed_object::KeyedObject, keyed_property::KeyedProperty, linear_animation::LinearAnimation,
    state_machine::StateMachine, state_machine_layer::StateMachineLayer,
    transition_artboard_condition::TransitionArtboardCondition,
    transition_focus_condition::TransitionFocusCondition,
    transition_property_viewmodel_comparator::TransitionPropertyViewModelComparator,
    transition_viewmodel_condition::TransitionViewModelCondition,
};
use crate::mechanical_port::source::core::CoreHandle;

pub(crate) fn dispose_animation_source(animation: &CoreHandle) {
    let children = animation
        .with_downcast::<LinearAnimation, _>(|animation| animation.keyed_objects().to_vec())
        .unwrap_or_default();
    // LinearAnimation's destructor body (including the testing delete counter)
    // precedes destruction of its unique_ptr keyed-object collection.
    animation.remove_occurrence();
    for child in children {
        dispose_owned_child(&child);
    }
}

pub(crate) fn dispose_state_machine_source(machine: &CoreHandle) {
    let children = machine
        .with_downcast::<StateMachine, _>(|machine| {
            let mut children = Vec::new();
            // Member destruction is reverse declaration order; each owning
            // vector disposes its elements in their stored order.
            children.extend((0..machine.data_bind_count()).filter_map(|i| machine.data_bind(i)));
            children.extend((0..machine.listener_count()).filter_map(|i| machine.listener(i)));
            children.extend((0..machine.input_count()).filter_map(|i| machine.input(i)));
            children.extend((0..machine.layer_count()).filter_map(|i| machine.layer(i)));
            children
        })
        .unwrap_or_default();
    for child in children {
        dispose_owned_child(&child);
    }
    machine.remove_occurrence();
}

fn dispose_owned_child(handle: &CoreHandle) {
    // Release every arena borrow before descending or invoking a destructor.
    let children = handle
        .with(|object| {
            let mut children = Vec::new();
            let any = object.as_any();
            if let Some(object) = any.downcast_ref::<KeyedObject>() {
                children.extend_from_slice(object.keyed_properties());
            } else if let Some(property) = any.downcast_ref::<KeyedProperty>() {
                children.extend_from_slice(property.keyframes());
            } else if let Some(layer) = any.downcast_ref::<StateMachineLayer>() {
                children.extend_from_slice(layer.states());
            } else if let Some(listener) = object.as_state_machine_listener() {
                children.extend(
                    (0..listener.listener_input_type_count())
                        .filter_map(|i| listener.listener_input_type(i)),
                );
                children.extend((0..listener.action_count()).filter_map(|i| listener.action(i)));
            } else if let Some(condition) = any
                .downcast_ref::<TransitionViewModelCondition>()
                .or_else(|| {
                    any.downcast_ref::<TransitionArtboardCondition>()
                        .map(|condition| &condition.base.base)
                })
                .or_else(|| {
                    any.downcast_ref::<TransitionFocusCondition>()
                        .map(|condition| &condition.base.base)
                })
            {
                children.extend(condition.left_comparator());
                children.extend(condition.right_comparator());
            } else if let Some(comparator) =
                any.downcast_ref::<TransitionPropertyViewModelComparator>()
            {
                children.extend(comparator.bindable_property());
            } else if let Some(animation) = any.downcast_ref::<BlendAnimationDirect>() {
                children.extend(animation.bindable_property());
            }

            // Derived destructor bodies run before their base destructor:
            // bindable property, then blend animations, then state transitions.
            if let Some(state) = any.downcast_ref::<BlendState1DViewModel>() {
                children.extend(state.bindable_property());
            }
            if let Some(animations) = object.blend_state_animations() {
                children.extend(animations);
            }
            if let Some(count) = object.layer_state_transition_count() {
                children.extend((0..count).filter_map(|i| object.layer_state_transition(i)));
            }
            if let Some(transition) = object.as_state_transition() {
                children.extend(
                    (0..transition.condition_count()).filter_map(|i| transition.condition(i)),
                );
            }
            // StateMachineLayerComponent is the base of states and transitions.
            // Its destructor body deletes fire events before its owning action
            // vector is destroyed (definition lives in the importer cpp).
            if let Some(events) = object.state_machine_layer_component_events() {
                children.extend(events);
            }
            if let Some(actions) = object.state_machine_layer_component_listener_actions() {
                children.extend(actions);
            }
            children
        })
        .unwrap_or_default();
    for child in children {
        dispose_owned_child(&child);
    }
    // Existing action/condition Drop implementations retain responsibility for
    // their script inputs and ListenerViewModelChange's bindable property.
    handle.remove_occurrence();
}
