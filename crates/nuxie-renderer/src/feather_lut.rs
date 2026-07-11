//! Canonical Gaussian lookup texture used by Rive's feather shaders.

pub(crate) const TABLE_SIZE: usize = 512;
const TEXTURE_STDDEVS: f32 = 3.0;

pub(crate) struct FeatherLut {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
}

impl FeatherLut {
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-feather-lut"),
            size: wgpu::Extent3d {
                width: TABLE_SIZE as u32,
                height: 2,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let rows = table_rows();
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&rows),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((TABLE_SIZE * size_of::<u16>()) as u32),
                rows_per_image: Some(2),
            },
            wgpu::Extent3d {
                width: TABLE_SIZE as u32,
                height: 2,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { texture, view }
    }
}

fn table_rows() -> [[u16; TABLE_SIZE]; 2] {
    let gaussian = gaussian_integral();
    let inverse = inverse_gaussian_integral();
    [gaussian.map(float_to_half), inverse.map(float_to_half)]
}

fn normal_distribution(x: f32, mu: f32, inverse_sigma: f32) -> f32 {
    const ONE_OVER_SQRT_2_PI: f32 = 0.398_942_3;
    let y = (x - mu) * inverse_sigma;
    (-0.5 * y * y).exp() * inverse_sigma * ONE_OVER_SQRT_2_PI
}

fn gaussian_integral() -> [f32; TABLE_SIZE] {
    const SAMPLES: i32 = 7;
    const EASE_IN_OUT_DISTANCE: usize = 8;

    let sigma = TABLE_SIZE as f32 / (TEXTURE_STDDEVS * 2.0);
    let inverse_sigma = 1.0 / sigma;
    let mu = TABLE_SIZE as f32 * 0.5;
    let mut table = [0.0; TABLE_SIZE];
    let mut integral = 0.0;
    for (i, entry) in table.iter_mut().enumerate() {
        for sample in 0..SAMPLES {
            let dx = (sample - (SAMPLES >> 1)) as f32 / SAMPLES as f32;
            integral += normal_distribution(i as f32 + dx, mu, inverse_sigma) / SAMPLES as f32;
        }
        *entry = integral;
    }

    let shift = 0.5 - (table[TABLE_SIZE / 2 - 1] + table[TABLE_SIZE / 2]) / 2.0;
    table[0] = (table[0] + shift).clamp(0.0, 1.0);
    for i in 1..TABLE_SIZE {
        table[i] = table[i - 1].max((table[i] + shift).clamp(0.0, 1.0));
    }
    for (i, entry) in table.iter_mut().enumerate() {
        if i < EASE_IN_OUT_DISTANCE {
            *entry *= i as f32 / EASE_IN_OUT_DISTANCE as f32;
        }
        if i > TABLE_SIZE - EASE_IN_OUT_DISTANCE - 1 {
            let t =
                (i - (TABLE_SIZE - EASE_IN_OUT_DISTANCE) + 1) as f32 / EASE_IN_OUT_DISTANCE as f32;
            *entry += (1.0 - *entry) * t;
        }
    }
    table
}

fn inverse_gaussian_integral() -> [f32; TABLE_SIZE] {
    const MULTIPLIER: usize = 32;

    let sigma = TABLE_SIZE as f32 / (TEXTURE_STDDEVS * 2.0);
    let inverse_sigma = 1.0 / sigma;
    let mu = TABLE_SIZE as f32 * 0.5;
    let samples = TABLE_SIZE * MULTIPLIER;
    let mut integral = 0.0;
    for i in 0..(samples + 1) / 2 {
        integral += normal_distribution(i as f32 / MULTIPLIER as f32, mu, inverse_sigma)
            / MULTIPLIER as f32;
    }
    integral = 0.5 - integral;

    let mut table = [0.0; TABLE_SIZE];
    table[TABLE_SIZE - 1] = 1.0;
    let mut last_inverse_x = f32::NAN;
    let mut last_inverse_y = 0.0;
    for i in 0..samples {
        let x = i as f32 / MULTIPLIER as f32;
        integral += normal_distribution(x, mu, inverse_sigma) / MULTIPLIER as f32;
        let inverse_x = integral.clamp(0.0, 1.0) * TABLE_SIZE as f32;
        let inverse_y = (i as f32 + 0.5) / samples as f32;
        let cell = inverse_x as usize;
        let cell_center_x = cell as f32 + 0.5;
        if cell_center_x == mu {
            table[cell] = 0.5;
        } else if last_inverse_x <= cell_center_x && inverse_x >= cell_center_x {
            let t = (cell_center_x - last_inverse_x) / (inverse_x - last_inverse_x);
            table[cell] = last_inverse_y + (inverse_y - last_inverse_y) * t;
        }
        last_inverse_x = inverse_x;
        last_inverse_y = inverse_y;
    }
    table
}

// This is the same finite/saturating conversion used by gpu::cast_f32_to_f16.
fn float_to_half(value: f32) -> u16 {
    let bits = value.to_bits().wrapping_add(0x1000);
    let exponent = (bits & 0x7f80_0000) >> 23;
    let mantissa = bits & 0x007f_ffff;
    let sign = (bits & 0x8000_0000) >> 16;
    let normalized = if exponent > 112 {
        (((exponent - 112) << 10) & 0x7c00) | (mantissa >> 13)
    } else {
        0
    };
    let denormalized = if (102..113).contains(&exponent) {
        (((0x007f_f000 + mantissa) >> (125 - exponent)) + 1) >> 1
    } else {
        0
    };
    let saturated = if exponent > 143 { 0x7fff } else { 0 };
    (sign | normalized | denormalized | saturated) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_rows_are_monotonic_with_canonical_endpoints() {
        let [gaussian, inverse] = table_rows();
        for row in [&gaussian, &inverse] {
            assert_eq!(row[0], 0);
            assert_eq!(row[TABLE_SIZE - 1], 0x3c00);
            assert!(row.windows(2).all(|pair| pair[0] <= pair[1]));
        }
        assert_eq!(gaussian[TABLE_SIZE / 2], 0x3805);
        assert_eq!(inverse[TABLE_SIZE / 2], 0x3801);
        assert_eq!(fnv1a(&gaussian), 0x0ca3_2867_e440_4413);
        assert_eq!(fnv1a(&inverse), 0xab56_7e66_1357_56cb);
    }

    #[test]
    fn half_conversion_matches_rive_boundaries() {
        assert_eq!(float_to_half(0.0), 0x0000);
        assert_eq!(float_to_half(0.5), 0x3800);
        assert_eq!(float_to_half(1.0), 0x3c00);
    }

    fn fnv1a(values: &[u16]) -> u64 {
        values.iter().fold(0xcbf2_9ce4_8422_2325, |hash, value| {
            value.to_le_bytes().into_iter().fold(hash, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
            })
        })
    }
}
