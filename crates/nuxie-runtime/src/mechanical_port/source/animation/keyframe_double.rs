use crate::mechanical_port::source::{
    animation::interpolating_keyframe::KeyFrameValueContext,
    generated::{
        animation::keyframe_double_base::KeyFrameDoubleBase,
        core_registry::{CoreRegistry, CoreRegistryObject},
    },
};
#[derive(Default)]
pub struct KeyFrameDouble {
    pub base: KeyFrameDoubleBase,
}
impl KeyFrameDouble {
    pub fn effective_value(&self, context: Option<&dyn KeyFrameValueContext>) -> f32 {
        context
            .and_then(|c| {
                self.base
                    .handle()
                    .and_then(|keyframe| c.number_value(&keyframe))
            })
            .unwrap_or_else(|| self.base.value())
    }
    fn apply_value(object: &mut dyn CoreRegistryObject, key: i32, mix: f32, value: f32) {
        if mix == 1.0 {
            CoreRegistry::set_double(object, key, value);
        } else {
            let mixed = CoreRegistry::get_double(object, key) * (1.0 - mix) + value * mix;
            CoreRegistry::set_double(object, key, mixed);
        }
    }
    pub fn apply(
        &self,
        object: &mut dyn CoreRegistryObject,
        key: i32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        Self::apply_value(object, key, mix, self.effective_value(context));
    }
    pub fn apply_interpolation(
        &self,
        object: &mut dyn CoreRegistryObject,
        key: i32,
        current_time: f32,
        next: &Self,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        let from = self.effective_value(context);
        let to = next.effective_value(context);
        let factor = (current_time - self.base.base.base.seconds())
            / (next.base.base.base.seconds() - self.base.base.base.seconds());
        let value = self
            .base
            .base
            .transform_value(context, from, to, factor)
            .unwrap_or_else(|| from + (to - from) * factor);
        Self::apply_value(object, key, mix, value);
    }
}
