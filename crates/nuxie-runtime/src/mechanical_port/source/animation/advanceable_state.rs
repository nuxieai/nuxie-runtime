use crate::mechanical_port::source::generated::animation::advanceable_state_base::AdvanceableStateBase;

#[derive(Default)]
pub struct AdvanceableState {
    pub base: AdvanceableStateBase,
}
impl std::ops::Deref for AdvanceableState {
    type Target = AdvanceableStateBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for AdvanceableState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::layer_state_base::LayerStateBaseCallbacks
    for AdvanceableState
{
    fn notify_property_changed(&mut self, key: u16) {
        self.base.notify_property_changed(key);
    }
}
impl crate::mechanical_port::source::generated::animation::advanceable_state_base::AdvanceableStateBaseCallbacks for AdvanceableState { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
