use crate::mechanical_port::source::generated::animation::exit_state_base::ExitStateBase;

#[derive(Default)]
pub struct ExitState {
    pub base: ExitStateBase,
}

impl std::ops::Deref for ExitState {
    type Target = ExitStateBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ExitState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
