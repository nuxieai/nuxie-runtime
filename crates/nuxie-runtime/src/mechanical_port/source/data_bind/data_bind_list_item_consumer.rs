use crate::mechanical_port::source::core::CoreHandle;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ListConsumerCoreType {
    ArtboardComponentList,
    ListPath,
    Text,
    ViewModelInstanceList,
    Other,
}
pub trait DataBindListItemConsumer {
    fn update_list(&mut self, list: &[CoreHandle]);
}
pub trait ListConsumerCore {
    fn core_type(&self) -> ListConsumerCoreType;
    fn as_list_consumer(&mut self) -> &mut dyn DataBindListItemConsumer;
}
pub fn from(component: &mut dyn ListConsumerCore) -> Option<&mut dyn DataBindListItemConsumer> {
    matches!(
        component.core_type(),
        ListConsumerCoreType::ArtboardComponentList
            | ListConsumerCoreType::ListPath
            | ListConsumerCoreType::Text
            | ListConsumerCoreType::ViewModelInstanceList
    )
    .then(|| component.as_list_consumer())
}
