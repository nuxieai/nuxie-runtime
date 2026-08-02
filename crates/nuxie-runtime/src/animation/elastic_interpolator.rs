fn elastic_interpolator_transform(
    factor: f32,
    amplitude: f32,
    serialized_period: f32,
    easing_value: u64,
) -> f32 {
    let period = if serialized_period == 0.0 {
        0.5
    } else {
        serialized_period
    };
    let shift = if amplitude < 1.0 {
        period / 4.0
    } else {
        period / (2.0 * std::f32::consts::PI) * (1.0 / amplitude).asin()
    };

    match easing_value {
        0 => elastic_ease_in(factor, amplitude, period, shift),
        1 => elastic_ease_out(factor, amplitude, period, shift),
        2 => elastic_ease_in_out(factor, amplitude, period, shift),
        _ => factor,
    }
}
