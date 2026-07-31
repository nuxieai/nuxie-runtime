//! ListPath owns one CubicDetachedVertex listener per retained list item.
//! DataBind list membership remains in the retained DataBind family; vertex
//! writes use the same generated callback chain as direct writes.

pub(crate) fn degrees_to_radians(value: f32) -> f32 {
    value * (std::f32::consts::PI / 180.0)
}

pub(crate) fn point_to_distance_rotation(x: f32, y: f32) -> (f32, f32) {
    ((x * x + y * y).sqrt(), y.atan2(x))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_listener_units_match_pinned_list_path() {
        assert_eq!(degrees_to_radians(180.0), std::f32::consts::PI);
        assert_eq!(
            point_to_distance_rotation(3.0, 4.0),
            (5.0, 4.0_f32.atan2(3.0))
        );
    }
}
