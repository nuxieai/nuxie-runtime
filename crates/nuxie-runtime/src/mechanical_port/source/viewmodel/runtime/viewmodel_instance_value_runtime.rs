use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DataType {
    None,
    String,
    Number,
    Boolean,
    Color,
    List,
    Enum,
    Trigger,
    ViewModel,
    SymbolListIndex,
    AssetImage,
    AssetFont,
    AssetBlob,
    Artboard,
}

pub trait ViewModelInstanceValue {
    fn name(&self) -> &str;
    fn add_dependent(&self, dependent: *const ());
    fn remove_dependent(&self, dependent: *const ());
}

pub struct ViewModelInstanceValueRuntime<T: ViewModelInstanceValue> {
    value: Rc<T>,
    dependent_identity: Rc<()>,
    has_changed: Cell<bool>,
}

impl<T: ViewModelInstanceValue> ViewModelInstanceValueRuntime<T> {
    pub fn new(value: Rc<T>) -> Self {
        let dependent_identity = Rc::new(());
        value.add_dependent(Rc::as_ptr(&dependent_identity));
        Self {
            value,
            dependent_identity,
            has_changed: Cell::new(false),
        }
    }
    pub fn add_dirt(&self, _dirt: u32, _recurse: bool) {
        self.has_changed.set(true);
    }
    pub fn clear_changes(&self) {
        self.has_changed.set(false);
    }
    pub fn has_changed(&self) -> bool {
        self.has_changed.get()
    }
    pub fn flush_changes(&self) -> bool {
        if self.has_changed.replace(false) {
            true
        } else {
            false
        }
    }
    pub fn name(&self) -> &str {
        self.value.name()
    }
    pub fn value(&self) -> &Rc<T> {
        &self.value
    }
    pub fn relink_data_bind(&self) {}
}

impl<T: ViewModelInstanceValue> Drop for ViewModelInstanceValueRuntime<T> {
    fn drop(&mut self) {
        self.value
            .remove_dependent(Rc::as_ptr(&self.dependent_identity));
    }
}
