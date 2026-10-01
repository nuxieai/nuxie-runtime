use crate::mechanical_port::source::{
    animation::{interpolating_keyframe::KeyFrameValueContext, keyframe::KeyFrame},
    generated::{
        animation::keyframe_uint_base::KeyFrameUintBase,
        core_registry::{CoreRegistry, CoreRegistryObject},
    },
};

#[derive(Default)]
pub struct KeyFrameUint {
    pub base: KeyFrameUintBase,
}

fn round_to_uint(value: f32) -> u32 {
    if value <= 0.0 {
        0
    } else {
        (value.round() as i64) as u32
    }
}

fn apply_uint_with_completion(
    object: &mut dyn CoreRegistryObject,
    property_key: i32,
    mix: f32,
    value: u32,
    completion: &mut crate::source::core::PropertySetterCompletion,
) {
    if mix == 1.0 {
        CoreRegistry::set_uint_with_completion(object, property_key, value, completion);
    } else {
        let mixi = 1.0 - mix;
        let current = CoreRegistry::get_uint(object, property_key);
        CoreRegistry::set_uint_with_completion(
            object,
            property_key,
            round_to_uint(current as f32 * mixi + value as f32 * mix),
            completion,
        );
    }
}

impl KeyFrameUint {
    pub fn apply(
        &self,
        object: &mut dyn CoreRegistryObject,
        property_key: i32,
        mix: f32,
        _context: Option<&dyn KeyFrameValueContext>,
    ) {
        let mut completion = crate::source::core::PropertySetterCompletion::default();
        self.apply_with_completion(object, property_key, mix, _context, &mut completion);
        completion.finish();
    }
    pub fn apply_with_completion(
        &self,
        object: &mut dyn CoreRegistryObject,
        property_key: i32,
        mix: f32,
        _context: Option<&dyn KeyFrameValueContext>,
        completion: &mut crate::source::core::PropertySetterCompletion,
    ) {
        if CoreRegistry::is_interpolatable_uint(property_key as u32) {
            apply_uint_with_completion(object, property_key, mix, self.base.value(), completion);
            return;
        }
        CoreRegistry::set_uint_with_completion(object, property_key, self.base.value(), completion);
    }

    pub fn apply_interpolation(
        &self,
        object: &mut dyn CoreRegistryObject,
        property_key: i32,
        current_time: f32,
        next_frame: &KeyFrame,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        let mut completion = crate::source::core::PropertySetterCompletion::default();
        self.apply_interpolation_with_completion(
            object,
            property_key,
            current_time,
            next_frame,
            mix,
            context,
            &mut completion,
        );
        completion.finish();
    }
    pub fn apply_interpolation_with_completion(
        &self,
        object: &mut dyn CoreRegistryObject,
        property_key: i32,
        current_time: f32,
        next_frame: &KeyFrame,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
        completion: &mut crate::source::core::PropertySetterCompletion,
    ) {
        if !CoreRegistry::is_interpolatable_uint(property_key as u32) {
            CoreRegistry::set_uint_with_completion(
                object,
                property_key,
                self.base.value(),
                completion,
            );
            return;
        }

        let next_value = next_frame
            .base
            .base
            .handle()
            .and_then(|next_frame| {
                next_frame.with_downcast::<KeyFrameUint, _>(|next_uint| next_uint.base.value())
            })
            .expect("uint keyframe interpolation requires a KeyFrameUint next frame");
        let from_value = self.base.value() as f32;
        let to_value = next_value as f32;
        let f = (current_time - self.base.base.base.seconds())
            / (next_frame.seconds() - self.base.base.base.seconds());
        let frame_value = self
            .base
            .base
            .transform_value(context, from_value, to_value, f)
            .unwrap_or_else(|| from_value + (to_value - from_value) * f);

        apply_uint_with_completion(
            object,
            property_key,
            mix,
            round_to_uint(frame_value),
            completion,
        );
    }
}
