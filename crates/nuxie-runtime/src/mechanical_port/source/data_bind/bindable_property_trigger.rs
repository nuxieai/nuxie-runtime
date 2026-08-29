use crate::mechanical_port::source::generated::data_bind::bindable_property_trigger_base::BindablePropertyTriggerBase;

#[derive(Default)]
pub struct BindablePropertyTrigger {
    pub base: BindablePropertyTriggerBase,
}
impl BindablePropertyTrigger {
    pub const DEFAULT_VALUE: u32 = 0;
}
