pub use nuxie_render_api::Factory;
use nuxie_render_api::PersistentFactoryContext;

/// Cloneable ownership seam for the one renderer factory occurrence shared by
/// the imported File, its Artboards, and scripting. Borrowing is closure-scoped
/// and every clone retains the same persistent renderer identity.
#[derive(Clone)]
pub struct RuntimeFactoryHandle(PersistentFactoryContext);

impl RuntimeFactoryHandle {
    pub fn from_factory(factory: &mut dyn Factory) -> Option<Self> {
        factory.persistent_context().map(Self)
    }

    pub fn from_context(context: PersistentFactoryContext) -> Self {
        Self(context)
    }

    pub fn persistent_context(&self) -> PersistentFactoryContext {
        self.0.clone()
    }

    pub fn with_factory_mut<R>(&self, use_factory: impl FnOnce(&mut dyn Factory) -> R) -> R {
        self.0.with_factory(use_factory)
    }
}
