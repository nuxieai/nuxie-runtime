fn cubic_interpolator_calc_cubic_value(
    t: f32,
    value_from: f32,
    control_1: f32,
    control_2: f32,
    value_to: f32,
) -> f32 {
    let a = value_to + 3.0 * (control_1 - control_2) - value_from;
    let b = 3.0 * (control_2 - control_1 * 2.0 + value_from);
    let c = 3.0 * (control_1 - value_from);
    ((a * t + b) * t + c) * t + value_from
}
