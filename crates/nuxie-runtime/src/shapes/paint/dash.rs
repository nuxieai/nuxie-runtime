use crate::{ArtboardInstance, properties::property_key_for_name};

pub(crate) fn double_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("Dash", "length") != Some(property_key) {
        return None;
    }
    Some(invalidate_parent_dash_path(artboard, local_id))
}

pub(crate) fn bool_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("Dash", "lengthIsPercentage") != Some(property_key) {
        return None;
    }
    Some(invalidate_parent_dash_path(artboard, local_id))
}

fn invalidate_parent_dash_path(artboard: &mut ArtboardInstance, local_id: usize) -> bool {
    // Dash is a Core object rather than a Component, so the retained owner
    // index is the Rust equivalent of C++ `parent()->as<DashPath>()`.
    // Dash and its DashPath are registered against the same EffectPath.
    super::stroke_effect::invalidate_effect_from_local(artboard, local_id)
}

pub(crate) fn normalized_length(
    value: f32,
    percentage: bool,
    contour_length: f32,
    wraps: bool,
) -> f32 {
    let mut normalized = value;
    if wraps {
        let right = if percentage { 1.0 } else { contour_length };
        normalized = value % right;
        if normalized < 0.0 {
            normalized += right;
        }
    }
    if percentage {
        normalized * contour_length
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::normalized_length;

    #[test]
    fn zero_length_dash_stays_zero_without_crashing() {
        assert_eq!(normalized_length(0.0, false, 0.0, false), 0.0);
        assert_eq!(normalized_length(0.0, true, 0.0, false), 0.0);
    }
}
