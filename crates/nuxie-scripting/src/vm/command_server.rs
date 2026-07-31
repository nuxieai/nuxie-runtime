// Direct owner for pinned C++ `src/command_server.cpp`'s scripting render
// context installation (`CommandServer::processCommands`, lines 675-704).
//
// The C++ CommandServer retains one Factory pointer for its full lifetime and
// installs that same pointer on ScriptingContext before File::import. Rust
// retains an owned PersistentFactoryContext instead: every proxy shares the
// same underlying factory identity, and the VM keeps that identity alive.
use std::cell::RefCell;
use std::rc::Rc;

use luaur_rt::{Error, Result};
use nuxie_render_api::{Factory as RenderFactory, PersistentFactoryContext};

#[derive(Clone, Default)]
pub(crate) struct PersistentRenderContext {
    context: Rc<RefCell<Option<PersistentFactoryContext>>>,
}

impl PersistentRenderContext {
    /// Install the VM's one render context before imported script code runs.
    ///
    /// Reinstalling the identical factory is an idempotent integration
    /// adapter for callers whose existing interfaces still carry `factory`.
    /// A different pointer is rejected: backend resources cannot migrate
    /// between the device/factory domains of one scripting VM.
    pub(crate) fn install(&self, factory: &mut dyn RenderFactory) -> Result<()> {
        let candidate = factory.persistent_context().ok_or_else(|| {
            Error::runtime("scripted files require a PersistentFactory renderer context")
        })?;
        let candidate_identity = candidate.identity();
        let mut installed = self.context.borrow_mut();
        match installed.as_ref() {
            None => {
                *installed = Some(candidate);
                Ok(())
            }
            Some(current) if current.identity() == candidate_identity => Ok(()),
            Some(_) => Err(Error::runtime(
                "scripting VM is already bound to a different persistent render context",
            )),
        }
    }

    /// Verify a callback adapter still carries the factory installed before
    /// import. Callback entry points must never establish ownership.
    pub(crate) fn verify(&self, factory: &mut dyn RenderFactory) -> Result<()> {
        let candidate = factory.persistent_context().ok_or_else(|| {
            Error::runtime("scripted files require a PersistentFactory renderer context")
        })?;
        let installed = self.context.borrow();
        match installed.as_ref() {
            Some(current) if current.identity() == candidate.identity() => Ok(()),
            Some(_) => Err(Error::runtime(
                "scripting VM is already bound to a different persistent render context",
            )),
            None => Err(Error::runtime(
                "renderer callback requires the VM's pre-import persistent render context",
            )),
        }
    }

    pub(crate) fn with_factory<R>(
        &self,
        f: impl FnOnce(&mut dyn RenderFactory) -> Result<R>,
    ) -> Result<R> {
        let context = self.context.borrow().clone();
        let Some(context) = context else {
            return Err(Error::runtime(
                "renderer resource allocation requires the VM's persistent render context",
            ));
        };
        context.with_factory(f)
    }
}
