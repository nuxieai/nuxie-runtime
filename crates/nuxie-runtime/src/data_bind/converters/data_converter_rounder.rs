//! Direct owner for C++ `DataConverterRounder`.

pub(crate) fn convert(value: f32, decimals: u64) -> f32 {
    let scale = 10.0_f32.powf(decimals as f32);
    (value * scale).round() / scale
}
