use std::{cell::RefCell, rc::Rc};

pub use nuxie_render_api::Factory;

/// Cloneable ownership seam for the one renderer factory occurrence shared by
/// importers. Borrowing is closure-scoped so an importer can be `'static`
/// without retaining a pointer or leaking a mutable factory reference.
#[derive(Clone)]
pub struct RuntimeFactoryHandle(Rc<RefCell<Box<dyn nuxie_render_api::Factory>>>);

impl RuntimeFactoryHandle {
    pub fn new(factory: Box<dyn nuxie_render_api::Factory>) -> Self {
        Self(Rc::new(RefCell::new(factory)))
    }

    pub fn with_factory_mut<R>(
        &self,
        use_factory: impl FnOnce(&mut dyn nuxie_render_api::Factory) -> R,
    ) -> R {
        use_factory(self.0.borrow_mut().as_mut())
    }
}
