use crate::mechanical_port::source::math::FloatContract;

pub type ColorInt = u32;
pub fn color_argb(a: i32, r: i32, g: i32, b: i32) -> ColorInt {
    ((((a & 0xff) << 24) | ((r & 0xff) << 16) | ((g & 0xff) << 8) | (b & 0xff)) as u32)
        & 0xffff_ffff
}
pub fn color_red(value: ColorInt) -> u32 {
    (value & 0x00ff_0000) >> 16
}
pub fn color_green(value: ColorInt) -> u32 {
    (value & 0x0000_ff00) >> 8
}
pub fn color_blue(value: ColorInt) -> u32 {
    value & 0x0000_00ff
}
pub fn color_alpha(value: ColorInt) -> u32 {
    (value & 0xff00_0000) >> 24
}
pub fn unpack_color_to_rgba8(color: ColorInt, out: &mut [u8; 4]) {
    *out = [
        color_red(color) as u8,
        color_green(color) as u8,
        color_blue(color) as u8,
        color_alpha(color) as u8,
    ];
}
pub fn unpack_color_to_rgba32f(color: ColorInt, out: &mut [f32; 4]) {
    *out = [
        color_red(color) as f32 * (1.0 / 255.0),
        color_green(color) as f32 * (1.0 / 255.0),
        color_blue(color) as f32 * (1.0 / 255.0),
        color_alpha(color) as f32 * (1.0 / 255.0),
    ];
}
pub fn unpack_color_to_rgba32f_premul(color: ColorInt, out: &mut [f32; 4]) {
    unpack_color_to_rgba32f(color, out);
    let alpha = out[3];
    out[0] *= alpha;
    out[1] *= alpha;
    out[2] *= alpha;
}
pub fn color_opacity(value: ColorInt) -> f32 {
    color_alpha(value) as f32 / 0xff as f32
}
pub fn opacity_to_alpha(opacity: f32) -> u8 {
    (255.0 * opacity.min(1.0).max(0.0)).round() as u8
}
pub fn color_with_alpha(value: ColorInt, alpha: u32) -> ColorInt {
    color_argb(
        alpha as i32,
        color_red(value) as i32,
        color_green(value) as i32,
        color_blue(value) as i32,
    )
}
pub fn color_with_opacity(value: ColorInt, opacity: f32) -> ColorInt {
    color_with_alpha(value, opacity_to_alpha(opacity) as u32)
}
pub fn color_modulate_opacity(value: ColorInt, opacity: f32) -> ColorInt {
    color_with_alpha(
        value,
        opacity_to_alpha(color_opacity(value) * opacity) as u32,
    )
}
pub fn color_modulate(value: ColorInt, color: ColorInt, opacity: f32) -> ColorInt {
    let mul8 = |a: u32, b: u32| ((a * b + 127) / 255) as i32;
    color_argb(
        opacity_to_alpha(color_opacity(value) * color_opacity(color) * opacity) as i32,
        mul8(color_red(value), color_red(color)),
        mul8(color_green(value), color_green(color)),
        mul8(color_blue(value), color_blue(color)),
    )
}
fn lerp(a: u32, b: u32, mix: f32) -> u32 {
    // Shipping C++ rounds b * mix, then contracts the left product and sum.
    let value = (a as f32).contracted_mul_add(1.0 - mix, b as f32 * mix);
    // std::min/max select their first operand for unordered comparisons.
    let upper = if value < 255.0 { value } else { 255.0 };
    let bounded = if 0.0 < upper { upper } else { 0.0 };
    bounded.round() as u32
}
pub fn color_lerp(from: ColorInt, to: ColorInt, mix: f32) -> ColorInt {
    color_argb(
        lerp(color_alpha(from), color_alpha(to), mix) as i32,
        lerp(color_red(from), color_red(to), mix) as i32,
        lerp(color_green(from), color_green(to), mix) as i32,
        lerp(color_blue(from), color_blue(to), mix) as i32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_unpacking_uses_the_source_rounded_reciprocal_before_premul() {
        let reciprocal = 1.0_f32 / 255.0;
        for channel in 0..=255 {
            let color = color_argb(37, channel, 89, 117);
            let normalized = [
                channel as f32 * reciprocal,
                89.0 * reciprocal,
                117.0 * reciprocal,
                37.0 * reciprocal,
            ];
            let mut unpacked = [0.0; 4];
            unpack_color_to_rgba32f(color, &mut unpacked);
            assert_eq!(unpacked.map(f32::to_bits), normalized.map(f32::to_bits));

            unpack_color_to_rgba32f_premul(color, &mut unpacked);
            let expected = [
                normalized[0] * normalized[3],
                normalized[1] * normalized[3],
                normalized[2] * normalized[3],
                normalized[3],
            ];
            assert_eq!(unpacked.map(f32::to_bits), expected.map(f32::to_bits));
        }
    }

    #[test]
    fn unordered_channel_interpolation_selects_the_source_upper_bound() {
        assert_eq!(color_lerp(0, 0, f32::NAN), 0xffff_ffff);
        assert_eq!(opacity_to_alpha(f32::NAN), 255);
    }

    #[test]
    fn channel_interpolation_rounds_the_mode_specific_source_stages() {
        let mix = f32::from_bits(0x3ed0_f81d);
        for (a, b) in [(101, 104), (47, 250), (255, 0)] {
            #[cfg(feature = "strict-fp")]
            let value = a as f32 * (1.0 - mix) + b as f32 * mix;
            #[cfg(not(feature = "strict-fp"))]
            let value = (a as f32).mul_add(1.0 - mix, b as f32 * mix);
            assert_eq!(lerp(a, b, mix), value.round() as u32);
        }
    }
}
