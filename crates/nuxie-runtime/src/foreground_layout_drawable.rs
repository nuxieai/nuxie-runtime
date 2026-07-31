use crate::artboard::ArtboardInstance;

/// Direct retained-parent lookup for `ForegroundLayoutDrawable`. The drawable
/// borrows layout geometry from its literal LayoutComponent parent.
pub(crate) fn parent_layout_local(
    instance: &ArtboardInstance,
    foreground_local: usize,
) -> Option<usize> {
    instance
        .component_parent_local(foreground_local)
        .filter(|parent_local| {
            instance
                .component(*parent_local)
                .is_some_and(|component| component.type_name == "LayoutComponent")
        })
}
