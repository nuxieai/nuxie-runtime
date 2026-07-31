// Direct owner for pinned C++ `src/nested_artboard_layout.cpp`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeNestedLayoutBoundsCacheKey {
    graph_global_id: u32,
    layout_revision: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RuntimeNestedLayoutDataTransferKey {
    parent_layout: RuntimeNestedLayoutBoundsCacheKey,
    assigned_bounds: RuntimeLayoutBounds,
    child_layout_revision: u64,
}

#[derive(Debug, Clone)]
struct RuntimeNestedLayoutBoundsFrame {
    key: RuntimeNestedLayoutBoundsCacheKey,
    bounds: Arc<Option<BTreeMap<usize, RuntimeLayoutBounds>>>,
}

fn is_nested_artboard_layout(component: Option<&RuntimeComponent>) -> bool {
    component.is_some_and(|component| component.type_name == "NestedArtboardLayout")
}
