use std::{cell::RefCell, rc::Rc};

use crate::mechanical_port::source::{
    artboard::{
        ArtboardInstance, RuntimeArtboardInstanceHandle, RuntimeArtboardInstanceWeakHandle,
    },
    core::CoreHandle,
    file::RuntimeFileWeakHandle,
};

/// The runtime occurrence created when an authored Artboard is bound into a
/// view-model value. The authored source remains a CoreHandle; the instantiated
/// scene remains the singular RuntimeArtboardInstanceHandle.
pub struct BindableArtboard {
    file: Option<RuntimeFileWeakHandle>,
    artboard: RuntimeArtboardInstanceHandle,
    source_artboard: Option<CoreHandle>,
}

#[derive(Clone)]
pub struct RuntimeBindableArtboardHandle(Rc<RefCell<BindableArtboard>>);

impl RuntimeBindableArtboardHandle {
    pub fn new(
        file: Option<RuntimeFileWeakHandle>,
        artboard: RuntimeArtboardInstanceHandle,
        source_artboard: Option<CoreHandle>,
    ) -> Self {
        Self(Rc::new(RefCell::new(BindableArtboard {
            file,
            artboard,
            source_artboard,
        })))
    }

    pub fn file(&self) -> Option<RuntimeFileWeakHandle> {
        self.0.borrow().file.clone()
    }

    pub fn source_artboard_handle(&self) -> Option<CoreHandle> {
        self.0.borrow().source_artboard.clone()
    }

    pub fn artboard_handle(&self) -> RuntimeArtboardInstanceHandle {
        self.0.borrow().artboard.clone()
    }

    pub fn artboard_weak_handle(&self) -> RuntimeArtboardInstanceWeakHandle {
        self.artboard_handle().downgrade()
    }

    pub fn with_artboard<R>(&self, use_artboard: impl FnOnce(&ArtboardInstance) -> R) -> R {
        self.artboard_handle().with_artboard(use_artboard)
    }

    pub fn with_artboard_mut<R>(&self, use_artboard: impl FnOnce(&mut ArtboardInstance) -> R) -> R {
        self.artboard_handle().with_artboard_mut(use_artboard)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
