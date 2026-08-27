fn cubic_ease_interpolator_transform_value(
    value_from: f32,
    value_to: f32,
    factor: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
) -> f32 {
    value_from
        + (value_to - value_from)
            * cubic_ease_interpolator_transform(factor, x1, y1, x2, y2)
}

fn cubic_ease_interpolator_transform(
    factor: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
) -> f32 {
    cubic_interpolator_calc_bezier(cubic_interpolator_get_t(factor, x1, x2), y1, y2)
}
