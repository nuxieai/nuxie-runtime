// Mirrors src/animation/blend_animation.cpp and
// include/rive/animation/blend_animation.hpp.
pub(crate) trait RuntimeBlendAnimation {
    fn retained_animation(&self) -> RuntimeLinearAnimationHandle;

    fn animation(&self) -> RuntimeLinearAnimationHandle {
        self.retained_animation()
    }
}

pub(crate) fn blend_animation_from_imported(
    animation: &nuxie_binary::RuntimeBlendAnimation<'_>,
    animation_index_by_global: &std::collections::BTreeMap<u32, usize>,
) -> RuntimeLinearAnimationHandle {
    animation
        .animation
        .and_then(|animation| animation_index_by_global.get(&animation.id).copied())
        .map(RuntimeLinearAnimationHandle::new)
        .unwrap_or_else(RuntimeLinearAnimationHandle::empty)
}
