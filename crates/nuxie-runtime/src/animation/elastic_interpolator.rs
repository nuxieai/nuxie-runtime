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
    let elastic = RuntimeElasticEase::new(amplitude, period);

    match easing_value {
        0 => elastic.ease_in(factor),
        1 => elastic.ease_out(factor),
        2 => elastic.ease_in_out(factor),
        _ => factor,
    }
}
