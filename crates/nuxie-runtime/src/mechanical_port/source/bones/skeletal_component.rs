use crate::mechanical_port::source::generated::bones::skeletal_component_base::SkeletalComponentBase;

/// The handwritten skeletal component adds no state or behavior to its
/// generated transform-component base.
pub struct SkeletalComponent {
    pub base: SkeletalComponentBase,
}

impl std::ops::Deref for SkeletalComponent {
    type Target = SkeletalComponentBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for SkeletalComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Default for SkeletalComponent {
    fn default() -> Self {
        Self {
            base: SkeletalComponentBase::default(),
        }
    }
}
