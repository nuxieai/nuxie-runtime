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
        matches!(type_key, Self::TYPE_KEY | 5 | 120 | 129 | 1)
    }

    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}
