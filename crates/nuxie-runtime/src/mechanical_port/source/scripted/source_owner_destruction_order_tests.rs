use super::*;
use crate::mechanical_port::source::{core::CoreArena, data_bind::data_bind::DataBind};
use std::{any::Any, cell::RefCell, rc::Rc};
struct CachedDropProbe {
    events: Rc<RefCell<Vec<(&'static str, bool)>>>,
    binding: CoreHandle,
}
impl DataValue for CachedDropProbe {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
impl Drop for CachedDropProbe {
    fn drop(&mut self) {
        self.events
            .borrow_mut()
            .push(("cache", self.binding.is_alive()));
    }
}
#[test]
fn scripted_converter_destroys_cache_and_script_base_before_owned_bindings() {
    let arena = CoreArena::default();
    let events = Rc::new(RefCell::new(Vec::new()));
    let binding = arena.insert(DataBind::default());
    let mut converter = ScriptedDataConverter::default();
    converter.base.base.add_data_bind(binding.clone());
    converter.data_value = Some(Box::new(CachedDropProbe {
        events: events.clone(),
        binding: binding.clone(),
    }));
    let seen = events.clone();
    let child = binding.clone();
    converter
        .scripted
        .retain_failed_script_property_cleanup(Rc::new(move || {
            seen.borrow_mut().push(("script", child.is_alive()))
        }));
    let owner = arena.insert(converter);
    assert!(owner.remove_occurrence());
    assert_eq!(&*events.borrow(), &[("cache", true), ("script", true)]);
    assert!(!binding.is_alive());
}
