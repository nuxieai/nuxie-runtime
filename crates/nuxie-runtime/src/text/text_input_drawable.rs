//! TextInput drawable ownership ported from `src/text/text_input_drawable.cpp`.

pub(super) fn is_concrete(type_name: &str) -> bool {
    matches!(
        type_name,
        "TextInputCursor" | "TextInputSelectedText" | "TextInputSelection" | "TextInputText"
    )
}

pub(super) fn valid_parent(parent_type: Option<&str>) -> bool {
    parent_type == Some("TextInput")
}

pub(super) fn will_draw(super_will_draw: bool, render_opacity: f32) -> bool {
    super_will_draw && render_opacity != 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_parent_validation_and_will_draw_are_ported() {
        assert!(valid_parent(Some("TextInput")));
        assert!(!valid_parent(Some("Text")));
        assert!(will_draw(true, 1.0));
        assert!(!will_draw(true, 0.0));
        assert!(is_concrete("TextInputSelection"));
    }
}
