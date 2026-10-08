use crate::mechanical_port::source::{
    animation::listener_invocation::ListenerInvocation,
    core::{CoreHandle, CoreObject},
    core_context::CoreContext,
    generated::animation::{
        state_machine_base::StateMachineBase, state_machine_listener_base::StateMachineListenerBase,
    },
    importers::{import_stack::ImportStack, state_machine_importer::StateMachineImporter},
    listener_type::ListenerType,
    pointer_button::PointerButton,
    status_code::StatusCode,
};
const POINTER_HIT_LISTENER_TYPES: [ListenerType; 9] = [
    ListenerType::Enter,
    ListenerType::Exit,
    ListenerType::Down,
    ListenerType::Up,
    ListenerType::Move,
    ListenerType::Click,
    ListenerType::DragStart,
    ListenerType::DragEnd,
    ListenerType::Drag,
];

// Preserve virtual hasListener dispatch for both current and legacy single
// listeners when the complete occurrence is available at the caller.
pub(crate) fn has_pointer_listeners(listener: &dyn CoreObject) -> bool {
    POINTER_HIT_LISTENER_TYPES
        .iter()
        .copied()
        .any(|kind| listener.state_machine_listener_has(kind).unwrap_or(false))
}
#[derive(Default)]
pub struct StateMachineListener {
    pub base: StateMachineListenerBase,
    actions: Vec<CoreHandle>,
    listener_input_types: Vec<CoreHandle>,
}
impl StateMachineListener {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn on_added_dirty(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        self.on_added_clean_with_pointer(context, self.has_pointer_listeners())
    }
    pub(crate) fn on_added_clean_with_pointer(
        &mut self,
        context: &mut dyn CoreContext,
        has_pointer: bool,
    ) -> StatusCode {
        if has_pointer {
            if let Some(target) = context.resolve(self.base.target_id()) {
                target.with_mut(|target| {
                    if let Some(layout) = target.as_layout_component_mut() {
                        layout.mark_listener_target();
                    }
                });
            }
        }
        StatusCode::Ok
    }
    pub fn has_pointer_listeners(&self) -> bool {
        self.has_listeners(&POINTER_HIT_LISTENER_TYPES)
    }
    pub fn has_listener(&self, kind: ListenerType) -> bool {
        self.listener_input_types.iter().any(|value| {
            value
                .with(|value| value.listener_input_type_value() == Some(kind as u32))
                .unwrap_or(false)
        })
    }
    pub fn has_listeners(&self, kinds: &[ListenerType]) -> bool {
        kinds.iter().copied().any(|kind| self.has_listener(kind))
    }
    pub fn has_listener_button(&self, kind: ListenerType, button: PointerButton) -> bool {
        if !listener_type_has_button(kind) {
            return self.has_listener(kind);
        }
        self.listener_input_types.iter().any(|value| {
            value
                .with(|value| {
                    value.listener_input_type_value() == Some(kind as u32)
                        && value.listener_input_type_pointer_button() == Some(button)
                })
                .unwrap_or(false)
        })
    }
    pub fn listens_to_button(&self, button: PointerButton) -> bool {
        self.listener_input_types.iter().any(|value| {
            value
                .with(|value| {
                    value
                        .listener_input_type_value()
                        .map(|kind| {
                            [
                                ListenerType::Down,
                                ListenerType::Up,
                                ListenerType::Click,
                                ListenerType::Drag,
                                ListenerType::DragStart,
                                ListenerType::DragEnd,
                            ]
                            .iter()
                            .any(|candidate| *candidate as u32 == kind)
                        })
                        .unwrap_or(false)
                        && value.listener_input_type_pointer_button() == Some(button)
                })
                .unwrap_or(false)
        })
    }
    #[cfg(any(test, feature = "testing"))]
    pub fn replace_listener_input_type_for_testing(&mut self, index: usize, value: CoreHandle) {
        self.listener_input_types[index] = value;
    }
    #[cfg(any(test, feature = "testing"))]
    pub fn add_listener_input_type_for_testing(&mut self, value: CoreHandle) {
        self.listener_input_types.push(value);
    }
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }
    pub fn listener_input_type_count(&self) -> usize {
        self.listener_input_types.len()
    }
    pub fn action(&self, index: usize) -> Option<CoreHandle> {
        self.actions.get(index).cloned()
    }
    pub fn listener_input_type(&self, index: usize) -> Option<CoreHandle> {
        self.listener_input_types.get(index).cloned()
    }
    pub(crate) fn add_action(&mut self, value: CoreHandle) {
        self.actions.push(value);
    }
    pub(crate) fn add_listener_input_type(&mut self, value: CoreHandle) {
        self.listener_input_types.push(value);
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = stack.latest::<StateMachineImporter>(StateMachineBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_listener(this);
        self.base.base.import(stack)
    }
    pub fn perform_changes(
        &self,
        machine: &mut crate::mechanical_port::source::animation::state_machine_instance::StateMachineInstance,
        invocation: &ListenerInvocation,
        mut dispatch: impl FnMut(
            &CoreHandle,
            &mut crate::mechanical_port::source::animation::state_machine_instance::StateMachineInstance,
            &ListenerInvocation,
        ),
    ) {
        machine.wake_row();
        #[cfg(feature = "tools")]
        let _write_source = super::super::viewmodel::write_attribution::WriteAttributionScope::new(
            super::super::viewmodel::write_attribution::WriteSourceKind::listener,
            self.base
                .base
                .base
                .base
                .handle()
                .as_ref()
                .map_or(self as *const Self as usize, CoreHandle::slot_address),
            Some(machine as *const super::state_machine_instance::StateMachineInstance as usize),
        );
        for action in &self.actions {
            dispatch(action, machine, invocation);
        }
    }
}
fn listener_type_has_button(kind: ListenerType) -> bool {
    matches!(
        kind,
        ListenerType::Down
            | ListenerType::Up
            | ListenerType::Click
            | ListenerType::Drag
            | ListenerType::DragStart
            | ListenerType::DragEnd
    )
}
impl std::ops::Deref for StateMachineListener {
    type Target = StateMachineListenerBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for StateMachineListener {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::animation::listener_types::listener_input_type_keyboard::KeyboardConstraintListener
    for StateMachineListener
{
    fn keyboard_input_types(&self) -> Vec<CoreHandle> {
        self.listener_input_types.clone()
    }
}
impl crate::mechanical_port::source::animation::listener_types::listener_input_type_gamepad::GamepadConstraintListener
    for StateMachineListener
{
    fn gamepad_input_types(&self) -> Vec<CoreHandle> {
        self.listener_input_types.clone()
    }
}
impl crate::mechanical_port::source::animation::listener_types::listener_input_type_semantic::SemanticConstraintListener
    for StateMachineListener
{
    fn semantic_input_types(&self) -> Vec<CoreHandle> {
        self.listener_input_types.clone()
    }
}
impl crate::mechanical_port::source::generated::animation::state_machine_component_base::StateMachineComponentBaseCallbacks for StateMachineListener { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
impl crate::mechanical_port::source::generated::animation::state_machine_listener_base::StateMachineListenerBaseCallbacks for StateMachineListener { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
