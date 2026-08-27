use super::viewmodel_instance_value_runtime::{
    DataType, ViewModelInstanceValue, ViewModelInstanceValueRuntime,
};
use std::rc::Rc;
pub trait BindableArtboard {
    fn artboard_name(&self) -> Option<&str>;
}
pub trait ArtboardValue: ViewModelInstanceValue {
    type Artboard: BindableArtboard;
    type Instance;
    fn set_asset(&self, value: Rc<Self::Artboard>);
    fn asset(&self) -> Option<Rc<Self::Artboard>>;
    fn set_bound_view_model_instance(&self, value: Option<Rc<Self::Instance>>);
}
pub struct ViewModelInstanceArtboardRuntime<T: ArtboardValue> {
    base: ViewModelInstanceValueRuntime<T>,
}
impl<T: ArtboardValue> ViewModelInstanceArtboardRuntime<T> {
    pub fn new(value: Rc<T>) -> Self {
        Self {
            base: ViewModelInstanceValueRuntime::new(value),
        }
    }
    pub fn set_value(&self, artboard: Rc<T::Artboard>) {
        self.base.value().set_bound_view_model_instance(None);
        self.base.value().set_asset(artboard)
    }
    pub fn set_view_model_instance(&self, instance: Rc<T::Instance>) {
        self.base
            .value()
            .set_bound_view_model_instance(Some(instance))
    }
    pub fn artboard_name(&self) -> String {
        self.base
            .value()
            .asset()
            .and_then(|asset| asset.artboard_name().map(str::to_owned))
            .unwrap_or_default()
    }
    pub fn data_type(&self) -> DataType {
        DataType::Artboard
    }
    #[cfg(feature = "testing")]
    pub fn testing_value(&self) -> Option<Rc<T::Artboard>> {
        self.base.value().asset()
    }
}
