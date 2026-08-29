use crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_symbol_base::ViewModelInstanceSymbolBase;

#[derive(Default)]
pub struct ViewModelInstanceSymbol {
    pub base: ViewModelInstanceSymbolBase,
}

impl std::ops::Deref for ViewModelInstanceSymbol {
    type Target = ViewModelInstanceSymbolBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ViewModelInstanceSymbol {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
