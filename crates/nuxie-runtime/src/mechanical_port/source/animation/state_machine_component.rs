use crate::mechanical_port::source::generated::animation::state_machine_component_base::StateMachineComponentBase;

#[derive(Default)]
pub struct StateMachineComponent {
    pub base: StateMachineComponentBase,
}
impl std::ops::Deref for StateMachineComponent {
    type Target = StateMachineComponentBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for StateMachineComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::state_machine_component_base::StateMachineComponentBaseCallbacks for StateMachineComponent { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
