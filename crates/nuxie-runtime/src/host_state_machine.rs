//! Host projections over the translated state-machine occurrence.

use crate::mechanical_port::source::{
    animation::{
        state_machine::StateMachine,
        state_machine_instance::{EventReport, RuntimeStateMachineInstanceHandle},
    },
    artboard::RuntimeArtboardInstanceHandle,
    core::CoreHandle,
    file::RuntimeFileHandle,
    generated::{component_base::ComponentBase, core_registry::CoreRegistry},
    math::vec2d::Vec2D,
    open_url_event::OpenUrlEvent,
};

pub use crate::mechanical_port::source::{
    animation::state_machine_instance::FocusState, hit_result::HitResult as RuntimeHitResult,
};

impl RuntimeHitResult {
    pub fn is_hit(self) -> bool {
        self != Self::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeEventPropertyValue {
    Number(f32),
    Bool(bool),
    String(Vec<u8>),
    Color(u32),
    Enum(u64),
    Trigger(u64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeEventProperty {
    pub name: Option<String>,
    pub value: RuntimeEventPropertyValue,
}

#[derive(Clone, Debug)]
pub struct StateMachineEventStringProperty {
    name: String,
    value: String,
}
impl StateMachineEventStringProperty {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateMachineInputKind {
    Bool,
    Number,
    Trigger,
}

/// An owned observation. Mutations go through the machine's real input owner.
#[derive(Clone, Debug)]
pub struct StateMachineInputInstance {
    index: usize,
    name: String,
    kind: StateMachineInputKind,
    bool_value: Option<bool>,
    number_value: Option<f32>,
    fired: bool,
}

impl StateMachineInputInstance {
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn name(&self) -> Option<&str> {
        Some(&self.name)
    }
    pub fn kind(&self) -> StateMachineInputKind {
        self.kind
    }
    pub fn input_core_type(&self) -> u16 {
        match self.kind {
            StateMachineInputKind::Bool => 59,
            StateMachineInputKind::Number => 56,
            StateMachineInputKind::Trigger => 58,
        }
    }
    pub fn bool_value(&self) -> Option<bool> {
        self.bool_value
    }
    pub fn number_value(&self) -> Option<f32> {
        self.number_value
    }
    pub fn trigger_fired(&self) -> bool {
        self.fired
    }
}

pub type RuntimeStateMachineInput = StateMachineInputInstance;

#[derive(Clone, Debug)]
pub struct RuntimeLayerState {
    pub handle: CoreHandle,
    pub core_type: u16,
}

impl RuntimeLayerState {
    pub fn native_handle(&self) -> CoreHandle {
        self.handle.clone()
    }
}

#[derive(Clone, Debug)]
pub struct StateMachineReportedEvent {
    event: CoreHandle,
    local_index: Option<usize>,
    core_type: u16,
    name: Option<String>,
    url: Option<String>,
    target: Option<String>,
    seconds_delay: f32,
    properties: Vec<RuntimeEventProperty>,
    string_properties: Vec<StateMachineEventStringProperty>,
}

impl StateMachineReportedEvent {
    fn from_native(report: EventReport, artboard: &RuntimeArtboardInstanceHandle) -> Option<Self> {
        let event = report.event?;
        let core_type = event.core_type()?;
        let local_index =
            artboard.with_artboard(|a| usize::try_from(a.base.object_index(&event)).ok());
        let name =
            CoreRegistry::get_string_handle(&event, i32::from(ComponentBase::NAME_PROPERTY_KEY))
                .filter(|name| !name.is_empty());
        let (url, target) = event
            .with_downcast::<OpenUrlEvent, _>(|event| {
                let target = match event.base.target_value() {
                    1 => "_self",
                    2 => "_parent",
                    3 => "_top",
                    _ => "_blank",
                };
                (Some(event.base.url().to_owned()), Some(target.to_owned()))
            })
            .unwrap_or_default();
        let properties = event_properties(&event);
        let string_properties = properties
            .iter()
            .filter_map(|property| {
                let RuntimeEventPropertyValue::String(value) = &property.value else {
                    return None;
                };
                Some(StateMachineEventStringProperty {
                    name: property.name.clone().unwrap_or_default(),
                    value: String::from_utf8_lossy(value).into_owned(),
                })
            })
            .collect();
        Some(Self {
            event,
            local_index,
            core_type,
            name,
            url,
            target,
            seconds_delay: report.seconds_delay,
            properties,
            string_properties,
        })
    }
    pub fn native_handle(&self) -> CoreHandle {
        self.event.clone()
    }
    pub fn event_local_index(&self) -> Option<usize> {
        self.local_index
    }
    pub fn event_core_type(&self) -> u32 {
        u32::from(self.core_type)
    }
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }
    pub fn seconds_delay(&self) -> f32 {
        self.seconds_delay
    }
    pub fn properties(&self) -> &[RuntimeEventProperty] {
        &self.properties
    }
    pub fn string_properties(&self) -> &[StateMachineEventStringProperty] {
        &self.string_properties
    }
}

fn event_properties(event: &CoreHandle) -> Vec<RuntimeEventProperty> {
    use crate::mechanical_port::source::generated::{
        custom_property_boolean_base::CustomPropertyBooleanBase as Boolean,
        custom_property_color_base::CustomPropertyColorBase as Color,
        custom_property_enum_base::CustomPropertyEnumBase as Enum,
        custom_property_number_base::CustomPropertyNumberBase as Number,
        custom_property_string_base::CustomPropertyStringBase as StringProperty,
        custom_property_trigger_base::CustomPropertyTriggerBase as Trigger,
    };
    let children = event
        .with(|event| {
            event
                .as_container_component()
                .map(|container| container.children().to_vec())
        })
        .flatten()
        .unwrap_or_default();
    children
        .into_iter()
        .filter_map(|property| {
            let value = if property.is_type_of(Number::TYPE_KEY) {
                RuntimeEventPropertyValue::Number(CoreRegistry::get_double_handle(
                    &property,
                    i32::from(Number::PROPERTY_VALUE_PROPERTY_KEY),
                )?)
            } else if property.is_type_of(Boolean::TYPE_KEY) {
                RuntimeEventPropertyValue::Bool(CoreRegistry::get_bool_handle(
                    &property,
                    i32::from(Boolean::PROPERTY_VALUE_PROPERTY_KEY),
                )?)
            } else if property.is_type_of(StringProperty::TYPE_KEY) {
                RuntimeEventPropertyValue::String(
                    CoreRegistry::get_string_handle(
                        &property,
                        i32::from(StringProperty::PROPERTY_VALUE_PROPERTY_KEY),
                    )?
                    .into_bytes(),
                )
            } else if property.is_type_of(Color::TYPE_KEY) {
                RuntimeEventPropertyValue::Color(CoreRegistry::get_color_handle(
                    &property,
                    i32::from(Color::PROPERTY_VALUE_PROPERTY_KEY),
                )? as u32)
            } else if property.is_type_of(Enum::TYPE_KEY) {
                RuntimeEventPropertyValue::Enum(u64::from(CoreRegistry::get_uint_handle(
                    &property,
                    i32::from(Enum::PROPERTY_VALUE_PROPERTY_KEY),
                )?))
            } else if property.is_type_of(Trigger::TYPE_KEY) {
                RuntimeEventPropertyValue::Trigger(u64::from(CoreRegistry::get_uint_handle(
                    &property,
                    i32::from(Trigger::PROPERTY_VALUE_PROPERTY_KEY),
                )?))
            } else {
                return None;
            };
            let name = CoreRegistry::get_string_handle(
                &property,
                i32::from(ComponentBase::NAME_PROPERTY_KEY),
            )
            .filter(|name| !name.is_empty());
            Some(RuntimeEventProperty { name, value })
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RuntimeStateMachineAdvanceResult {
    pub keep_going: bool,
}

pub struct StateMachineInstance {
    native: RuntimeStateMachineInstanceHandle,
    artboard: RuntimeArtboardInstanceHandle,
    file: RuntimeFileHandle,
    index: usize,
}

impl std::fmt::Debug for StateMachineInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateMachineInstance")
            .field("index", &self.index)
            .field("name", &self.name())
            .finish()
    }
}

impl StateMachineInstance {
    pub(crate) fn from_native(
        file: RuntimeFileHandle,
        artboard: RuntimeArtboardInstanceHandle,
        index: usize,
        native: RuntimeStateMachineInstanceHandle,
    ) -> Self {
        Self {
            native,
            artboard,
            file,
            index,
        }
    }

    pub fn native_handle(&self) -> RuntimeStateMachineInstanceHandle {
        self.native.clone()
    }
    pub fn native_artboard(&self) -> RuntimeArtboardInstanceHandle {
        self.artboard.clone()
    }
    pub fn native_file(&self) -> RuntimeFileHandle {
        self.file.clone()
    }
    pub fn state_machine_index(&self) -> usize {
        self.index
    }
    pub fn name(&self) -> String {
        self.native.with_instance(|machine| machine.name())
    }
    pub fn input_count(&self) -> usize {
        self.native.with_instance(|machine| machine.input_count())
    }
    pub fn input(&self, index: usize) -> Option<StateMachineInputInstance> {
        let id = u32::try_from(index).ok()?;
        self.native.with_instance(|machine| {
            let input = machine.input(index)?;
            let bool_value = machine.bool_input(id).map(|input| input.value());
            let number_value = machine.number_input(id).map(|input| input.value());
            let trigger = machine.trigger_input(id);
            let kind = if bool_value.is_some() {
                StateMachineInputKind::Bool
            } else if number_value.is_some() {
                StateMachineInputKind::Number
            } else if trigger.is_some() {
                StateMachineInputKind::Trigger
            } else {
                return None;
            };
            Some(StateMachineInputInstance {
                index,
                name: input.name().to_owned(),
                kind,
                bool_value,
                number_value,
                fired: trigger.is_some_and(|trigger| trigger.fired()),
            })
        })
    }
    pub fn input_index_named(&self, name: &str) -> Option<usize> {
        (0..self.input_count()).find(|&index| {
            self.input(index)
                .is_some_and(|input| input.name() == Some(name))
        })
    }
    pub fn input_named(&self, name: &str) -> Option<StateMachineInputInstance> {
        self.input(self.input_index_named(name)?)
    }
    pub fn get_bool(&self, name: &str) -> Option<StateMachineInputInstance> {
        self.input_named(name)
            .filter(|input| input.kind == StateMachineInputKind::Bool)
    }
    pub fn get_number(&self, name: &str) -> Option<StateMachineInputInstance> {
        self.input_named(name)
            .filter(|input| input.kind == StateMachineInputKind::Number)
    }
    pub fn get_trigger(&self, name: &str) -> Option<StateMachineInputInstance> {
        self.input_named(name)
            .filter(|input| input.kind == StateMachineInputKind::Trigger)
    }
    pub fn set_bool(&mut self, index: usize, value: bool) -> bool {
        let Ok(index) = u32::try_from(index) else {
            return false;
        };
        self.native.with_instance_mut(|machine| {
            let Some(input) = machine.bool_input_mut(index) else {
                return false;
            };
            let changed = input.value() != value;
            input.set_value(value);
            changed
        })
    }
    pub fn set_number(&mut self, index: usize, value: f32) -> bool {
        let Ok(index) = u32::try_from(index) else {
            return false;
        };
        self.native.with_instance_mut(|machine| {
            let Some(input) = machine.number_input_mut(index) else {
                return false;
            };
            let changed = input.value() != value;
            input.set_value(value);
            changed
        })
    }
    pub fn fire_trigger(&mut self, index: usize) -> bool {
        let Ok(index) = u32::try_from(index) else {
            return false;
        };
        self.native.with_instance_mut(|machine| {
            let Some(input) = machine.trigger_input_mut(index) else {
                return false;
            };
            input.fire();
            true
        })
    }
    pub fn advance(&mut self, seconds: f32, new_frame: bool) -> bool {
        self.native
            .with_instance_mut(|machine| machine.advance(seconds, new_frame))
    }
    pub fn advance_and_apply(&mut self, seconds: f32) -> bool {
        self.native
            .with_instance_mut(|machine| machine.advance_and_apply(seconds))
    }
    pub fn advance_and_apply_view_models(
        &mut self,
        seconds: f32,
        advance_view_models: bool,
    ) -> bool {
        self.native.with_instance_mut(|machine| {
            machine.advance_and_apply_view_models(seconds, advance_view_models)
        })
    }
    pub fn needs_advance(&self) -> bool {
        self.native.with_instance(|machine| machine.needs_advance())
    }
    pub fn has_listeners(&self) -> bool {
        self.native.with_instance(|machine| machine.has_listeners())
    }
    pub fn changed_state_count(&self) -> usize {
        self.native
            .with_instance(|machine| machine.state_changed_count())
    }
    pub fn changed_state(&self, index: usize) -> Option<RuntimeLayerState> {
        let handle = self
            .native
            .with_instance_mut(|machine| machine.state_changed_by_index(index))?;
        Some(RuntimeLayerState {
            core_type: handle.core_type()?,
            handle,
        })
    }
    pub fn layer_count(&self) -> usize {
        let machine = self.native.with_instance(|machine| machine.state_machine());
        machine
            .with_downcast::<StateMachine, _>(|machine| machine.layer_count())
            .unwrap_or(0)
    }
    pub fn layer_state(&self, index: usize) -> Option<RuntimeLayerState> {
        let handle = self
            .native
            .with_instance_mut(|machine| machine.layer_state(index))?;
        Some(RuntimeLayerState {
            core_type: handle.core_type()?,
            handle,
        })
    }
    pub fn reported_event_count(&self) -> usize {
        self.native
            .with_instance(|machine| machine.reported_event_count())
    }
    pub fn reported_event(&self, index: usize) -> Option<StateMachineReportedEvent> {
        let report = self.native.with_instance(|machine| {
            (index < machine.reported_event_count()).then(|| machine.reported_event_at(index))
        })?;
        StateMachineReportedEvent::from_native(report, &self.artboard)
    }
    pub fn reported_events(&self) -> Vec<StateMachineReportedEvent> {
        (0..self.reported_event_count())
            .filter_map(|index| self.reported_event(index))
            .collect()
    }
    pub fn pointer_move(
        &mut self,
        _artboard: &mut crate::host_artboard::ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> RuntimeHitResult {
        self.pointer_move_at(x, y, 0.0, pointer_id)
    }
    pub fn pointer_move_at(
        &mut self,
        x: f32,
        y: f32,
        timestamp: f32,
        pointer_id: i32,
    ) -> RuntimeHitResult {
        self.native.with_instance_mut(|machine| {
            machine.pointer_move(Vec2D::new(x, y), timestamp, pointer_id)
        })
    }
    pub fn pointer_down(
        &mut self,
        _artboard: &mut crate::host_artboard::ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> RuntimeHitResult {
        self.native
            .with_instance_mut(|machine| machine.pointer_down(Vec2D::new(x, y), pointer_id))
    }
    pub fn pointer_up(
        &mut self,
        _artboard: &mut crate::host_artboard::ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> RuntimeHitResult {
        self.native
            .with_instance_mut(|machine| machine.pointer_up(Vec2D::new(x, y), pointer_id))
    }
    pub fn pointer_exit(
        &mut self,
        _artboard: &mut crate::host_artboard::ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> RuntimeHitResult {
        self.native
            .with_instance_mut(|machine| machine.pointer_exit(Vec2D::new(x, y), pointer_id))
    }
    pub fn bind_native_view_model(&mut self, instance: Option<CoreHandle>) {
        self.native
            .with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    }
    pub fn bind_owned_view_model_handle(&mut self, instance: crate::RuntimeOwnedViewModelHandle) {
        self.bind_native_view_model(Some(instance.native_handle()));
    }
    pub fn has_focus_nodes(&self) -> bool {
        self.native
            .with_instance(|machine| machine.has_focus_nodes())
    }
    pub fn focus_state(&self) -> FocusState {
        self.native.with_instance(|machine| machine.focus_state())
    }
    pub fn focus_next(&mut self) -> bool {
        self.native
            .with_instance_mut(|machine| machine.focus_next())
    }
    pub fn focus_previous(&mut self) -> bool {
        self.native
            .with_instance_mut(|machine| machine.focus_previous())
    }
    pub fn focus_up(&mut self) -> bool {
        self.native.with_instance_mut(|machine| machine.focus_up())
    }
    pub fn focus_down(&mut self) -> bool {
        self.native
            .with_instance_mut(|machine| machine.focus_down())
    }
    pub fn focus_left(&mut self) -> bool {
        self.native
            .with_instance_mut(|machine| machine.focus_left())
    }
    pub fn focus_right(&mut self) -> bool {
        self.native
            .with_instance_mut(|machine| machine.focus_right())
    }
    pub fn clear_focus(&mut self) {
        self.native
            .with_instance_mut(|machine| machine.clear_focus());
    }
    pub fn reset_state(&mut self) {
        self.native
            .with_instance_mut(|machine| machine.reset_state());
    }
}
