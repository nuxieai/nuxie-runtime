use crate::mechanical_port::source::{
    animation::interpolating_keyframe::KeyFrameValueContext,
    core::CoreHandle,
    generated::{animation::keyframe_color_base::KeyFrameColorBase, core_registry::CoreRegistry},
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
    fn apply_value(object: &CoreHandle, key: i32, mix: f32, value: i32) {
        let value = if mix == 1.0 {
            value
        } else {
            color_lerp(
                CoreRegistry::get_color_handle(object, key).expect("live color target") as u32,
                value as u32,
                mix,
            ) as i32
        };
        CoreRegistry::set_color_handle(object, key, value);
    }
    pub fn apply(
        &self,
        object: &CoreHandle,
        key: i32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        if let Some(accumulator) = context.and_then(KeyFrameValueContext::blend_accumulator) {
            accumulator.borrow_mut().apply_color(
                object,
                key,
                mix,
                self.effective_value(context) as u32,
            );
            return;
        }
        Self::apply_value(object, key, mix, self.effective_value(context));
    }
    pub fn apply_interpolation(
        &self,
        object: &CoreHandle,
        key: i32,
        current_time: f32,
        next: &Self,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        let from_value = self.effective_value(context) as u32;
        let to_value = next.effective_value(context) as u32;
        let factor = (current_time - self.base.base.base.seconds())
            / (next.base.base.base.seconds() - self.base.base.base.seconds());
        let factor = self.base.base.transform(context, factor).unwrap_or(factor);
        let value = color_lerp(from_value, to_value, factor) as i32;
        if let Some(accumulator) = context.and_then(KeyFrameValueContext::blend_accumulator) {
            accumulator
                .borrow_mut()
                .apply_color(object, key, mix, value as u32);
            return;
        }
        Self::apply_value(object, key, mix, value);
    }
}
