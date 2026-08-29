use crate::mechanical_port::source::dirtyable::Dirtyable;

pub trait ViewModelValueDependent: Dirtyable {
    fn relink_data_bind(&mut self);

    /// A number's property callback owns its mutable Core borrow. Dependent
    /// listeners may read that same value synchronously (e.g. ListPath); carry
    /// its current scalar through the call instead of borrowing its owner twice.
    fn add_dirt_from_number(
        &mut self,
        value: crate::mechanical_port::source::component_dirt::ComponentDirt,
        recurse: bool,
        _source: &crate::mechanical_port::source::core::CoreHandle,
        _number_value: f32,
    ) {
        self.add_dirt(value, recurse);
    }

    /// A trigger's property callback still owns its mutable Core borrow. Pass
    /// that same source and its current scalar through the synchronous callback
    /// instead of requiring a dependent to borrow the trigger again.
    fn add_dirt_from_trigger(
        &mut self,
        value: crate::mechanical_port::source::component_dirt::ComponentDirt,
        recurse: bool,
        _source: &crate::mechanical_port::source::core::CoreHandle,
        _trigger_value: u32,
    ) {
        self.add_dirt(value, recurse);
    }
}
