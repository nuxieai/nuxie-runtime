use crate::mechanical_port::source::generated::viewmodel::viewmodel_property_symbol_base::ViewModelPropertySymbolBase;

#[derive(Default)]
pub struct ViewModelPropertySymbol {
    pub base: ViewModelPropertySymbolBase,
}

impl std::ops::Deref for ViewModelPropertySymbol {
    type Target = ViewModelPropertySymbolBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ViewModelPropertySymbol {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
