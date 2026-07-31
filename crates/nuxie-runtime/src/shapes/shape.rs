//! Shape membership and PathComposer ownership are occurrence-local. The
//! retained owner is represented by `RuntimeShapeList`; this direct module is
//! the callback/lifecycle correspondence point for pinned `shape.cpp`.

pub(crate) fn can_defer_path_update(
    render_opacity: f32,
    clipping_or_never_defer: bool,
    has_skinned_path_dependent: bool,
    follow_path_consumer: bool,
) -> bool {
    render_opacity == 0.0
        && !clipping_or_never_defer
        && !has_skinned_path_dependent
        && !follow_path_consumer
}

pub(crate) fn needs_save_operation(container_needs_save: bool, paint_count: usize) -> bool {
    container_needs_save || paint_count > 1
}
