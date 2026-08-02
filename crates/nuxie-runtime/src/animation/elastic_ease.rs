fn elastic_actual_amplitude(time: f32, amplitude: f32, shift: f32) -> f32 {
    if amplitude < 1.0 {
        let shift_abs = shift.abs();
        let time_abs = time.abs();
        if time_abs < shift_abs {
            let l = time_abs / shift_abs;
            return (amplitude * l) + (1.0 - l);
        }
    }

    amplitude
}

fn elastic_ease_out(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    let time = factor;
    let actual_amplitude = elastic_actual_amplitude(time, amplitude, shift);
    actual_amplitude
        * 2.0_f32.powf(10.0 * -time)
        * ((time - shift) * (2.0 * std::f32::consts::PI) / period).sin()
        + 1.0
}

fn elastic_ease_in(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    let time = factor - 1.0;
    let actual_amplitude = elastic_actual_amplitude(time, amplitude, shift);
    -(actual_amplitude
        * 2.0_f32.powf(10.0 * time)
        * ((-time - shift) * (2.0 * std::f32::consts::PI) / period).sin())
}

fn elastic_ease_in_out(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    let time = factor * 2.0 - 1.0;
    let actual_amplitude = elastic_actual_amplitude(time, amplitude, shift);
    if time < 0.0 {
        -0.5 * actual_amplitude
            * 2.0_f32.powf(10.0 * time)
            * ((-time - shift) * (2.0 * std::f32::consts::PI) / period).sin()
    } else {
        0.5 * (actual_amplitude
            * 2.0_f32.powf(10.0 * -time)
            * ((time - shift) * (2.0 * std::f32::consts::PI) / period).sin())
            + 1.0
    }
}
