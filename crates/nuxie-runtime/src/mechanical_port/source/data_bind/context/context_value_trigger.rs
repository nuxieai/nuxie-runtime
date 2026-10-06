use super::context_value::{ContextApplyBinding, DataBindContextValue};
use crate::mechanical_port::source::data_bind::data_values::data_value_trigger::DataValueTrigger;
pub struct DataBindContextValueTrigger {
    base: DataBindContextValue,
    source_count: u32,
    source_count_known: bool,
    target_count_known: bool,
}
impl crate::source::data_bind::data_bind::BindContextValue for DataBindContextValueTrigger {
    fn apply(
        &mut self,
        target: Option<crate::source::core::CoreHandle>,
        key: u32,
        main: bool,
        bind: crate::source::core::CoreHandle,
    ) {
        let mut binding = super::context_value::CoreBinding::new(bind, target, key);
        Self::apply(self, key, main, &mut binding);
    }
    fn refresh_target_value(&mut self, bind: crate::source::core::CoreHandle) {
        self.base
            .refresh_target_value(&super::context_value::CoreBinding::for_bind(bind));
    }
    fn invalidate(&mut self) {
        self.base.invalidate();
    }
    fn request_source_write(&mut self) {
        self.base.source_write_requested = true;
    }
    fn invalidation_handle(&self) -> std::rc::Rc<std::cell::Cell<bool>> {
        self.base.invalidation_handle()
    }
    fn apply_to_source(
        &mut self,
        target: crate::source::core::CoreHandle,
        key: u32,
        main: bool,
        bind: crate::source::core::CoreHandle,
    ) {
        let mut binding = super::context_value::CoreBinding::new(bind, Some(target.clone()), key);
        self.apply_to_source(Some(target), key, main, &mut binding);
    }
}
impl DataBindContextValueTrigger {
    pub fn new(binding: &mut dyn ContextApplyBinding) -> Self {
        Self {
            base: DataBindContextValue::new(binding),
            source_count: 0,
            source_count_known: false,
            target_count_known: false,
        }
    }
    pub fn apply(
        &mut self,
        property_key: u32,
        is_main_direction: bool,
        binding: &mut dyn ContextApplyBinding,
    ) {
        self.base.sync_source_value(binding);
        if !binding.target_is_trigger() {
            let calculated = binding.convert(self.base.data_value().unwrap(), is_main_direction);
            let value = calculated
                .as_any()
                .downcast_ref::<DataValueTrigger>()
                .map_or(DataValueTrigger::DEFAULT_VALUE, DataValueTrigger::value);
            binding.set_uint(property_key, value);
            return;
        }
        let count = if let Some(value) = self
            .base
            .data_value()
            .and_then(|v| v.as_any().downcast_ref::<DataValueTrigger>())
        {
            value.value()
        } else {
            let calculated = binding.convert(self.base.data_value().unwrap(), is_main_direction);
            calculated
                .as_any()
                .downcast_ref::<DataValueTrigger>()
                .map_or(DataValueTrigger::DEFAULT_VALUE, DataValueTrigger::value)
        };
        if self.source_count_known && count != self.source_count {
            binding.set_uint(property_key, binding.uint_value().wrapping_add(1));
        }
        self.source_count = count;
        self.source_count_known = true;
        self.target_count_known = true;
    }

    pub fn apply_to_source(
        &mut self,
        target: Option<crate::source::core::CoreHandle>,
        property_key: u32,
        is_main_direction: bool,
        binding: &mut dyn ContextApplyBinding,
    ) {
        if !binding.source_is_trigger() {
            self.base
                .apply_to_source(target, property_key, is_main_direction, binding);
            return;
        }
        let target_fired =
            self.base.target_value.sync_target_value(binding) && self.target_count_known;
        self.target_count_known = true;
        let requested = std::mem::take(&mut self.base.source_write_requested);
        self.base.is_valid.set(true);
        if !target_fired && !requested {
            return;
        }
        binding.suppress_dirt(true);
        binding.fire_source_trigger();
        binding.suppress_dirt(false);
        self.source_count = binding.source_uint();
        self.source_count_known = true;
    }
}
