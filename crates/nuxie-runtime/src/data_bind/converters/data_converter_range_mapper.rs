//! Direct owner for C++ `DataConverterRangeMapper`.

use crate::RuntimeTransitionInterpolator;

pub(crate) fn convert(
    input: f32,
    min_input: f32,
    max_input: f32,
    min_output: f32,
    max_output: f32,
    flags: u64,
    interpolation_type: u64,
    interpolator: Option<RuntimeTransitionInterpolator>,
) -> f32 {
    if min_output == max_output {
        return min_output;
    }
    let mut value = input;
    if value < min_input && flags & 1 != 0 {
        value = min_input;
    } else if value > max_input && flags & 2 != 0 {
        value = max_input;
    }
    if (value < min_input || value > max_input) && flags & 4 != 0 {
        value = (crate::data_converter_operation::positive_mod(value, max_input - min_input)
            + min_input)
            .abs();
    }
    let mut percent = (value - min_input) / (max_input - min_input);
    if flags & 8 != 0 {
        percent = 1.0 - percent;
    }
    if let Some(interpolator) = interpolator {
        if percent > 0.0 && percent < 1.0 {
            percent = interpolator.transform(percent);
        } else if interpolation_type == 0 {
            percent = if percent <= 0.0 { 0.0 } else { 1.0 };
        }
    } else if interpolation_type == 0 {
        percent = if percent <= 0.0 { 0.0 } else { 1.0 };
    }
    percent * max_output + (1.0 - percent) * min_output
}
