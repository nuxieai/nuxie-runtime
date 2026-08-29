use crate::mechanical_port::source::generated::backboard_base::BackboardBase;

#[derive(Default)]
pub struct Backboard {
    pub base: BackboardBase,
}

impl std::ops::Deref for Backboard {
    type Target = BackboardBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Backboard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
