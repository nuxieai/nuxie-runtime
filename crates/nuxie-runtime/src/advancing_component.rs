use nuxie_graph::{AdvancingComponentKind, ArtboardGraph};

use crate::components::ComponentHandle;
use crate::objects::{InstanceObjectArena, ObjectHandle};

/// Occurrence-local counterpart of C++ `AdvancingComponent*`.
///
/// C++ discovers the interface from a `Core*` in
/// `src/advancing_component.cpp`; the Rust graph records that same dispatch
/// kind during import, while this value retains the concrete occurrence that
/// `Artboard` advances in authored order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeAdvancingComponent {
    pub(crate) local_id: usize,
    pub(crate) object: ObjectHandle,
    pub(crate) component: Option<ComponentHandle>,
    pub(crate) kind: AdvancingComponentKind,
}

/// Build the clone-owned counterpart of C++
/// `AdvancingComponent::from(Core*)` in authored object order.
pub(super) fn build_runtime_advancing_components(
    objects: &InstanceObjectArena,
    graph: &ArtboardGraph,
) -> Vec<RuntimeAdvancingComponent> {
    // The graph rows are a construction-only projection of the exact
    // `m_Objects` visitation and C++ family switch. Advancing accepts Core
    // (not Component), so ScriptedDataConverter deliberately carries no
    // ComponentHandle (`src/advancing_component.cpp:17-44`;
    // `src/artboard.cpp:330-395`).
    graph
        .advancing_components
        .iter()
        .filter_map(|entry| {
            Some(RuntimeAdvancingComponent {
                local_id: entry.local_id,
                object: objects.object_handle(entry.local_id)?,
                component: objects.component_handle(entry.local_id),
                kind: entry.kind,
            })
        })
        .collect()
}
