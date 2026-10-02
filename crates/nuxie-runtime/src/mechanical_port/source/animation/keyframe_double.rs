use crate::mechanical_port::source::{
    animation::interpolating_keyframe::KeyFrameValueContext,
    core::CoreHandle,
    generated::{animation::keyframe_double_base::KeyFrameDoubleBase, core_registry::CoreRegistry},
    math::FloatContract,
};

#[inline]
fn interpolated_value(from: f32, to: f32, factor: f32) -> f32 {
    #[cfg(feature = "strict-fp")]
    {
        from + (to - from) * factor
    }
    #[cfg(not(feature = "strict-fp"))]
    {
        (to - from).mul_add(factor, from)
    }
}

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
    fn apply_value(object: &CoreHandle, key: i32, mix: f32, value: f32) -> bool {
        if mix == 1.0 {
            CoreRegistry::set_double_handle(object, key, value)
        } else {
            let Some(current) = CoreRegistry::get_double_handle(object, key) else {
                return false;
            };
            let mixed = current.contracted_mul_add(1.0 - mix, value * mix);
            CoreRegistry::set_double_handle(object, key, mixed)
        }
    }
    pub fn apply(
        &self,
        object: &CoreHandle,
        key: i32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) -> bool {
        if let Some(accumulator) = context.and_then(KeyFrameValueContext::blend_accumulator) {
            accumulator
                .borrow_mut()
                .apply_double(object, key, mix, self.effective_value(context));
            return true;
        }
        Self::apply_value(object, key, mix, self.effective_value(context))
    }
    pub fn apply_interpolation(
        &self,
        object: &CoreHandle,
        key: i32,
        current_time: f32,
        next: &Self,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) -> bool {
        let from = self.effective_value(context);
        let to = next.effective_value(context);
        let factor = (current_time - self.base.base.base.seconds())
            / (next.base.base.base.seconds() - self.base.base.base.seconds());
        let value = self
            .base
            .base
            .transform_value(context, from, to, factor)
            .unwrap_or_else(|| interpolated_value(from, to, factor));
        if let Some(accumulator) = context.and_then(KeyFrameValueContext::blend_accumulator) {
            accumulator
                .borrow_mut()
                .apply_double(object, key, mix, value);
            return true;
        }
        Self::apply_value(object, key, mix, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanical_port::source::{
        core::CoreArena, custom_property_number::CustomPropertyNumber,
        generated::custom_property_number_base::CustomPropertyNumberBase,
    };

    #[test]
    fn direct_mixing_keeps_the_right_product_rounded_before_the_sum() {
        let arena = CoreArena::default();
        let object = arena.insert(CustomPropertyNumber::default());
        let key = i32::from(CustomPropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY);
        let current = f32::from_bits(0x3d82_a90a);
        let value = f32::from_bits(0x3f14_7ae1);
        let mix = f32::from_bits(0x3ed0_f81d);
        assert!(CoreRegistry::set_double_handle(&object, key, current));
        assert!(KeyFrameDouble::apply_value(&object, key, mix, value));
        #[cfg(feature = "strict-fp")]
        let expected = current * (1.0 - mix) + value * mix;
        #[cfg(not(feature = "strict-fp"))]
        let expected = current.mul_add(1.0 - mix, value * mix);
        assert_eq!(
            CoreRegistry::get_double_handle(&object, key)
                .unwrap()
                .to_bits(),
            expected.to_bits()
        );
        assert!(KeyFrameDouble::apply_value(&object, key, 1.0, value));
        assert_eq!(
            CoreRegistry::get_double_handle(&object, key)
                .unwrap()
                .to_bits(),
            value.to_bits()
        );
    }
}
