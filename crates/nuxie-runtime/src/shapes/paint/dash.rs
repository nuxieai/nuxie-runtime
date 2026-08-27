use crate::{ArtboardInstance, properties::property_key_for_name};

// Pinned `Dash()` retains only the generated 0/false fields, while
// `Dash(value, percentage)` assigns those same fields before the object has a
// parent. Rust materializes that state in the imported object arena and its
// `DashNode` projection; there is no constructor-only retained state here.

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

// `Dash::onAddedClean` is enforced at the import lifecycle boundary by
// `nuxie_binary::importers::dash_importer`: an accepted Dash has a DashPath
// parent before this runtime owner becomes callable.

fn length_changed(artboard: &mut ArtboardInstance, local_id: usize) -> bool {
    // `Dash` is a Component. Its retained parent relation is established
    // before this callback can run, and registration maps the Dash occurrence
    // to the same effect owner as its DashPath parent. Invalidating from the
    // Dash local therefore mirrors `parent()->as<DashPath>()->invalidateDash()`.
    super::stroke_effect::invalidate_effect_from_local(artboard, local_id)
}

fn length_is_percentage_changed(artboard: &mut ArtboardInstance, local_id: usize) -> bool {
    length_changed(artboard, local_id)
}

pub(crate) fn double_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("Dash", "length") != Some(property_key) {
        return None;
    }
    Some(length_changed(artboard, local_id))
}

pub(crate) fn bool_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("Dash", "lengthIsPercentage") != Some(property_key) {
        return None;
    }
    Some(length_is_percentage_changed(artboard, local_id))
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
