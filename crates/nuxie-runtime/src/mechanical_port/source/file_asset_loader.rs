use std::{cell::RefCell, rc::Rc};

use crate::mechanical_port::source::{core::CoreHandle, factory::RuntimeFactoryHandle};

pub trait FileAssetLoader {
    fn load_contents(
        &mut self,
        asset: CoreHandle,
        in_band_bytes: &[u8],
        factory: &RuntimeFactoryHandle,
    ) -> bool;
}

#[derive(Clone)]
pub struct FileAssetLoaderRef(Rc<RefCell<Box<dyn FileAssetLoader>>>);

impl FileAssetLoaderRef {
    pub fn new(loader: Box<dyn FileAssetLoader>) -> Self {
        Self(Rc::new(RefCell::new(loader)))
    }

    pub fn with_loader_mut<R>(&self, use_loader: impl FnOnce(&mut dyn FileAssetLoader) -> R) -> R {
        use_loader(self.0.borrow_mut().as_mut())
    }
}
