use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader,
    layout::n_slicer::NSlicer,
};

pub struct NSlicerBase {
    pub base: ContainerComponent,
}

impl Default for NSlicerBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
        }
    }
}

impl NSlicerBase {
    pub const TYPE_KEY: u16 = 493;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> NSlicer {
        let mut cloned = NSlicer::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(self, &mut cloned);
        cloned.base = base;
        cloned
    }
}

impl std::ops::Deref for NSlicerBase {
    type Target = ContainerComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for NSlicerBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
