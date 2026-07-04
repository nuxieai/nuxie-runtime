// Runtime instance orchestration for the C++ state machine path.
// Mirrors /Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp.
use super::*;
use crate::{
    ArtboardInstance, RuntimeDataBindGraph, RuntimeDataBindGraphApplyPhase,
    RuntimeDataBindGraphTargetsMut, RuntimeDefaultViewModelArtboardSourceHandle,
    RuntimeDefaultViewModelAssetSourceHandle, RuntimeDefaultViewModelBooleanSourceHandle,
    RuntimeDefaultViewModelColorSourceHandle, RuntimeDefaultViewModelEnumSourceHandle,
    RuntimeDefaultViewModelListSourceHandle, RuntimeDefaultViewModelNumberSourceHandle,
    RuntimeDefaultViewModelStringSourceHandle, RuntimeDefaultViewModelSymbolListIndexSourceHandle,
    RuntimeDefaultViewModelTriggerSourceHandle, RuntimeDefaultViewModelViewModelSourceHandle,
    RuntimeImportedViewModelInstanceContext, RuntimeOwnedViewModelInstance,
    runtime_default_view_model_artboard_property_path_for_name,
    runtime_default_view_model_artboard_property_path_for_name_path,
    runtime_default_view_model_asset_property_path_for_name,
    runtime_default_view_model_asset_property_path_for_name_path,
    runtime_default_view_model_boolean_property_path_for_name,
    runtime_default_view_model_boolean_property_path_for_name_path,
    runtime_default_view_model_color_property_path_for_name,
    runtime_default_view_model_color_property_path_for_name_path,
    runtime_default_view_model_enum_property_path_for_name,
    runtime_default_view_model_enum_property_path_for_name_path,
    runtime_default_view_model_list_property_path_for_name,
    runtime_default_view_model_list_property_path_for_name_path,
    runtime_default_view_model_number_property_path_for_name,
    runtime_default_view_model_number_property_path_for_name_path,
    runtime_default_view_model_string_property_path_for_name,
    runtime_default_view_model_string_property_path_for_name_path,
    runtime_default_view_model_symbol_list_index_property_path_for_name,
    runtime_default_view_model_symbol_list_index_property_path_for_name_path,
    runtime_default_view_model_trigger_property_path_for_name,
    runtime_default_view_model_trigger_property_path_for_name_path,
    runtime_default_view_model_view_model_property_path_for_name,
    runtime_default_view_model_view_model_property_path_for_name_path,
};
use rive_binary::RuntimeFile;

#[derive(Debug, Clone)]
pub struct StateMachineInstance {
    state_machine_index: usize,
    inputs: Vec<StateMachineInputInstance>,
    bindable_numbers: Vec<StateMachineBindableNumberInstance>,
    bindable_integers: Vec<StateMachineBindableIntegerInstance>,
    bindable_colors: Vec<StateMachineBindableColorInstance>,
    bindable_strings: Vec<StateMachineBindableStringInstance>,
    bindable_enums: Vec<StateMachineBindableEnumInstance>,
    bindable_assets: Vec<StateMachineBindableAssetInstance>,
    bindable_artboards: Vec<StateMachineBindableArtboardInstance>,
    bindable_lists: Vec<StateMachineBindableListInstance>,
    bindable_triggers: Vec<StateMachineBindableTriggerInstance>,
    bindable_view_models: Vec<StateMachineBindableViewModelInstance>,
    bindable_booleans: Vec<StateMachineBindableBooleanInstance>,
    default_view_model_triggers: Vec<StateMachineViewModelTriggerInstance>,
    view_model_triggers: Vec<StateMachineViewModelTriggerInstance>,
    layers: Vec<StateMachineLayerInstance>,
    reported_events: Vec<StateMachineReportedEvent>,
    changed_state_count: usize,
    needs_advance: bool,
    data_bind_graph: RuntimeDataBindGraph,
    pointer_down_listener_hits: Vec<RuntimePointerDownListenerHit>,
    pointer_listener_states: Vec<RuntimePointerListenerState>,
}

#[derive(Debug, Clone)]
struct RuntimePointerDownListenerHit {
    pointer_id: i32,
    listener_index: usize,
}

#[derive(Debug, Clone)]
struct RuntimePointerListenerState {
    pointer_id: i32,
    listener_index: usize,
    is_hovered: bool,
}

impl StateMachineInstance {
    pub(crate) fn new(
        state_machine_index: usize,
        state_machine: &RuntimeStateMachine,
        artboard: &ArtboardInstance,
    ) -> Self {
        let inputs = state_machine
            .inputs
            .iter()
            .enumerate()
            .map(|(index, input)| StateMachineInputInstance::new(index, input))
            .collect::<Vec<_>>();
        let bindable_numbers = state_machine
            .bindable_numbers
            .iter()
            .map(StateMachineBindableNumberInstance::new)
            .collect::<Vec<_>>();
        let bindable_integers = state_machine
            .bindable_integers
            .iter()
            .map(StateMachineBindableIntegerInstance::new)
            .collect::<Vec<_>>();
        let bindable_colors = state_machine
            .bindable_colors
            .iter()
            .map(StateMachineBindableColorInstance::new)
            .collect::<Vec<_>>();
        let bindable_strings = state_machine
            .bindable_strings
            .iter()
            .map(StateMachineBindableStringInstance::new)
            .collect::<Vec<_>>();
        let bindable_enums = state_machine
            .bindable_enums
            .iter()
            .map(StateMachineBindableEnumInstance::new)
            .collect::<Vec<_>>();
        let bindable_assets = state_machine
            .bindable_assets
            .iter()
            .map(StateMachineBindableAssetInstance::new)
            .collect::<Vec<_>>();
        let bindable_artboards = state_machine
            .bindable_artboards
            .iter()
            .map(StateMachineBindableArtboardInstance::new)
            .collect::<Vec<_>>();
        let bindable_lists = state_machine
            .bindable_lists
            .iter()
            .map(StateMachineBindableListInstance::new)
            .collect::<Vec<_>>();
        let bindable_triggers = state_machine
            .bindable_triggers
            .iter()
            .map(StateMachineBindableTriggerInstance::new)
            .collect::<Vec<_>>();
        let bindable_view_models = state_machine
            .bindable_view_models
            .iter()
            .map(StateMachineBindableViewModelInstance::new)
            .collect::<Vec<_>>();
        let bindable_booleans = state_machine
            .bindable_booleans
            .iter()
            .map(StateMachineBindableBooleanInstance::new)
            .collect::<Vec<_>>();
        let view_model_triggers = state_machine
            .view_model_triggers
            .iter()
            .map(|trigger| {
                StateMachineViewModelTriggerInstance::new(
                    trigger.global_id,
                    trigger.view_model_property_id,
                    trigger.value,
                )
            })
            .collect::<Vec<_>>();
        let default_view_model_triggers = view_model_triggers.clone();
        let layers = state_machine
            .layers
            .iter()
            .map(|layer| {
                StateMachineLayerInstance::new(layer, artboard, &inputs, &bindable_numbers)
            })
            .collect();
        Self {
            state_machine_index,
            inputs,
            bindable_numbers,
            bindable_integers,
            bindable_colors,
            bindable_strings,
            bindable_enums,
            bindable_assets,
            bindable_artboards,
            bindable_lists,
            bindable_triggers,
            bindable_view_models,
            bindable_booleans,
            default_view_model_triggers,
            view_model_triggers,
            layers,
            reported_events: Vec::new(),
            changed_state_count: 0,
            needs_advance: false,
            data_bind_graph: RuntimeDataBindGraph::new(state_machine),
            pointer_down_listener_hits: Vec::new(),
            pointer_listener_states: Vec::new(),
        }
    }

