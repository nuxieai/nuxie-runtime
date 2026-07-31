// Direct owner for pinned C++ `src/bindable_artboard.cpp`.

/// A concrete Artboard source is a pointer-style binding: explicit null
/// removes the occurrence, an unresolved non-null target preserves it, and a
/// resolved equal target changes only when the binding explicitly reprojects.
fn bindable_artboard_requires_replacement(
    current_global_id: Option<u32>,
    resolved_global_id: u32,
    force_projection: bool,
) -> bool {
    force_projection || current_global_id != Some(resolved_global_id)
}
