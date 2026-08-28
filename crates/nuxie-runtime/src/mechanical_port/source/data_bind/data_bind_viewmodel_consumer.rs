use crate::mechanical_port::source::core::CoreHandle;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewModelConsumerCoreType {
    ViewModelInstanceViewModel,
    Other,
}
pub trait DataBindViewModelConsumer {
    fn update_view_model(&mut self, value: CoreHandle);
}
pub trait ViewModelConsumerCore {
    fn core_type(&self) -> ViewModelConsumerCoreType;
    fn as_view_model_consumer(&mut self) -> &mut dyn DataBindViewModelConsumer;
}
pub fn from(
    component: &mut dyn ViewModelConsumerCore,
) -> Option<&mut dyn DataBindViewModelConsumer> {
    (component.core_type() == ViewModelConsumerCoreType::ViewModelInstanceViewModel)
        .then(|| component.as_view_model_consumer())
}
use crate::mechanical_port::source::core::CoreHandle;
