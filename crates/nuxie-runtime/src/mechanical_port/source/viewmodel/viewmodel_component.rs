use crate::mechanical_port::source::generated::viewmodel::viewmodel_component_base::ViewModelComponentBase;

#[derive(Default)]
pub struct ViewModelComponent {
    pub base: ViewModelComponentBase,
}

impl std::ops::Deref for ViewModelComponent {
    type Target = ViewModelComponentBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ViewModelComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
