pub(crate) fn argb(alpha: i32, red: i32, green: i32, blue: i32) -> u32 {
    (((alpha & 0xff) as u32) << 24)
        | (((red & 0xff) as u32) << 16)
        | (((green & 0xff) as u32) << 8)
        | ((blue & 0xff) as u32)
}

pub(crate) fn red(value: u32) -> u8 {
    ((value >> 16) & 0xff) as u8
}

pub(crate) fn green(value: u32) -> u8 {
    ((value >> 8) & 0xff) as u8
}

pub(crate) fn blue(value: u32) -> u8 {
    (value & 0xff) as u8
}

pub(crate) fn alpha(value: u32) -> u8 {
    ((value >> 24) & 0xff) as u8
}

pub(crate) fn unpack_rgba8(value: u32) -> [u8; 4] {
    [red(value), green(value), blue(value), alpha(value)]
}

pub(crate) fn unpack_rgba32f(value: u32) -> [f32; 4] {
    unpack_rgba8(value).map(|channel| f32::from(channel) / 255.0)
}

pub(crate) fn unpack_rgba32f_premultiplied(value: u32) -> [f32; 4] {
    let [red, green, blue, alpha] = unpack_rgba32f(value);
    [red * alpha, green * alpha, blue * alpha, alpha]
}

pub(crate) fn opacity(value: u32) -> f32 {
    f32::from(alpha(value)) / 255.0
}

pub(crate) fn opacity_to_alpha(opacity: f32) -> u8 {
    (255.0 * opacity.clamp(0.0, 1.0)).round() as u8
}

pub(crate) fn with_alpha(value: u32, alpha: u32) -> u32 {
    argb(
        alpha as i32,
        red(value).into(),
        green(value).into(),
        blue(value).into(),
    )
}

pub(crate) fn with_opacity(value: u32, opacity: f32) -> u32 {
    with_alpha(value, u32::from(opacity_to_alpha(opacity)))
}

pub(crate) fn modulate_opacity(color: u32, opacity: f32) -> u32 {
    with_opacity(color, self::opacity(color) * opacity)
}

pub(crate) fn lerp(from: u32, to: u32, mix: f32) -> u32 {
    let channel = |shift: u32| {
        let from = ((from >> shift) & 0xff) as f32;
        let to = ((to >> shift) & 0xff) as f32;
        ((from * (1.0 - mix) + to * mix).clamp(0.0, 255.0)).round() as u32
    };
    (channel(24) << 24) | (channel(16) << 16) | (channel(8) << 8) | channel(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_color_unpacking_matches_pinned_argb_layout() {
        let color = 0x1234_5678;
        assert_eq!(unpack_rgba8(color), [0x34, 0x56, 0x78, 0x12]);

        let unpacked = unpack_rgba32f(color);
        assert_eq!(
            unpacked,
            [
                0x34 as f32 / 255.0,
                0x56 as f32 / 255.0,
                0x78 as f32 / 255.0,
                0x12 as f32 / 255.0
            ]
        );

        let premultiplied = unpack_rgba32f_premultiplied(color);
        assert_eq!(premultiplied[0], unpacked[0] * unpacked[3]);
        assert_eq!(premultiplied[1], unpacked[1] * unpacked[3]);
        assert_eq!(premultiplied[2], unpacked[2] * unpacked[3]);
        assert_eq!(premultiplied[3], unpacked[3]);
    }

    #[test]
    fn packed_color_lerp_matches_pinned_channel_rounding_and_clamping() {
        assert_eq!(lerp(0xff00_0000, 0xffff_ffff, 0.5), 0xff80_8080);
        assert_eq!(lerp(0x9090_9090, 0x1e1e_1e1e, 1.3), 0);
        assert_eq!(modulate_opacity(0x8033_6699, 0.5), 0x4033_6699);
        assert_eq!(with_alpha(0x1234_5678, 0xab), 0xab34_5678);
    }
}
