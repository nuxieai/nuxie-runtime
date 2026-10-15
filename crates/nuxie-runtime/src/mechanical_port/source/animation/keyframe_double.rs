use crate::mechanical_port::source::{
    animation::{interpolating_keyframe::KeyFrameValueContext, keyframe_cells::KeyFrameEntry},
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
                    .handle_ref()
                    .and_then(|keyframe| c.number_value(keyframe))
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
            // Select the source receiver first, then evaluate the open value
            // callback before borrowing the accumulator for applyDouble.
            let value = self.effective_value(context);
            accumulator
                .borrow_mut()
                .apply_double(object, key, mix, value);
            return true;
        }
        Self::apply_value(object, key, mix, self.effective_value(context))
    }

    /// apply() for the double keyframe `keyframe`, whose value `stored` its
    /// shared cells hold: the same accumulator, bound-value and mix steps,
    /// without borrowing the keyframe.
    pub(crate) fn apply_stored(
        keyframe: &CoreHandle,
        stored: f32,
        object: &CoreHandle,
        key: i32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) -> bool {
        let effective_value = || {
            context
                .and_then(|context| context.number_value(keyframe))
                .unwrap_or(stored)
        };
        if let Some(accumulator) = context.and_then(KeyFrameValueContext::blend_accumulator) {
            let value = effective_value();
            accumulator
                .borrow_mut()
                .apply_double(object, key, mix, value);
            return true;
        }
        Self::apply_value(object, key, mix, effective_value())
    }

    /// apply_interpolation() from the double keyframe `from` to the double
    /// keyframe `to` when `from` has no interpolator (so interpolation is
    /// linear), read from their shared cells: the bound values, then the
    /// times, then the accumulator or mix step, without borrowing either
    /// keyframe.
    pub(crate) fn interpolate_stored(
        from: &KeyFrameEntry,
        to: &KeyFrameEntry,
        object: &CoreHandle,
        key: i32,
        current_time: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) -> bool {
        let effective_value = |frame: &KeyFrameEntry| {
            context
                .and_then(|context| context.number_value(&frame.handle))
                .or_else(|| frame.cells.double_value())
                .expect("a double keyframe's cells hold its value")
        };
        let from_value = effective_value(from);
        let to_value = effective_value(to);
        let from_seconds = from.cells.seconds();
        let factor = (current_time - from_seconds) / (to.cells.seconds() - from_seconds);
        let value = interpolated_value(from_value, to_value, factor);
        if let Some(accumulator) = context.and_then(KeyFrameValueContext::blend_accumulator) {
            accumulator
                .borrow_mut()
                .apply_double(object, key, mix, value);
            return true;
        }
        Self::apply_value(object, key, mix, value)
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

#[cfg(test)]
#[path = "keyframe_double_context_tests.rs"]
mod context_tests;
