//! TextInput drawable ownership ported from `src/text/text_input_drawable.cpp`.

use crate::Mat2D;

pub(super) fn is_concrete(type_name: &str) -> bool {
    matches!(
        type_name,
        "TextInputCursor" | "TextInputSelectedText" | "TextInputSelection" | "TextInputText"
    )
}

pub(super) fn valid_parent(parent_type: Option<&str>) -> bool {
    parent_type == Some("TextInput")
}

/// `TextInputDrawable::worldPath` is deliberately unreachable. A Stroke that
/// does not let its transform affect the stroke asks for this path in pinned
/// C++; preserve that invalid authored combination at the concrete owner.
pub(super) fn world_path() -> ! {
    unreachable!("TextInputDrawable::worldPath is unreachable")
}

pub(super) fn shape_world_transform(world_transform: Mat2D) -> Mat2D {
    world_transform
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_parent_validation_is_ported() {
        assert!(valid_parent(Some("TextInput")));
        assert!(!valid_parent(Some("Text")));
        assert!(is_concrete("TextInputSelection"));
    }
}
