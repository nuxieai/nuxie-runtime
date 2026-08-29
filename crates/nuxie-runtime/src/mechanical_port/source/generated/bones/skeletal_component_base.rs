use crate::mechanical_port::source::transform_component::TransformComponent;

pub struct SkeletalComponentBase {
    pub base: TransformComponent,
}

impl Default for SkeletalComponentBase {
    fn default() -> Self {
        Self {
            base: TransformComponent::default(),
        }
    }
}

impl SkeletalComponentBase {
    pub const TYPE_KEY: u16 = 39;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 38 | 91 | 11 | 10)
    }

    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}

impl std::ops::Deref for SkeletalComponentBase {
    type Target = TransformComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for SkeletalComponentBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
