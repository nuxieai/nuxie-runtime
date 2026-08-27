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
        color_red(color) as f32 / 255.0,
        color_green(color) as f32 / 255.0,
        color_blue(color) as f32 / 255.0,
        color_alpha(color) as f32 / 255.0,
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
    (255.0 * opacity.clamp(0.0, 1.0)).round() as u8
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
fn lerp(a: u32, b: u32, mix: f32) -> u32 {
    (a as f32 * (1.0 - mix) + b as f32 * mix)
        .clamp(0.0, 255.0)
        .round() as u32
}
pub fn color_lerp(from: ColorInt, to: ColorInt, mix: f32) -> ColorInt {
    color_argb(
        lerp(color_alpha(from), color_alpha(to), mix) as i32,
        lerp(color_red(from), color_red(to), mix) as i32,
        lerp(color_green(from), color_green(to), mix) as i32,
        lerp(color_blue(from), color_blue(to), mix) as i32,
    )
}