    pub fn state_machine_index(&self) -> usize {
        self.state_machine_index
    }

    pub fn changed_state_count(&self) -> usize {
        self.changed_state_count
    }

    pub fn needs_advance(&self) -> bool {
        self.needs_advance
    }

    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    pub fn input(&self, index: usize) -> Option<&StateMachineInputInstance> {
        self.inputs.get(index)
    }

    pub fn input_named(&self, name: &str) -> Option<&StateMachineInputInstance> {
        self.inputs.iter().find(|input| input.name() == Some(name))
    }

    pub fn input_index_named(&self, name: &str) -> Option<usize> {
        self.inputs
            .iter()
            .position(|input| input.name() == Some(name))
    }

    pub fn set_bool(&mut self, index: usize, value: bool) -> bool {
        let Some(input) = self.inputs.get_mut(index) else {
            return false;
        };
        if !input.set_bool(value) {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_number(&mut self, index: usize, value: f32) -> bool {
        let Some(input) = self.inputs.get_mut(index) else {
            return false;
        };
        if !input.set_number(value) {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn fire_trigger(&mut self, index: usize) -> bool {
        let Some(input) = self.inputs.get_mut(index) else {
            return false;
        };
        if !input.fire_trigger() {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn pointer_down(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        self.pointer_down_listener_hits
            .retain(|hit| hit.pointer_id != pointer_id);
        let Some(state_machine) = artboard.state_machine(self.state_machine_index) else {
            return false;
        };

        let mut hit = false;
        for (listener_index, listener) in state_machine.listeners.iter().enumerate() {
            let listener_hit = listener.hit_test(artboard, x, y);
            let hover_action = self.update_pointer_listener_hover(
                listener_index,
                listener,
                listener_hit,
                pointer_id,
            );
            let click_action = listener_hit && listener.has_listener(RuntimeListenerType::Click);
            if click_action {
                self.pointer_down_listener_hits
                    .push(RuntimePointerDownListenerHit {
                        pointer_id,
                        listener_index,
                    });
            }
            let direct_action = listener_hit && listener.has_listener(RuntimeListenerType::Down);
            let action_type =
                hover_action.or_else(|| direct_action.then_some(RuntimeListenerType::Down));
            if listener_hit && (click_action || direct_action || hover_action.is_some()) {
                hit = true;
            }
            if action_type.is_some()
                && self.perform_pointer_listener_actions(&listener.listener_actions)
            {
                self.needs_advance = true;
            }
        }
        hit
    }

    pub fn pointer_move(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        _seconds: f32,
        pointer_id: i32,
    ) -> bool {
        self.update_pointer_listeners(artboard, RuntimeListenerType::Move, x, y, pointer_id)
    }

    pub fn pointer_up(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        let Some(state_machine) = artboard.state_machine(self.state_machine_index) else {
            self.pointer_down_listener_hits
                .retain(|hit| hit.pointer_id != pointer_id);
            return false;
        };

        let mut hit = false;
        for (listener_index, listener) in state_machine.listeners.iter().enumerate() {
            let listener_hit = listener.hit_test(artboard, x, y);
            let hover_action = self.update_pointer_listener_hover(
                listener_index,
                listener,
                listener_hit,
                pointer_id,
            );
            let click_matched = listener.has_listener(RuntimeListenerType::Click)
                && self.pointer_down_listener_hits.iter().any(|hit| {
                    hit.pointer_id == pointer_id && hit.listener_index == listener_index
                });
            let direct_action = listener_hit && listener.has_listener(RuntimeListenerType::Up);
            let action_type = hover_action
                .or_else(|| (listener_hit && click_matched).then_some(RuntimeListenerType::Click))
                .or_else(|| direct_action.then_some(RuntimeListenerType::Up));
            if listener_hit && (click_matched || direct_action || hover_action.is_some()) {
                hit = true;
            }
            if action_type.is_some()
                && self.perform_pointer_listener_actions(&listener.listener_actions)
            {
                self.needs_advance = true;
            }
        }
        self.pointer_down_listener_hits
            .retain(|hit| hit.pointer_id != pointer_id);
        hit
    }

    pub fn pointer_exit(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        let hit =
            self.update_pointer_listeners(artboard, RuntimeListenerType::Exit, x, y, pointer_id);
        self.pointer_listener_states
            .retain(|state| state.pointer_id != pointer_id);
        hit
    }

    fn update_pointer_listeners(
        &mut self,
        artboard: &ArtboardInstance,
        listener_type: RuntimeListenerType,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        let Some(state_machine) = artboard.state_machine(self.state_machine_index) else {
            return false;
        };

        let mut hit = false;
        for (listener_index, listener) in state_machine.listeners.iter().enumerate() {
            let listener_hit =
                listener_type != RuntimeListenerType::Exit && listener.hit_test(artboard, x, y);
            let hover_action = self.update_pointer_listener_hover(
                listener_index,
                listener,
                listener_hit,
                pointer_id,
            );
            let direct_action = listener_hit && listener.has_listener(listener_type);
            let action_type = hover_action.or_else(|| direct_action.then_some(listener_type));
            if listener_hit && (direct_action || hover_action.is_some()) {
                hit = true;
            }
            if action_type.is_some()
                && self.perform_pointer_listener_actions(&listener.listener_actions)
            {
                self.needs_advance = true;
            }
        }
        hit
    }

    fn update_pointer_listener_hover(
        &mut self,
        listener_index: usize,
        listener: &RuntimeStateMachineListener,
        is_hovered: bool,
        pointer_id: i32,
    ) -> Option<RuntimeListenerType> {
        let state_index = self.pointer_listener_states.iter().position(|state| {
            state.pointer_id == pointer_id && state.listener_index == listener_index
        });
        let was_hovered = state_index
            .map(|index| self.pointer_listener_states[index].is_hovered)
            .unwrap_or(false);

        match state_index {
            Some(index) => self.pointer_listener_states[index].is_hovered = is_hovered,
            None => self
                .pointer_listener_states
                .push(RuntimePointerListenerState {
                    pointer_id,
                    listener_index,
                    is_hovered,
                }),
        }

        match (was_hovered, is_hovered) {
            (false, true) if listener.has_listener(RuntimeListenerType::Enter) => {
                Some(RuntimeListenerType::Enter)
            }
            (true, false) if listener.has_listener(RuntimeListenerType::Exit) => {
                Some(RuntimeListenerType::Exit)
            }
            _ => None,
        }
    }

    fn perform_pointer_listener_actions(
        &mut self,
        listener_actions: &[RuntimeScheduledListenerAction],
    ) -> bool {
        let mut changed = false;
        for action in listener_actions {
            match action {
                RuntimeScheduledListenerAction::FireEvent { event, .. } => {
                    self.reported_events.push(event.clone());
                    changed = true;
                }
                RuntimeScheduledListenerAction::BoolChange {
                    input_index, value, ..
                } => {
                    if let Some(input) = self.inputs.get_mut(*input_index) {
                        changed |= input.apply_listener_bool_change(*value);
                    }
                }
                RuntimeScheduledListenerAction::NumberChange {
                    input_index, value, ..
                } => {
                    if let Some(input) = self.inputs.get_mut(*input_index) {
                        changed |= input.set_number(*value);
                    }
                }
                RuntimeScheduledListenerAction::TriggerChange { input_index, .. } => {
                    if let Some(input) = self.inputs.get_mut(*input_index) {
                        changed |= input.fire_trigger();
                    }
                }
                RuntimeScheduledListenerAction::ViewModelChange {
                    data_bind_index,
                    value,
                    ..
                } => {
                    changed |= self.perform_listener_view_model_change(*data_bind_index, value);
                }
            }
        }
        if changed {
            self.needs_advance = true;
        }
        changed
    }

    fn perform_listener_view_model_change(
        &mut self,
        data_bind_index: usize,
        value: &RuntimeListenerViewModelChangeValue,
    ) -> bool {
        match value {
            RuntimeListenerViewModelChangeValue::Number(value) => {
                self.set_default_view_model_number_source_for_data_bind(data_bind_index, *value)
            }
            RuntimeListenerViewModelChangeValue::Integer(value) => self
                .set_default_view_model_symbol_list_index_source_for_data_bind(
                    data_bind_index,
                    *value,
                ),
            RuntimeListenerViewModelChangeValue::Color(value) => {
                self.set_default_view_model_color_source_for_data_bind(data_bind_index, *value)
            }
            RuntimeListenerViewModelChangeValue::String(value) => {
                self.set_default_view_model_string_source_for_data_bind(data_bind_index, value)
            }
            RuntimeListenerViewModelChangeValue::Enum(value) => {
                self.set_default_view_model_enum_source_for_data_bind(data_bind_index, *value)
            }
            RuntimeListenerViewModelChangeValue::Asset(value) => {
                self.set_default_view_model_asset_source_for_data_bind(data_bind_index, *value)
            }
            RuntimeListenerViewModelChangeValue::Artboard(value) => {
                self.set_default_view_model_artboard_source_for_data_bind(data_bind_index, *value)
            }
            RuntimeListenerViewModelChangeValue::Trigger(value) => {
                self.perform_listener_trigger_view_model_change(data_bind_index, *value)
            }
            RuntimeListenerViewModelChangeValue::Boolean(value) => {
                self.set_default_view_model_boolean_source_for_data_bind(data_bind_index, *value)
            }
        }
    }

    fn perform_listener_trigger_view_model_change(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(bindable_trigger) = self
            .bindable_triggers
            .iter_mut()
            .find(|bindable_trigger| bindable_trigger.has_data_bind_index(data_bind_index))
        else {
            return false;
        };

        // Mirrors src/animation/listener_viewmodel_change.cpp: listener
        // actions invalidate the target-to-source binding even when the
        // trigger target value itself did not change.
        bindable_trigger.set_value(value);
        if !self
            .data_bind_graph
            .mark_trigger_target_dirty_for_data_bind(data_bind_index)
        {
            return false;
        }
        if !self.update_data_binds_apply_target_to_source() {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_number_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: f32,
    ) -> bool {
        let Some(bindable_number) = self
            .bindable_numbers
            .iter_mut()
            .find(|bindable_number| bindable_number.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_number.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_number_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_boolean_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: bool,
    ) -> bool {
        let Some(bindable_boolean) = self
            .bindable_booleans
            .iter_mut()
            .find(|bindable_boolean| bindable_boolean.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_boolean.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_boolean_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_integer_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(bindable_integer) = self
            .bindable_integers
            .iter_mut()
            .find(|bindable_integer| bindable_integer.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_integer.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_integer_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_color_for_data_bind(&mut self, data_bind_index: usize, value: u32) -> bool {
        let Some(bindable_color) = self
            .bindable_colors
            .iter_mut()
            .find(|bindable_color| bindable_color.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_color.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_color_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_string_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: &[u8],
    ) -> bool {
        let Some(bindable_string) = self
            .bindable_strings
            .iter_mut()
            .find(|bindable_string| bindable_string.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_string.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_string_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_enum_for_data_bind(&mut self, data_bind_index: usize, value: u64) -> bool {
        let Some(bindable_enum) = self
            .bindable_enums
            .iter_mut()
            .find(|bindable_enum| bindable_enum.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_enum.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_enum_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_asset_for_data_bind(&mut self, data_bind_index: usize, value: u64) -> bool {
        let Some(bindable_asset) = self
            .bindable_assets
            .iter_mut()
            .find(|bindable_asset| bindable_asset.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_asset.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_asset_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_artboard_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(bindable_artboard) = self
            .bindable_artboards
            .iter_mut()
            .find(|bindable_artboard| bindable_artboard.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_artboard.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_artboard_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_list_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: usize,
    ) -> bool {
        let Some(bindable_list) = self
            .bindable_lists
            .iter_mut()
            .find(|bindable_list| bindable_list.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_list.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_list_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_trigger_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(bindable_trigger) = self
            .bindable_triggers
            .iter_mut()
            .find(|bindable_trigger| bindable_trigger.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_trigger.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_trigger_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_view_model_for_data_bind(
        &mut self,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        let Some(value) = self
            .data_bind_graph
            .imported_view_model_target_value_for_data_bind(data_bind_index, instance_index)
        else {
            return false;
        };
        let Some(bindable_view_model) = self
            .bindable_view_models
            .iter_mut()
            .find(|bindable_view_model| bindable_view_model.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_view_model.set_imported_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_view_model_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_view_model_source_instance_index_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        self.data_bind_graph
            .default_view_model_view_model_source_instance_index_for_data_bind(data_bind_index)
    }

    pub fn bindable_view_model_instance_index_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        let global_id = self
            .data_bind_graph
            .view_model_target_global_id_for_data_bind(data_bind_index)?;
        let value = self
            .bindable_view_models
            .iter()
            .find(|bindable_view_model| bindable_view_model.global_id == global_id)
            .map(|bindable_view_model| bindable_view_model.value)?;
        self.data_bind_graph
            .view_model_instance_index_for_data_bind_value(data_bind_index, value)
    }

    pub fn default_view_model_number_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<f32> {
        self.data_bind_graph
            .default_view_model_number_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_number_value_for_data_bind(&self, data_bind_index: usize) -> Option<f32> {
        if let Some(value) = self
            .data_bind_graph
            .number_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| {
                self.bindable_numbers
                    .iter()
                    .find(|bindable_number| bindable_number.global_id == global_id)
                    .map(|bindable_number| bindable_number.value)
            })
        {
            return Some(value);
        }
        self.bindable_numbers
            .iter()
            .find(|bindable_number| bindable_number.has_data_bind_index(data_bind_index))
            .map(|bindable_number| bindable_number.value)
    }

    pub fn default_view_model_boolean_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<bool> {
        self.data_bind_graph
            .default_view_model_boolean_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_boolean_value_for_data_bind(&self, data_bind_index: usize) -> Option<bool> {
        if let Some(value) = self
            .data_bind_graph
            .boolean_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_boolean_value(&self.bindable_booleans, global_id))
        {
            return Some(value);
        }
        self.bindable_booleans
            .iter()
            .find(|bindable_boolean| bindable_boolean.has_data_bind_index(data_bind_index))
            .map(|bindable_boolean| bindable_boolean.value)
    }

    pub fn default_view_model_list_source_item_count_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        self.data_bind_graph
            .default_view_model_list_source_item_count_for_data_bind(data_bind_index)
    }

    pub fn bindable_list_property_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        let global_id = self
            .data_bind_graph
            .list_target_global_id_for_data_bind(data_bind_index)?;
        self.bindable_lists
            .iter()
            .find(|bindable_list| bindable_list.global_id == global_id)
            .map(|bindable_list| bindable_list.property_value)
    }

    pub fn default_view_model_string_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<&[u8]> {
        self.data_bind_graph
            .default_view_model_string_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_string_value_for_data_bind(&self, data_bind_index: usize) -> Option<&[u8]> {
        if let Some(value) = self
            .data_bind_graph
            .string_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_string_value(&self.bindable_strings, global_id))
        {
            return Some(value);
        }
        self.bindable_strings
            .iter()
            .find(|bindable_string| bindable_string.has_data_bind_index(data_bind_index))
            .map(|bindable_string| bindable_string.value.as_slice())
    }

    pub fn default_view_model_color_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u32> {
        self.data_bind_graph
            .default_view_model_color_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_color_value_for_data_bind(&self, data_bind_index: usize) -> Option<u32> {
        if let Some(value) = self
            .data_bind_graph
            .color_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_color_value(&self.bindable_colors, global_id))
        {
            return Some(value);
        }
        self.bindable_colors
            .iter()
            .find(|bindable_color| bindable_color.has_data_bind_index(data_bind_index))
            .map(|bindable_color| bindable_color.value)
    }

    pub fn default_view_model_enum_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        self.data_bind_graph
            .default_view_model_enum_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_enum_value_for_data_bind(&self, data_bind_index: usize) -> Option<u64> {
        if let Some(value) = self
            .data_bind_graph
            .enum_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_enum_value(&self.bindable_enums, global_id))
        {
            return Some(value);
        }
        self.bindable_enums
            .iter()
            .find(|bindable_enum| bindable_enum.has_data_bind_index(data_bind_index))
            .map(|bindable_enum| bindable_enum.value)
    }

    pub fn default_view_model_symbol_list_index_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        self.data_bind_graph
            .default_view_model_symbol_list_index_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_integer_value_for_data_bind(&self, data_bind_index: usize) -> Option<u64> {
        if let Some(value) = self
            .data_bind_graph
            .integer_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_integer_value(&self.bindable_integers, global_id))
        {
            return Some(value);
        }
        self.bindable_integers
            .iter()
            .find(|bindable_integer| bindable_integer.has_data_bind_index(data_bind_index))
            .map(|bindable_integer| bindable_integer.value)
    }

    pub fn default_view_model_asset_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        self.data_bind_graph
            .default_view_model_asset_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_asset_value_for_data_bind(&self, data_bind_index: usize) -> Option<u64> {
        if let Some(value) = self
            .data_bind_graph
            .asset_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_asset_value(&self.bindable_assets, global_id))
        {
            return Some(value);
        }
        self.bindable_assets
            .iter()
            .find(|bindable_asset| bindable_asset.has_data_bind_index(data_bind_index))
            .map(|bindable_asset| bindable_asset.value)
    }

    pub fn default_view_model_artboard_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        self.data_bind_graph
            .default_view_model_artboard_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_artboard_value_for_data_bind(&self, data_bind_index: usize) -> Option<u64> {
        if let Some(value) = self
            .data_bind_graph
            .artboard_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_artboard_value(&self.bindable_artboards, global_id))
        {
            return Some(value);
        }
        self.bindable_artboards
            .iter()
            .find(|bindable_artboard| bindable_artboard.has_data_bind_index(data_bind_index))
            .map(|bindable_artboard| bindable_artboard.value)
    }

    pub fn default_view_model_trigger_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        self.data_bind_graph
            .default_view_model_trigger_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_trigger_value_for_data_bind(&self, data_bind_index: usize) -> Option<u64> {
        if let Some(value) = self
            .data_bind_graph
            .default_view_model_trigger_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_trigger_value(&self.bindable_triggers, global_id))
        {
            return Some(value);
        }
        self.bindable_triggers
            .iter()
            .find(|bindable_trigger| bindable_trigger.has_data_bind_index(data_bind_index))
            .map(|bindable_trigger| bindable_trigger.value)
    }

    pub fn set_default_view_model_number_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: f32,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_number_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_number_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelNumberSourceHandle> {
        let path = runtime_default_view_model_number_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelNumberSourceHandle { path })
    }

    pub fn default_view_model_number_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelNumberSourceHandle> {
        let path =
            runtime_default_view_model_number_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelNumberSourceHandle { path })
    }

    pub fn set_default_view_model_number_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelNumberSourceHandle,
        value: f32,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_number_source_for_path(&handle.path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_number_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: f32,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_number_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_number_source_for_path(&path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_boolean_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: bool,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_boolean_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_boolean_source_for_path(&path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_boolean_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelBooleanSourceHandle> {
        let path = runtime_default_view_model_boolean_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelBooleanSourceHandle { path })
    }

    pub fn default_view_model_boolean_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelBooleanSourceHandle> {
        let path =
            runtime_default_view_model_boolean_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelBooleanSourceHandle { path })
    }

    pub fn set_default_view_model_boolean_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelBooleanSourceHandle,
        value: bool,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_boolean_source_for_path(&handle.path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_boolean_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: bool,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_boolean_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_string_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: &[u8],
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_string_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_string_source_for_path(&path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_string_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelStringSourceHandle> {
        let path = runtime_default_view_model_string_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelStringSourceHandle { path })
    }

    pub fn default_view_model_string_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelStringSourceHandle> {
        let path =
            runtime_default_view_model_string_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelStringSourceHandle { path })
    }

    pub fn set_default_view_model_string_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelStringSourceHandle,
        value: &[u8],
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_string_source_for_path(&handle.path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_string_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: &[u8],
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_string_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_color_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u32,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_color_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_color_source_for_path(&path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_color_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelColorSourceHandle> {
        let path = runtime_default_view_model_color_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelColorSourceHandle { path })
    }

    pub fn default_view_model_color_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelColorSourceHandle> {
        let path =
            runtime_default_view_model_color_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelColorSourceHandle { path })
    }

    pub fn set_default_view_model_color_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelColorSourceHandle,
        value: u32,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_color_source_for_path(&handle.path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_color_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u32,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_color_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_enum_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_enum_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_enum_source_for_path(&path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_enum_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelEnumSourceHandle> {
        let path = runtime_default_view_model_enum_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelEnumSourceHandle { path })
    }

    pub fn default_view_model_enum_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelEnumSourceHandle> {
        let path =
            runtime_default_view_model_enum_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelEnumSourceHandle { path })
    }

    pub fn set_default_view_model_enum_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelEnumSourceHandle,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_enum_source_for_path(&handle.path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_enum_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_enum_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_symbol_list_index_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) = runtime_default_view_model_symbol_list_index_property_path_for_name(
            file,
            property_name,
        ) else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_symbol_list_index_source_for_path(&path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_symbol_list_index_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelSymbolListIndexSourceHandle> {
        let path = runtime_default_view_model_symbol_list_index_property_path_for_name(
            file,
            property_name,
        )?;
        Some(RuntimeDefaultViewModelSymbolListIndexSourceHandle { path })
    }

    pub fn default_view_model_symbol_list_index_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelSymbolListIndexSourceHandle> {
        let path = runtime_default_view_model_symbol_list_index_property_path_for_name_path(
            file,
            property_path,
        )?;
        Some(RuntimeDefaultViewModelSymbolListIndexSourceHandle { path })
    }

    pub fn set_default_view_model_symbol_list_index_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelSymbolListIndexSourceHandle,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_symbol_list_index_source_for_path(&handle.path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_symbol_list_index_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_symbol_list_index_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_asset_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_asset_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_asset_source_for_path(&path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_asset_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelAssetSourceHandle> {
        let path = runtime_default_view_model_asset_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelAssetSourceHandle { path })
    }

    pub fn default_view_model_asset_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelAssetSourceHandle> {
        let path =
            runtime_default_view_model_asset_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelAssetSourceHandle { path })
    }

    pub fn set_default_view_model_asset_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelAssetSourceHandle,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_asset_source_for_path(&handle.path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_asset_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_asset_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_artboard_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_artboard_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_artboard_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_artboard_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_artboard_source_for_path(&path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_artboard_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelArtboardSourceHandle> {
        let path = runtime_default_view_model_artboard_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelArtboardSourceHandle { path })
    }

    pub fn default_view_model_artboard_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelArtboardSourceHandle> {
        let path =
            runtime_default_view_model_artboard_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelArtboardSourceHandle { path })
    }

    pub fn set_default_view_model_artboard_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelArtboardSourceHandle,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_artboard_source_for_path(&handle.path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    fn set_default_view_model_trigger_target_mirror(
        &mut self,
        bindable_global_id: u32,
        value: u64,
    ) {
        let Some(trigger_global_id) = self
            .bindable_triggers
            .iter()
            .find(|trigger| trigger.global_id == bindable_global_id)
            .and_then(|trigger| match trigger.source {
                RuntimeBindableTriggerSource::DefaultViewModelTrigger { trigger_global_id } => {
                    Some(trigger_global_id)
                }
                RuntimeBindableTriggerSource::None => None,
            })
        else {
            return;
        };
        if let Some(trigger) = self
            .default_view_model_triggers
            .iter_mut()
            .find(|trigger| trigger.global_id() == trigger_global_id)
        {
            trigger.set_value(value);
        }
        if self
            .data_bind_graph
            .default_view_model_source_context_bound()
            && let Some(trigger) = self
                .view_model_triggers
                .iter_mut()
                .find(|trigger| trigger.global_id() == trigger_global_id)
        {
            trigger.set_value(value);
        }
    }

    fn set_default_view_model_trigger_source_mirror(
        &mut self,
        view_model_property_id: u32,
        value: u64,
    ) {
        if let Some(trigger) = self
            .default_view_model_triggers
            .iter_mut()
            .find(|trigger| trigger.view_model_property_id() == view_model_property_id)
        {
            trigger.set_value(value);
        }
        if self
            .data_bind_graph
            .default_view_model_source_context_bound()
            && let Some(trigger) = self
                .view_model_triggers
                .iter_mut()
                .find(|trigger| trigger.view_model_property_id() == view_model_property_id)
        {
            trigger.set_value(value);
        }
    }

    fn sync_default_view_model_trigger_mirrors_from_data_bind_graph(&mut self) {
        let updates: Vec<(u32, u64)> = self
            .bindable_triggers
            .iter()
            .filter_map(|bindable_trigger| {
                if !matches!(
                    bindable_trigger.source,
                    RuntimeBindableTriggerSource::DefaultViewModelTrigger { .. }
                ) {
                    return None;
                }
                bindable_trigger
                    .data_bind_indices
                    .iter()
                    .find_map(|data_bind_index| {
                        self.data_bind_graph
                            .default_view_model_trigger_source_value_for_data_bind(*data_bind_index)
                    })
                    .map(|value| (bindable_trigger.global_id, value))
            })
            .collect();
        for (bindable_global_id, value) in updates {
            self.set_default_view_model_trigger_target_mirror(bindable_global_id, value);
        }
    }

    pub fn set_default_view_model_trigger_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let bindable_global_id = self
            .data_bind_graph
            .default_view_model_trigger_target_global_id_for_data_bind(data_bind_index);
        let trigger_property_id = self
            .data_bind_graph
            .default_view_model_trigger_source_property_id_for_data_bind(data_bind_index);
        if !self
            .data_bind_graph
            .set_default_view_model_trigger_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        if let Some(trigger_property_id) = trigger_property_id {
            self.set_default_view_model_trigger_source_mirror(trigger_property_id, value);
        }
        if let Some(bindable_global_id) = bindable_global_id {
            self.set_default_view_model_trigger_target_mirror(bindable_global_id, value);
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_trigger_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_trigger_property_path_for_name(file, property_name)
        else {
            return false;
        };
        let bindable_global_ids = self
            .data_bind_graph
            .default_view_model_trigger_target_global_ids_for_source_path(&path);
        if !self
            .data_bind_graph
            .set_default_view_model_trigger_source_for_path(&path, value)
        {
            return false;
        }
        if let Some(trigger_property_id) = path.last().copied() {
            self.set_default_view_model_trigger_source_mirror(trigger_property_id, value);
        }
        for bindable_global_id in bindable_global_ids {
            self.set_default_view_model_trigger_target_mirror(bindable_global_id, value);
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_trigger_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelTriggerSourceHandle> {
        let path = runtime_default_view_model_trigger_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelTriggerSourceHandle { path })
    }

    pub fn default_view_model_trigger_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelTriggerSourceHandle> {
        let path =
            runtime_default_view_model_trigger_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelTriggerSourceHandle { path })
    }

    pub fn set_default_view_model_trigger_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelTriggerSourceHandle,
        value: u64,
    ) -> bool {
        let bindable_global_ids = self
            .data_bind_graph
            .default_view_model_trigger_target_global_ids_for_source_path(&handle.path);
        if !self
            .data_bind_graph
            .set_default_view_model_trigger_source_for_path(&handle.path, value)
        {
            return false;
        }
        if let Some(trigger_property_id) = handle.path.last().copied() {
            self.set_default_view_model_trigger_source_mirror(trigger_property_id, value);
        }
        for bindable_global_id in bindable_global_ids {
            self.set_default_view_model_trigger_target_mirror(bindable_global_id, value);
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_list_source_item_count_for_data_bind(
        &mut self,
        data_bind_index: usize,
        item_count: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_list_source_item_count_for_data_bind(
                data_bind_index,
                item_count,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_list_source_item_count_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        item_count: usize,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_list_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_list_source_item_count_for_path(&path, item_count)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_list_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelListSourceHandle> {
        let path = runtime_default_view_model_list_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelListSourceHandle { path })
    }

    pub fn default_view_model_list_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelListSourceHandle> {
        let path =
            runtime_default_view_model_list_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelListSourceHandle { path })
    }

    pub fn set_default_view_model_list_source_item_count_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelListSourceHandle,
        item_count: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_list_source_item_count_for_path(&handle.path, item_count)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_view_model_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_view_model_source_for_data_bind(data_bind_index, instance_index)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn relink_default_view_model_view_model_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .relink_default_view_model_view_model_source_for_data_bind(
                data_bind_index,
                instance_index,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn relink_default_view_model_view_model_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        instance_index: usize,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_view_model_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .relink_default_view_model_view_model_source_for_path(&path, instance_index)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_view_model_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelViewModelSourceHandle> {
        let path =
            runtime_default_view_model_view_model_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelViewModelSourceHandle { path })
    }

    pub fn default_view_model_view_model_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelViewModelSourceHandle> {
        let path =
            runtime_default_view_model_view_model_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelViewModelSourceHandle { path })
    }

    pub fn relink_default_view_model_view_model_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelViewModelSourceHandle,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .relink_default_view_model_view_model_source_for_path(&handle.path, instance_index)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn relink_view_model_instance_view_model_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .relink_view_model_instance_view_model_source_for_data_bind(
                data_bind_index,
                instance_index,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn relink_imported_view_model_context_view_model_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .relink_imported_view_model_context_view_model_source_for_data_bind(
                context,
                data_bind_index,
                instance_index,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_number_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: f32,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_number_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_number_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: f32,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_number_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_symbol_list_index_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_symbol_list_index_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_boolean_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: bool,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_boolean_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_enum_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_enum_source_for_data_bind(context, data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_color_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u32,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_color_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_string_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: &[u8],
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_string_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_trigger_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_trigger_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_list_source_item_count_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        item_count: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_list_source_item_count_for_data_bind(
                context,
                data_bind_index,
                item_count,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_asset_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_asset_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_artboard_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_artboard_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_view_model_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_view_model_source_for_data_bind(
                context,
                data_bind_index,
                instance_index,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_boolean_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: bool,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_boolean_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_string_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: &[u8],
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_string_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_color_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u32,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_color_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_enum_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_enum_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_symbol_list_index_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_symbol_list_index_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_asset_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_asset_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_artboard_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_artboard_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_trigger_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let bindable_global_id = self
            .data_bind_graph
            .default_view_model_trigger_target_global_id_for_data_bind(data_bind_index);
        let trigger_property_id = self
            .data_bind_graph
            .default_view_model_trigger_source_property_id_for_data_bind(data_bind_index);
        if !self
            .data_bind_graph
            .set_imported_view_model_context_trigger_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        if let Some(trigger_property_id) = trigger_property_id {
            if let Some(trigger) = self
                .default_view_model_triggers
                .iter_mut()
                .find(|trigger| trigger.view_model_property_id() == trigger_property_id)
            {
                trigger.set_value(value);
            }
            if let Some(trigger) = self
                .view_model_triggers
                .iter_mut()
                .find(|trigger| trigger.view_model_property_id() == trigger_property_id)
            {
                trigger.set_value(value);
            }
        }
        if let Some(trigger_global_id) = bindable_global_id.and_then(|bindable_global_id| {
            self.bindable_triggers
                .iter()
                .find(|trigger| trigger.global_id == bindable_global_id)
                .and_then(|trigger| match trigger.source {
                    RuntimeBindableTriggerSource::DefaultViewModelTrigger { trigger_global_id } => {
                        Some(trigger_global_id)
                    }
                    RuntimeBindableTriggerSource::None => None,
                })
        }) && let Some(trigger) = self
            .view_model_triggers
            .iter_mut()
            .find(|trigger| trigger.global_id() == trigger_global_id)
        {
            trigger.set_value(value);
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_list_source_item_count_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        item_count: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_list_source_item_count_for_data_bind(
                context,
                data_bind_index,
                item_count,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn relink_view_model_instance_view_model_source_by_property_name_path(
        &mut self,
        file: &RuntimeFile,
        property_path: &str,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .relink_view_model_instance_view_model_source_by_property_name_path(
                file,
                property_path,
                instance_index,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn bind_empty_data_context(&mut self) -> bool {
        if !self.data_bind_graph.bind_empty_data_context() {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn bind_default_view_model_context(&mut self) -> bool {
        if !self.data_bind_graph.bind_default_view_model_context() {
            return false;
        }
        self.bind_active_default_view_model_triggers();
        self.needs_advance = true;
        true
    }

    pub fn set_data_bind_formula_random_values(&mut self, values: &[f32]) {
        self.data_bind_graph.set_formula_random_values(values);
    }

    pub fn data_bind_formula_random_call_count(&self) -> usize {
        self.data_bind_graph.formula_random_call_count()
    }

    pub fn bind_view_model_instance_context(
        &mut self,
        file: &RuntimeFile,
        view_model_index: usize,
        instance_index: usize,
    ) -> bool {
        if !self.data_bind_graph.bind_view_model_instance_context(
            file,
            view_model_index,
            instance_index,
        ) {
            return false;
        }
        self.bind_active_imported_view_model_triggers(file, view_model_index, instance_index, None);
        self.needs_advance = true;
        true
    }

    pub fn bind_imported_view_model_context(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeImportedViewModelInstanceContext,
    ) -> bool {
        if !self
            .data_bind_graph
            .bind_imported_view_model_context(file, context)
        {
            return false;
        }
        self.bind_active_imported_view_model_triggers(
            file,
            context.view_model_index,
            context.instance_index,
            Some(context),
        );
        self.needs_advance = true;
        true
    }

    pub fn bind_owned_view_model_context(
        &mut self,
        context: &RuntimeOwnedViewModelInstance,
    ) -> bool {
        if !self.data_bind_graph.bind_owned_view_model_context(context) {
            return false;
        }
        self.bind_active_owned_view_model_triggers(context);
        self.needs_advance = true;
        true
    }

    pub fn advance_data_context(&mut self) -> bool {
        if !self.data_bind_graph.data_context_present() {
            return false;
        }
        if self.data_bind_graph.default_view_model_context_bound() {
            self.data_bind_graph
                .apply_default_view_model_number_targets_to_sources(&self.bindable_numbers);
            self.data_bind_graph
                .apply_default_view_model_symbol_list_index_targets_to_sources(
                    &self.bindable_integers,
                );
            self.data_bind_graph
                .apply_default_view_model_boolean_targets_to_sources(&self.bindable_booleans);
            self.data_bind_graph
                .apply_default_view_model_string_targets_to_sources(&self.bindable_strings);
            self.data_bind_graph
                .apply_default_view_model_color_targets_to_sources(&self.bindable_colors);
            self.data_bind_graph
                .apply_default_view_model_enum_targets_to_sources(&self.bindable_enums);
            self.data_bind_graph
                .apply_default_view_model_asset_targets_to_sources(&self.bindable_assets);
            self.data_bind_graph
                .apply_default_view_model_artboard_targets_to_sources(&self.bindable_artboards);
            self.data_bind_graph
                .apply_default_view_model_list_targets_to_sources();
            self.data_bind_graph
                .apply_default_view_model_trigger_targets_to_sources(&self.bindable_triggers);
            self.data_bind_graph
                .apply_default_view_model_view_model_targets_to_sources(&self.bindable_view_models);
            self.apply_default_view_model_bindings(true, RuntimeDataBindGraphApplyPhase::Immediate);
            for trigger in &mut self.view_model_triggers {
                trigger.reset();
            }
            self.data_bind_graph.reset_bound_trigger_sources();
            if self
                .data_bind_graph
                .default_view_model_source_context_bound()
            {
                self.sync_default_view_model_triggers_from_active();
            }
        }
        true
    }

    pub fn update_data_binds_apply_target_to_source(&mut self) -> bool {
        if !self.data_bind_graph.data_context_present() {
            return false;
        }
        if self.data_bind_graph.default_view_model_context_bound() {
            self.data_bind_graph
                .apply_default_view_model_number_public_update_targets_to_sources(
                    &self.bindable_numbers,
                );
            self.data_bind_graph
                .apply_default_view_model_symbol_list_index_public_update_targets_to_sources(
                    &self.bindable_integers,
                );
            self.data_bind_graph
                .apply_default_view_model_boolean_public_update_targets_to_sources(
                    &self.bindable_booleans,
                );
            self.data_bind_graph
                .apply_default_view_model_string_public_update_targets_to_sources(
                    &self.bindable_strings,
                );
            self.data_bind_graph
                .apply_default_view_model_color_public_update_targets_to_sources(
                    &self.bindable_colors,
                );
            self.data_bind_graph
                .apply_default_view_model_enum_public_update_targets_to_sources(
                    &self.bindable_enums,
                );
            self.data_bind_graph
                .apply_default_view_model_asset_public_update_targets_to_sources(
                    &self.bindable_assets,
                );
            self.data_bind_graph
                .apply_default_view_model_artboard_public_update_targets_to_sources(
                    &self.bindable_artboards,
                );
            self.data_bind_graph
                .apply_default_view_model_list_public_update_targets_to_sources();
            self.data_bind_graph
                .apply_default_view_model_trigger_public_update_targets_to_sources(
                    &self.bindable_triggers,
                );
            self.data_bind_graph
                .apply_default_view_model_view_model_public_update_targets_to_sources(
                    &self.bindable_view_models,
                );
            self.apply_default_view_model_bindings(
                true,
                RuntimeDataBindGraphApplyPhase::PublicUpdate,
            );
            self.sync_default_view_model_trigger_mirrors_from_data_bind_graph();
        }
        true
    }

    pub fn current_animation_count(&self) -> usize {
        self.layers
            .iter()
            .filter(|layer| layer.has_current_animation())
            .count()
    }

    pub fn current_animation(&self, index: usize) -> Option<&LinearAnimationInstance> {
        self.layers
            .iter()
            .filter_map(StateMachineLayerInstance::current_animation)
            .nth(index)
    }

    pub fn reported_event_count(&self) -> usize {
        self.reported_events.len()
    }

    pub fn reported_event(&self, index: usize) -> Option<&StateMachineReportedEvent> {
        self.reported_events.get(index)
    }

    pub fn view_model_trigger_count(&self, index: usize) -> Option<u64> {
        self.default_view_model_triggers
            .get(index)
            .map(StateMachineViewModelTriggerInstance::value)
    }

    pub fn view_model_trigger_value_count(&self) -> usize {
        self.default_view_model_triggers.len()
    }

    pub fn view_model_trigger_property_id(&self, index: usize) -> Option<u32> {
        self.default_view_model_triggers
            .get(index)
            .map(StateMachineViewModelTriggerInstance::view_model_property_id)
    }

    pub(crate) fn advance(
        &mut self,
        artboard: &mut ArtboardInstance,
        state_machine: &RuntimeStateMachine,
        elapsed_seconds: f32,
    ) -> bool {
        self.reported_events.clear();
        self.changed_state_count = 0;
        self.needs_advance = false;
        self.apply_default_view_model_bindings(
            true,
            RuntimeDataBindGraphApplyPhase::BeforeStatefulAdvance,
        );
        let data_bind_advance = self
            .data_bind_graph
            .advance_stateful_converters(elapsed_seconds);
        self.apply_default_view_model_bindings(
            true,
            RuntimeDataBindGraphApplyPhase::AfterStatefulAdvance {
                elapsed_positive: elapsed_seconds > 0.0,
            },
        );
        let data_context_present = self.data_bind_graph.data_context_present();
        let data_context_view_model_bound = self.data_bind_graph.default_view_model_context_bound();
        let mut keep_going = false;
        for (layer_index, (layer_instance, layer)) in self
            .layers
            .iter_mut()
            .zip(&state_machine.layers)
            .enumerate()
        {
            let layer_result = layer_instance.advance(
                artboard,
                layer,
                elapsed_seconds,
                layer_index,
                &mut self.inputs,
                &self.bindable_numbers,
                &self.bindable_integers,
                &self.bindable_colors,
                &self.bindable_strings,
                &self.bindable_enums,
                &self.bindable_assets,
                &self.bindable_artboards,
                &self.bindable_triggers,
                &self.bindable_view_models,
                &self.bindable_booleans,
                data_context_present,
                data_context_view_model_bound,
                &mut self.view_model_triggers,
                &mut self.reported_events,
            );
            if layer_result.changed_state {
                self.changed_state_count += 1;
            }
            keep_going |= layer_result.keep_going;
        }
        for input in &mut self.inputs {
            input.advanced();
        }
        if self
            .data_bind_graph
            .default_view_model_source_context_bound()
        {
            self.sync_default_view_model_triggers_from_active();
        }
        self.needs_advance =
            data_bind_advance.keep_going || keep_going || !self.reported_events.is_empty();
        self.needs_advance
    }

    fn apply_default_view_model_bindings(
        &mut self,
        include_view_models: bool,
        phase: RuntimeDataBindGraphApplyPhase,
    ) {
        self.data_bind_graph.apply_default_view_model_bindings(
            RuntimeDataBindGraphTargetsMut {
                numbers: &mut self.bindable_numbers,
                integers: &mut self.bindable_integers,
                booleans: &mut self.bindable_booleans,
                strings: &mut self.bindable_strings,
                colors: &mut self.bindable_colors,
                enums: &mut self.bindable_enums,
                assets: &mut self.bindable_assets,
                artboards: &mut self.bindable_artboards,
                lists: &mut self.bindable_lists,
                triggers: &mut self.bindable_triggers,
                view_models: &mut self.bindable_view_models,
                include_view_models,
            },
            phase,
        );
    }

    fn bind_active_default_view_model_triggers(&mut self) {
        self.view_model_triggers = self.default_view_model_triggers.clone();
    }

    fn bind_active_imported_view_model_triggers(
        &mut self,
        file: &RuntimeFile,
        view_model_index: usize,
        instance_index: usize,
        context: Option<&RuntimeImportedViewModelInstanceContext>,
    ) {
        let instance = file
            .view_model(view_model_index)
            .and_then(|view_model| view_model.instances.into_iter().nth(instance_index));
        let Ok(view_model_path_id) = u32::try_from(view_model_index) else {
            self.reset_active_view_model_triggers();
            return;
        };
        let Some(instance) = instance else {
            self.reset_active_view_model_triggers();
            return;
        };

        let mut active = self.default_view_model_triggers.clone();
        for trigger in &mut active {
            let path = [view_model_path_id, trigger.view_model_property_id()];
            let value = context
                .and_then(|context| context.trigger_overrides.get(path.as_slice()).copied())
                .or_else(|| {
                    file.data_context_view_model_property_for_instance(instance.object, &path)
                        .and_then(|source| {
                            file.view_model_instance_trigger_count_for_object(source)
                        })
                });
            if let Some(value) = value {
                trigger.replace_value(value);
            } else {
                trigger.reset();
            }
        }
        self.view_model_triggers = active;
    }

    fn bind_active_owned_view_model_triggers(&mut self, context: &RuntimeOwnedViewModelInstance) {
        let mut active = self.default_view_model_triggers.clone();
        for trigger in &mut active {
            let value = usize::try_from(trigger.view_model_property_id())
                .ok()
                .and_then(|property_index| context.trigger_value_by_property_index(property_index));
            if let Some(value) = value {
                trigger.replace_value(value);
            } else {
                trigger.reset();
            }
        }
        self.view_model_triggers = active;
    }

    fn reset_active_view_model_triggers(&mut self) {
        self.view_model_triggers = self.default_view_model_triggers.clone();
        for trigger in &mut self.view_model_triggers {
            trigger.reset();
        }
    }

    fn sync_default_view_model_triggers_from_active(&mut self) {
        for default_trigger in &mut self.default_view_model_triggers {
            if let Some(active_trigger) = self
                .view_model_triggers
                .iter()
                .find(|trigger| trigger.global_id() == default_trigger.global_id())
            {
                *default_trigger = active_trigger.clone();
            }
        }
    }
}
