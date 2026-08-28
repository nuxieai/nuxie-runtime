use crate::mechanical_port::source::{
    animation::interpolating_keyframe::KeyFrameValueContext,
    generated::{
        animation::keyframe_color_base::KeyFrameColorBase,
        core_registry::{CoreRegistry, CoreRegistryObject},
    },
    shapes::paint::color::color_lerp,
};
#[derive(Default)]
pub struct KeyFrameColor {
    pub base: KeyFrameColorBase,
}
impl KeyFrameColor {
    pub fn effective_value(&self, context: Option<&dyn KeyFrameValueContext>) -> i32 {
        context
            .and_then(|c| {
                self.base
                    .handle()
                    .and_then(|keyframe| c.color_value(&keyframe))
            })
            .unwrap_or_else(|| self.base.value())
    }
    fn apply_value(object: &mut dyn CoreRegistryObject, key: i32, mix: f32, value: i32) {
        let value = if mix == 1.0 {
            value
        } else {
            color_lerp(
                CoreRegistry::get_color(object, key) as u32,
                value as u32,
                mix,
            ) as i32
        };
        CoreRegistry::set_color(object, key, value);
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
        let factor = (current_time - self.base.base.base.seconds())
            / (next.base.base.base.seconds() - self.base.base.base.seconds());
        let factor = self.base.base.transform(context, factor).unwrap_or(factor);
        let value = color_lerp(
            self.effective_value(context) as u32,
            next.effective_value(context) as u32,
            factor,
        ) as i32;
        Self::apply_value(object, key, mix, value);
    }
}
